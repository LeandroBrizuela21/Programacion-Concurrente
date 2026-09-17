extern crate rand;

use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;
use std::thread::JoinHandle;
use rand::{Rng, thread_rng};

// En este codigo se implementa un sistema multi-thread concurrente donde 
// cinco workers comparten y modifican un estado global numerico utilizando
// Locks de lectura y escritura. 

// HERRAMIENTAS UTILIZADAS:
// - `Arc`: Permite que multiples hilos compartan la propiedad del mismo dato en memoria.
//          * Cada `.clone()` aumenta un contador interno. Cuando todos los hilos mueran, 
//            este contador llegara a cero y la memoria se liberara.              
// - `RwLock`: Permite que varios hilos lean el dato al mismo tiempo, pero cuando un hilo
//             necesita modificarlo, exige acceso exclusivo y bloquea todos los demas lectores
//             y escritores hasta terminar.

const WORKERS: u32 = 5;

fn worker(id:u32,  signal_source:Arc<RwLock<f64>>) {
    loop {
        // El worker solicita el Lock de lectura y copia el valor actual. 
        // Si no hay nadie escribiendo, múltiples hilos pueden leer al mismo tiempo.
        let signal = *signal_source.read().unwrap() / (WORKERS as f64);
        println!("[WORKER {}] inicio con señal {}", id, signal);
        // Genera un numero aleatorio
        let rand = thread_rng().gen_range(0.0, 1.0);
        // Congela el hilo (como es hilo del sistema operativo, no afecta a los otros workers).
        thread::sleep(Duration::from_millis((2000.0 * rand) as u64));
        
        // Cálculo del resultado intermedio de forma estrictamente local en el hilo.
        let result = signal * rand;
        
        // ESCRITURA - SECCIÓN CRÍTICA
        // Solicita el Lock de escritura.
        // - Si otro worker esta escribiendo/leyendo en ese instante, el hilo se pausa y espera su turno. 
        if let Ok(mut guard) = signal_source.write() {
            // Una vez obtenido el Lock, garantizamos ser el único hilo en modificar la variable compartida.
            *guard += result - signal;
        }
        println!("[WORKER {}] resultado {}", id, result);
    }

}


fn main() {
    // Creamos el estado global compartido entre los hilos, inicializado en 100.0
    // Se envuelve en `RwLock` para el control de concurrencia y luego en `Arc` para poder repartir
    // referencias hacia distintos hilos.
    let signal = Arc::new(RwLock::new(100.0));

    // Creamos los hilos.
    let workers: Vec<JoinHandle<()>> = (0..WORKERS)
        .map(|id| {
            // Clonamos el `Arc`, lo que incrementa su contador de referencias atómico en 1.
            // Cada hilo obtendrá su propia copia de este puntero hacia el dato central compartido.
            let signal_local = signal.clone();
            
            // Se lanza un nuevo hilo a nivel del sistema operativo. La palabra `move` obliga al closure 
            // a tomar propiedad absoluta de las variables capturadas (`id` y`signal_local`).
            thread::spawn(move || worker(id, signal_local))
        })
        // Recolectamos los `JoinHandle` reaultantes en un vector.
        .collect();

    // El hilo principal se bloquea esperando a que todos los workers terminen. 
    // Iteramos sobre los handlers invocamos `join()` en cada uno para esperar su ejecución y descartamos
    // (drop) los resultados del `join()`
    workers.into_iter()
        .flat_map(|x| x.join())
        .for_each(drop);
}