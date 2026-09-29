use std::thread;
use std::time::{Duration, SystemTime};

// Este codigo presenta dos lecciones importantes:
// - Ejecutar tareas de forma verdaderamente concurrente usando `join!`.
// - NO meter codigo bloqueante sincronico (`thread::sleep`) dentro de una funcion `asyc`.

use async_std::task;
use futures::join;

async fn hello() -> String {
    println!("before hello");
    // Al usar `task::sleep` con `await` la tarea se comporta de forma cooperativa.
    // Devuelve `Pending` y le avisa al motor, que lo despierte despues de 2 segundos,
    // mientras puede usar este hilo para otras tareas.
    task::sleep(Duration::from_secs(2)).await;
    
    // Esta instrucción congela el hilo en su totalidad, sin posibilidad de pueda operar
    // otras tareas.
    //thread::sleep(Duration::from_secs(2));
    println!("after hello");
    String::from("Hello")
}

async fn world() -> String {
    println!("before world");
    task::sleep(Duration::from_secs(1)).await;
    println!("after world");
    String::from(" World!")
}

async fn async_main() -> String {
    // `join` toma las Maquina de estados de `hello` y `world` agrupandolas.
    // - Hace `poll` a `hello()`. Este imprime "before hello", choca con su sleep asincrono
    //   y devuele `Pending`.
    // - En el mismo microsegundo, `join!` avanza y hace `poll` a `world()`. 
    //      Este imprime "before world", choca con su sleep asincrono y devuele `Pending`.
    // - A 1 segundo de espera: `world()` despierta, se pollea de nuevo, imprime "after world" y termina (`Ready`).
    // - A los 2 segundos de espera: `hello()` despierta, se pollea de nuevo, imprime "after hello" y termina (`Ready`).
    // - Con ambas listas, `join!` entrega la tupla final (h, w).
    let (h, w) = join!(hello(), world());
    h + w.as_str()
}

fn main() {
    let start = SystemTime::now();
    println!("{}", task::block_on(async_main()));
    println!("{:?}", SystemTime::now().duration_since(start))
}