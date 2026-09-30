extern crate rand;

use std::collections::HashSet;
use std::thread;
use std::time::Duration;
use rand::{thread_rng, Rng};
use std::thread::JoinHandle;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};

// Este codigo implementa una solucion al PROBLEMA SETI utiliazando paso de mensajes mediante canales (mpsc). 
// En este diseño existe un hilo "Coordinador" (el principal) que se encarga de dividir el trabajo y enviarle
// a cada Trabajador su porcion directamente por una tuberia de comunicacion dedicada. 
// Los Trabajadores procesan la informacion y devuelven el resultado a traves de un unico canal de retorno 
// compartido. El Coordinador sincroniza las "epocas" simplemente esperando a recibir exactamente una respuesta
// de cada Trabajador antes de sumar los resultados y enviar la siguiente tanda de trabajo.

const WORKERS: i32 = 10;

fn main() {
    // Señal de inicio global
    let mut signal = 100.0;

    // Creamos un canal MPSC:
    // - `result_send`: Se lo queda el Coordinador para escuchar. 
    // - `result_receive`: Se clonará para dárselo a cada Trabajador.
    let (result_send, result_receive) = mpsc::channel();

    // -- CREACION DE WORKERS
    let workers: Vec<(Sender<f64>, JoinHandle<()>)> = (0..WORKERS)
        .map(|id| {
            // Cada Trabajador necesita su propio canal para que el Coordinador le hable en privado.
            let (worker_send, worker_receive) = mpsc::channel();
            
            // Le damos una copia del transmisor central a este worker.
            let result_send_worker = result_send.clone();
            
            // Nace el hilo. Se lleva su receptor privado y el transmisor cental.
            let t = thread::spawn(move || worker(id, worker_receive, result_send_worker));
            
            // Guardamor el transmisor pricado del worker y el control del hilo para usarlo luego.
            (worker_send, t)
        })
        .collect();

    // EL CLICLO DE ÉPOCAS - Trabajo del Coordinador
    loop {
        // Reparte la señal actual entre los 10 Trabajadores.
        let mut signal_epoch = start_epoch(&mut signal, &workers);
        
        // Una lista de asistencia para saber qué worker ya nos respondieron.
        let mut results = HashSet::new();

        // Esperamos hasta tener exactamente 10 respuestas de 10 workers distintos.
        while(results.len() < (WORKERS as usize)) {
            
            // El Coordinador se bloquea aca esperando que le llege ALGO por el tubo central.
            let (who, result) = result_receive.recv().unwrap();
            println!("[COORDINADOR] recibí de {} señal {}", who, result);
            
            // Si es la primera vez que este Trabajador responde en esta época, lo marcamos como presente.
            if !results.contains(&who) {
                results.insert(who);
                signal_epoch += result;
            }
        }

        println!("[COORDINADOR] señal final {}", signal_epoch);
        signal = signal_epoch
    }
    // Esperamos a que los hilos terminen
    let _:Vec<()> = workers.into_iter()
        .flat_map(|(_,h)| h.join())
        .collect();
}


// Función para repartir el trabajo
fn start_epoch(signal: &mut f64, workers: &Vec<(Sender<f64>, JoinHandle<()>)>) -> f64 {
    // Divide la señal en 10 partes iguales.
    let signal_worker = *signal / (WORKERS as f64);
    
    // Le envia la porción a cada worker por su canal privado.
    // Como los canales son asíncronos por defecto en Rust, esto no bloquea al coordinador.
    for (worker, _) in workers {
        worker.send(signal_worker).unwrap();
    }

    let mut signal_epoch = 0.0;
    signal_epoch
}

// Definimos el comportamiento del Trabajador.
fn worker(id: i32, signal_source: Receiver<f64>, result: Sender<(i32, f64)>) {
    loop {
        // El Trabajador se bloquea en su canal personal esperando a que el Coordinador le envie su porcion.
        let signal = signal_source.recv().unwrap();
        println!("[WORKER {}] señal {}", id, signal);
        // Simula el procesamiento.
        thread::sleep(Duration::from_secs(2));
        // Calcula su aporte aleatorio basado en la señal recibida.
        let resultado = signal * thread_rng().gen_range(0., 1.);
        println!("[WORKER {}] resultado {}", id, resultado);
        // Empaqueta su ID junto con el resultado calculado en una tupla, y lo despacha por la tuberia 
        // compartida hacia el Coordinador.
        result.send((id, resultado));
    }
}