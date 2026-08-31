use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::path::PathBuf; 
use std::time::{Instant};
use std::fs::{File, read_dir};
use rayon::iter::{IntoParallelRefIterator, ParallelBridge, ParallelIterator}; 

fn main() {
    rayon::ThreadPoolBuilder::new()
    .num_threads(8) 
    .build_global()
    .unwrap();

    let start = Instant::now();
    
    let result = read_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/data")).unwrap()
        // Extraemos todos los archivos validos.
        .flatten()
        // Extraemos los path de los archivos que se encuentran en /data
        .map(|d| d.path())
        .collect::<Vec<PathBuf>>()
        .par_iter()
        // Extraemos la ruta del archivo.
        .flat_map(|path|{
            let file = File::open(path);
            let reader = BufReader::new(file.unwrap());
            reader.lines().par_bridge()
        })
        .map(|l|{
            let line = l.unwrap();
            let words = line.split(' ');
            let mut count = HashMap::new();
            words.for_each(|w| *count.entry(w.to_string()).or_insert(0) +=1);
            count       
        })
        .reduce(|| HashMap::new(), |mut acc, words|{
            words.iter().for_each(|(k,v)| *acc.entry(k.clone()).or_insert(0) += v);
            acc            
        });
    
        println!("Tiempo de procesamiento: {:?}",start.elapsed());
        println!("{:?}", result);
} 

// Tiempo de procesamiento con 1 hilos: 905.3130006s
// Tiempo de procesamiento con 2 hilos: 486.1032113s
// Tiempo de procesamiento con 3 hilos: 374.9436406s
// Tiempo de procesamiento con 4 hilos: 343.6749484s
// Tiempo de procesamiento con 8 hilos: 236.8647257s
