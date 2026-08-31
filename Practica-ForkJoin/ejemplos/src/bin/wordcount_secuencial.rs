use std::collections::HashMap;
use std::{env, thread};
use std::fs::{File, read_dir};
use std::io::{BufRead, BufReader}; 
use std::time::{Instant, Duration};

fn main() {
    // Iniciamos el cronometro.
    let start = Instant::now();

    // Abrimos la carpeta `/data`
    let result = read_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/data")).unwrap()
        // Nos quedamos solo con la ruta de cada archivo encontrado.
        .map(|d| d.unwrap().path())
        // Unifica todo en un solo flujo continuo de lineas que van llegando una atras de otra, sin
        // importar de que archivo vengan.
        .flat_map(|path| { 
            // Abrimos el archivo.
            let file = File::open(path);
            // Leemos en bloques para optimizar la memoria.
            let reader = BufReader::new(file.unwrap());
            // Devolvemos un iterador de sus lineas.
            reader.lines()
        })
        // Procesamos cada linea y hacemos el conteo individual.
        .map(|l| {
            let line = l.unwrap();
            // Separamos la lines en palabras por los espacios.
            let words = line.split(' ');
            // Mandamos a dormir el thread para simular una operacion de "trabajo pesado".
            thread::sleep(Duration::from_millis(100));
            // Creamos un HashMap local solo para esta linea.
            let mut counts = HashMap::new();
            // Contamos cuantas veces aparece cada palabra en esta linea. Donde `.entry()` busca 
            // la palabra, en caso de que no exista la crea con valor 0.
            // El asterisco (*) "desempaqueta" ese puntero para acceder al número real en la memoria.
            // Una vez que tenemos el número físico, el += 1 le suma uno, 
            // actualizando el conteo en el diccionario original.
            words.for_each(|w| *counts.entry(w.to_string()).or_insert(0) += 1);
            counts
        })
        // Toma los resultados del paso anterior y los acumula.
        // `.fold` toma todos los mini-diccionarios de cada linea y los va a fusionar en uno solo.
        // - `acc`: Es la caja grande que almacenara la cuenta total.
        // - `words`: Es la caja pequeña (el HashMap de una sola linea) que nos entrego el `.map()` anterior.
        .fold(HashMap::new(), |mut acc, words| {
            // Iteramos sobre cada palabra y su cantidad de la caja pequeña y lo vamos almacenendo en el `acc`.
            words.iter().for_each(|(k, v)| *acc.entry(k.clone()).or_insert(0) += v);
            acc
        });
    println!("Tiempo total: {:?}", start.elapsed());


    println!("{:?}", result);
}