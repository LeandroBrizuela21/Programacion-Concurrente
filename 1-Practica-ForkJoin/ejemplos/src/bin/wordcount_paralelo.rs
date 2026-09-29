use std::collections::HashMap;
use std::{env, thread};
use std::fs::{File, read_dir};
use std::io::{BufRead, BufReader};

use rayon::prelude::{IntoParallelRefIterator, ParallelBridge, ParallelIterator};
use std::path::PathBuf;
use std::time::{Instant, Duration};

fn main() {
    // Iniciamos el cronometro.
    let start = Instant::now();

    // Abrimos la carpeta `/data`
    let result = read_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/data")).unwrap()
        .flatten()
        .map(|d| d.path())
        // Guardamos todas las rutas en memoria para que Rayon conozca el tamaño total.
        .collect::<Vec<PathBuf>>()
        // Dividimos los archivos entre los nucleos de la CPU.
        .par_iter()
        .flat_map(|path| {
            let file = File::open(path);
            let reader = BufReader::new(file.unwrap());
            // Tomamos el flujo de texto secuencial y lo mandamos a un grupo de hilos workers
            // (thread pool) de Rayon     
            reader.lines().par_bridge()
        })
        // Cada hilo toma una linea, la divide en palabras y crea su propio HashMap 
        // (mini-diccionario) temporal.
        .map(|l| {
            let line = l.unwrap();
            let words = line.split(' ');
            thread::sleep(Duration::from_millis(100));
            let mut counts = HashMap::new();
            words.for_each(|w| *counts.entry(w.to_string()).or_insert(0) += 1);
            counts
        })
        // `.reduce` esta diseñado para fusionar el trabajo de multiples hilos en paralelo.
        // Se fusionaran dos diccionarios.
        // - `acc`: Es el diccionario acumulado por el Hilo A.
        // - `words`: Es el diccionario acumulado por el Hilo B (ya no es una sola linea).
        .reduce(|| HashMap::new(), |mut acc, words| {
            // Iteramos sobre todas las palabras encontradas por el Hilo B y las insertamos 
            // sumamos al diccionario del Hilo A.
            words.iter().for_each(|(k, v)| {
                *acc.entry(k.clone()).or_insert(0) += v;
            });
            acc
        });

    println!("Tiempo total: {:?}", start.elapsed());
    println!("{:?}", result);
}

// ¿Por que necesitamos repartir tareas dos veces (por archivo y por linea)?
// Se debe a un problemma llamado Desbalanceo de Carga. 
// Supongamos el siguiente escenario, donde tenemos 4 hilos en nuestro procesador y 
// se tiene que procesar 4 archivos:
// - A.txt (pesa 10 Gigabytes)
// - B.txt (pesa 1 Megabyte)
// - C.txt (pesa 1 Megabyte)
// - D.txt (pesa 1 Megabyte)
//
// Si solo usamos par_iter para los archivos:
// - El Hilo 1 toma A.txt.
// - El Hilo 2 toma B.txt.
// - El Hilo 3 toma C.txt.
// - El Hilo 4 toma D.txt.
//
// Los Hilos 2, 3 y 4 van a terminar su trabajo en una fraccion de segundo. ¿Que hacen despues? Nada. 
// Se quedan cruzados de brazos mirando como el Hilo 1 procesa el archivo de 10 Gigabytes. 
// Donde desperdiciamos el 75% del procesador.
//
// Al agregar `par_bridge()` adentro, cambiamos como se comporta el codigo:
// El Hilo 1 toma el archivo gigante A.txt, pero en lugar de leerlo en secreto, usa par_bridge para 
// desarmarlo en lineas y dejarlas disponibles sobre la mesa.
// Los Hilos 2, 3 y 4 terminan sus archivos pequeños casi al instante.
// Como ven que no hay mas archivos enteros, hacen un Work-Stealing (Robo de Trabajo): se meten a la mesa 
// del Hilo 1 y empiezan a agarrar las lineas sueltas generadas por par_bridge.
// Ahora, los 4 hilos estan procesando el archivo gigante juntos.