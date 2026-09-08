use std::future::Future;
use std::time::SystemTime;

use async_std::task;
use futures::future::join;

// En este codigo se ve que es lo que ocurre cuando usamos `async` para calculos pesados de procesador en 
// lugar de operaciones de I/O.

// Recursion en funciones `async`:
// - En Rust una funcion asincrona se compila como una estructura que contiene sus variables. Si una
//      funcion `async` se llama asi misma recursivamente sin mas, Rust calcularia que el tamaño en memoria
//      de esa estructura es infinito y arrojara un error de compilacion.
// - Al envolver la llamada recursiva en un `Box::pin`, alojamos el Future hijo en la memoria dinamica (heap) 
//      dejando en la estructura principal unicamente un puntero de tamaño fijo.
// - Con Box::pin` le aseguramos al compilador que dicha función sera inamovible de esa dirección de memoria.
//      De esta forma, las auto-referenicas no se corrompen. 
async fn fibonacci(n: u32) -> u32 {
    if n <= 1 {
        n
    } else {
        // Aunque son asíncronas, esto se evalúa de izquierda a derecha.
        // Primero se calcula todo el fibonacci(n-1), y solo cuando termina,
        // arranca fibonacci(n-2).
        Box::pin(fibonacci(n - 1)).await + Box::pin(fibonacci(n - 2)).await
    }
}

async fn measure<Fut: Future>(f: Fut) {
    let start = SystemTime::now();
    f.await;
    println!("{:?}", SystemTime::now().duration_since(start));
}

fn main() {
    task::block_on(async {
        // Ejecución aislada: Tarda un tiempo "X"
        measure(fibonacci(34)).await;
        // Ejecución aislada: Tarda un timepo "Y"
        measure(fibonacci(36)).await;

        // ¿Porque join no corre en paralelo en este caso?
        // Porque `fibonacci` es puro calculo del procesador. No lee discos, no espera peticiones web ni
        //  usa temporizadores. Como no hay ninguna operacion externa que la haga esperar, esta funcion
        //  nunca devuelve `Pending`, entonces, corre de principio a fin.
        //
        //- Tarda un tiempo "X + Y"
        measure(join(fibonacci(34), fibonacci(36))).await;
    });
}