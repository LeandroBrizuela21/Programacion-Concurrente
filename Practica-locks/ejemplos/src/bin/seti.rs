extern crate rand;

use std::time::Duration;

use std::thread;
use std::thread::JoinHandle;

use rand::{Rng};

// En este codigo se implementa el modelo concurrente de Fork-Join.
// Ciclo de sincronizacion por epocas (epoch).
// 1. Fork: Tomamos la señal acumulada, la dividimos en cinco partes y reiniciamos el contador central a 0.
// 2. Ejecución: Cada worker recibe su parte y calcula un resultado.
// 3. Join: Esperamos a que los cinco workers terminen, sumamos los retornos para reconstruir la señal global
//          y asi avanza a la siguiente epoca.

// DESVENTAJAS:
// - Creamos y destruimos threads todo el tiempo: El sistema operativo tiene que asginar memoria, 
//   crear los threads, ejecutarlos, y luego destruirlos y limpiar su memoria en cada epoca.
//      * Pedirle al sistema operativo que cree threads constantemente es una operacion muy lenta a 
//        nivel procesador.
// - Posibilidad cuello de botella por un worker mas lento.

const WORKERS: i32 = 5;

fn worker(id: i32, signal: f64) -> f64 {
    println!("[WORKER {}] recibo {}", id, signal);

    // Para probar panic en thread
    if signal < 1.0 {
        panic!("señal muy debil, no se puede analizar");
    }

    // Genera un numero aleatorio
    let random_result: f64 = rand::thread_rng().gen_range(0.0, 1.0);
    
    // Congela el hilo (como es hilo del sistema operativo, no afecta a los otros workers).
    thread::sleep(Duration::from_millis((random_result * 1000.0) as u64));

    // Cálculo del resultado intermedio de forma estrictamente local en el hilo.
    let result = signal * random_result;

    println!("[WORKER {}] devuelvo {}", id, result);
    result
}


fn main() {

    // Variable no comportadida. Solo le pertenece al hilo principal.
    let mut signal = 100.0;
    let mut epoch = 1;

    // Para pensar: ¿Por qué dos ciclos for?
    // Si hicieramos el `spawn` e inmediatamente el `join` en el mismo ciclo, no habria concurrencia. 
    // El hilo principal se bloquearia esperando a que el worker termine antes de crear el siguiente.
    
    
    loop {
        println!("[COORDINADOR] epoch {}, señal {}", epoch, signal);
        let mut workers = vec![];
        
        // Se calcula que porción de la señal original le toca a cada worker.
        let signal_worker = signal / (WORKERS as f64);
        
        // Reseteamos la señal del coordinador a 0 para empezar a acumular los nuevos resultados
        // que devuelven los workers más adelante. 
        signal = 0.0;

        // Fork: Se crean los hilos workers y se les asigna la porción de señal que les corresponde.
        for id in 0..WORKERS {
            // `thread::spawn` arranca un hilo en segundo plano de forma inmediata.
            // `move` indica que el hilo se queda con la propiedad de las variables que se le pasan.
            // Al hacer push en el vector `workers` se guarda un `JoinHandle` que permite esperar a que el 
            // hilo termine y recuperar su valor de retorno.
            workers.push(thread::spawn(move || worker(id, signal_worker)))
        }


        // Join: Se espera a que todos los hilos terminen y se suman sus resultados para recontruir la señal global.
        for worker in workers {
            // `join()` es una llamada bloqueante. El programa se frena hasta que el `worker` especifico termine de ejecutarse 
            // y devuelva su valor.
            match worker.join() {
               // Si el hilo termino con exito, desempaquetamos el valor devuelto y lo suamamos a la señal de esta época.
                Ok(processed) => signal += processed,
                Err(str) => panic!("{:?}", str)
            }
        }

        epoch += 1
    }
}