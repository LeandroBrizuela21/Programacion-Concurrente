extern crate actix;

use std::{io};
use std::time::{Duration, SystemTime};

use actix::{Actor, Context, Handler, Message, System};
use std::io::Read;
use actix_async_handler::async_handler;

// En este codigo se explora como el framework Actix maneja el tiempo, las pausas y el bloqueo de hilos. 
// El codigo ilustra la diferencia fundamental entre hacer que un Actor espere de forma asíncrona 
// (liberando recursos del sistema para que otros Actores trabajen) versus bloquear un hilo por completo.
// También demuestra empiricamente que, aunque varios Actores diferentes funcionan al mismo tiempo en paralelo,
// un Actor individual procesa los mensajes de su propio buzon de manera estrictamente secuencial,
// encolando el trabajo.


// Mensaje con la finalidad de dormir por x cantidad de segundos.
#[derive(Message)]
#[rtype(result = "()")]
struct Sleep(u64);

// === ACTOR DORMILON ===
struct Sleepyhead {
    id: usize
}

// Convertimos el `struct` normal en un Actor de Actix.
impl Actor for Sleepyhead {
    type Context = Context<Self>;
}

// === HANDLER ASÍNCRONICO ===
#[async_handler]
impl Handler<Sleep> for Sleepyhead {
    type Result = ();

    async fn handle(&mut self, msg: Sleep, _ctx: &mut <Sleepyhead as Actor>::Context) -> Self::Result  {
        println!("[{}] durmiendo por {}", self.id, msg.0);
        
        // - tokio::time::sleep es asíncronico.
        // - EL actor suspende la ejecución de esta tarea, pero no bloquea el hilo del sistema operativo.
        // - Otros actores pueden usar el procesador.
        // - Sin embargo, su propio buzón queda en pausa hasta que despierte.        
        tokio::time::sleep(Duration::from_secs(msg.0)).await;
        println!("[{}] desperté de {}", self.id, msg.0);
    }
}


// === Hanlder que detiene por completo el sisteam ===
// impl Handler<Sleep> for Sleepyhead {
//     type Result = ();
//
//     fn handle(&mut self, msg: Sleep, _ctx: &mut <Sleepyhead as Actor>::Context) -> Self::Result  {
//         println!("[{}] durmiendo por {}", self.id, msg.0);
//         
//         - thread::sleep es sincrónico (bloqueante).
//         - Si se realiza esto en Actrix, congela el hilo de trabajo entero 
//         - Si hay otros actores viviendo en este mismo hilo, quedaran totalmente paralizados, creando
//           un cuello de botella.
//         thread::sleep(Duration::from_secs(msg.0 * 10));
//         println!("[{}] desperté de {}", self.id, msg.0);
//     }
// }


// Iniciamos el motor asicrono y definimos el punto de entrada.
#[actix_rt::main]
async fn main() {
    // console_subscriber::init();
    
    // Pausamos la ejecucion del programa esperando que el Usuario presione "Enter" en la consola
    // antes de lanzar los Actores.
    println!("Enter para empezar");
    io::stdin().read(&mut [0u8]).unwrap();

    // Instanciamos un Actor `Sleepyhead` con ID 1 guardando su direccion en `addr`.  
    let addr = Sleepyhead { id: 1}.start(); 
    // Instanciamos un Actor `Sleepyhead` con ID 2 guardando su direccion en `other`.
    let other = Sleepyhead { id: 2 }.start(); 
    
    
    //  === Solución a tareas pesadas ===
    // Si se debe usar si o si `thread::sleep` o hacer cálculos pesados, levantamos al actor usando 'SyncArbiter'
    // Esto le asigana hilos dedicados (2 en este caso) separandolos del sistema asíncrono principal, para que 
    // puedan bloquearse sin molestar a nadie.
    
    // Parametros:
    // - El primer parametros nos indica la cantidad de hilos dedicados que tendra ese actor.
    //   De esta forma, dos hilos, con el mismo actor clonado, pueden atender la misma cola de mensajes.
    // - El sedungo parametro, es la fabrica de Acotores.
    // let addr = SyncArbiter::start(1, || Sleepyhead { id: 1 });
    // let addr = SyncArbiter::start(2, || Sleepyhead { id: 1 });
    // let other = SyncArbiter::start(1, || Sleepyhead { id: 2 });

    // Iniciamos un cronometro para medir la duracion total de la prueba.
    let now = SystemTime::now();
    
    // Enviamos de forma no bloqueante (dispara y olvida) la orden de dormir 3 segundos al Actor 1.
    addr.try_send(Sleep(3)).unwrap();
    println!("El Actor 1 duerme 3 segundos");

    // Enviamos de forma no bloqueante (dispara y olvida) la orden de dormir 2 segundos al Actor 2.
    // - En este punto, ambos actores duermen al mismo tiempo de manera concurrente.
    other.try_send(Sleep(2)).unwrap();
    println!("El Actor 2 duerme 2 segundos");
    
    

    // Se envia un segundo mensaje al Actor 1 pidiendole dormir otros 2 segundos.
    // - Esta vez el hilo prinicipal se bloquea con `.await` para esperar a que termine el trabajo.
    // - Como los Actores encolan los mensajes de su buzon en orden, el Actor 1 primero duerme los 
    //   3 segundos originales y recien despues procesa este mensaje de 2 segundos. 
    addr.send(Sleep(2)).await.unwrap();
    
    // Calcula e imprime el tiempo total. El resultado será ~5 segundos, que es el tiempo total acumulado 
    // del Actor 1 (3s + 2s), demostrando que el actor 2 ejecuto su pausa de 2 segundos de forma paralela
    // en el fondo sin estorbar.
    println!("Terminé. Tardé {}", now.elapsed().unwrap().as_secs());

    // Pausamos la ejecucion del programa esperando que el Usuario presione "Enter" en la consola
    // antes de terminar.
    println!("Enter para terminar");
    io::stdin().read(&mut [0u8]).unwrap();

    // Cerramos el sistema de Actores y salimos del programa de forma segura.
    System::current().stop();
}