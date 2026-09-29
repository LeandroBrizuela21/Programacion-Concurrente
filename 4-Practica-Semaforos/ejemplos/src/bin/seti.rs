extern crate rand;

use std::sync::{Arc, RwLock, Barrier};
use std::thread;
use std::time::Duration;
use rand::Rng;

// En este codigo se simula un sistema de procesamiento distribuido por "epocas" o ciclos de trabajo,
// donde multiples trabajadores operan en paralelo sobre un recurso centralizado representado por una
// variable `signal`. 
// - Utiliza barretas (`Barriers`) para obligar a todos los hilos a sincronizarse.
// - Tambien utiliza un Lock de lectura/escritura (`RwLock`) para permitir que los hilos lean el estado
//   simultaneamente pero lo modifiquen de forma segura y exclusiva. 

const WORKERS: u32 = 10;

fn main() {
    // Inicializamos el recurso compartido.
    let signal: f64 = 100.0;
    
    // Envolvemos la señal en un `RwLock`, para escrituras simultaneas, y luego en un 
    // Arc para poder repartir copias a cada hilo.
    let lock = Arc::new(RwLock::new(signal));

    // Creamos dos barreras.
    // El número de WORKERS le indica a la barrera a cuántos hilos tiene que esperar 
    // antes de abrirse.
    let barrier = Arc::new(Barrier::new(WORKERS as usize));
    let barrier2 = Arc::new(Barrier::new(WORKERS as usize));

    let mut workers = vec![];

    // Clonamos las referencias y lanzamos los 10 hilos trabajadores.
    for id in 0..WORKERS {
        let lock_clone = lock.clone();
        let barrier_clone = barrier.clone();
        let barrier2_clone = barrier2.clone();
        workers.push(thread::spawn(move || worker(id, lock_clone, barrier_clone, barrier2_clone)));
    }

    // El hilo principal espera a que todos los workers terminen.
    for worker in workers {
        worker.join().unwrap();
    }

}

fn worker(id: u32, lock: Arc<RwLock<f64>>, barrier: Arc<Barrier>, barrier2: Arc<Barrier>) {
    let mut epoch = 0;

    loop {
        // --- PUNTO DE CONTROL 1 ---
        // Todos los hilos se detienen aquí. Nadie empieza la ronda 'epoch' hasta que
        // los 10 trabajadores estén listos.
        barrier.wait();

        // --- FASE DE LECTURA ---
        // Usamos '.read()'. Como es un RwLock, los 10 hilos pueden entrar a leer exactamente
        // al mismo tiempo sin hacer fila.
        let signal = *lock.read().unwrap() / WORKERS as f64;
        println!("[WORKER {}] inicio epoch {} signal {}", id, epoch, signal);
       
       // --- PUNTO DE CONTROL 2 ---
       // ¿Por que es crucial esto?
       // Si un hilo es rapidisimo, podria pasar a la siguiente linea y restar su dinero (escribir)
       // mientras los hilos lentos todavía están intentando leer el valor original de arriba.
       // Aseguramos que nadie reste dinero hasta que todos hayan terminado de leer.
        barrier2.wait();

        // --- FASE DE EXTRACIÓN ---
        // Usamos '.write()'. Solo un hilo a la vez puede escribrir, los otros 9 hacen fila.
        if let Ok(mut money_guard) = lock.write() {
            *money_guard -= signal;
        }

        epoch += 1;
        
        // --- SIMULACIÓN DE TRABAJO ---
        // Cada trabajador genera un numero aleatorio y duerme un tiempo basado en el.
        // Aca no se necestan candados (Locks) porque cada uno usa sus propias variables locales.
        let mut rng = rand::thread_rng();
        let random_result: f64 = rng.gen_range(0.0, 1.0);
        thread::sleep(Duration::from_millis((2000 as f64 * random_result) as u64));
        
        // El worker calcula cuanto dinero va a devolver.
        let result = signal * random_result;
        println!("[WORKER {}] voy a retornar {}", id, result);

        // --- FASE DE DEVOLICIÓN ---
        // Nuevamente pedimos permisos exclusivos para escribir y sumar el resultado.
        if let Ok(mut guard) = lock.write() {
            *guard += result;
        }

        println!("[WORKER] {} retorné {}", id, result);
    }
}