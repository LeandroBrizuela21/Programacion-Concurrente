extern crate num_derive;
extern crate num_traits;
extern crate rand;

use std::cell::UnsafeCell;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use rand::{Rng, thread_rng};
use rand::seq::SliceRandom;

// ============= PROBLEMA LECTOR-ESCRITOR =============
// - Un estado se comparte entre varios procesos.
// - Algunos procesos necesitan actualizar dicho estado, mientras que otros solo necesitan leerlo.
// - Mientras que un proceso está leyendo el estado, otros pueden leerlo, pero ninguno modificarlo.
// - Mientras que un proceso está modificando el estado, ningun otro puede leerlo ni modificarlo. 


// Es el estado de sincronizacion central.
// - Mantiene un conteo de cuantos lectores activos hay en un momento dado y 
//   una bandera booleana para saber si un escritor tiene el control exclusivo.
#[derive(Debug)]
struct ReadWrite {
    readers: i32,
    writing: bool
}

// Envuelve el dato real (en este caso, un entero).
// `UnsafeCell` nos permite apagar las reglas de seguridad de Rust para este i32.
struct DataHolder {
    data: UnsafeCell<i32>
}

// Le prometemos al compilador que nosotros manejaremos la seguridad entre hilos.
unsafe impl Sync for DataHolder {}

fn main() {
    // Establecemos la cantidad de hilos y escritores a crear.
    const READERS: i32 = 5;
    const WRITERS: i32 = 2;

    // `Mutex` + `Condvar` para proteger  el estado de sincronización. No el dato.
    let pair = Arc::new((Mutex::new(ReadWrite { readers: 0, writing: false }), Condvar::new()));
    
    // Inicializamos el recurso que los hilos van a leer y modificar, estableciendo el numero 42 como inicio.    
    let data = Arc::new(DataHolder { data: UnsafeCell::new(42) } );

    // --- HILOS LECTORES ---
    // Iteramos para crear los hilos lectores, clonando las referencias compartidas de `pair` y `data`.
    let readers: Vec<JoinHandle<()>> = (0..READERS)
        .map(|me| {
            let pair_reader = pair.clone();
            let data_reader = data.clone();

            thread::spawn(move || loop {
                let (lock, cvar) = &*pair_reader;

                // Sacar esto para llegar a starvation del writer
                // thread::sleep(Duration::from_millis(thread_rng().gen_range(500, 1500)));
                
                // El  {} es crucial, define cuanto tiempo el lector sostiene el Mutex.
                {
                    // REGLA DEL LECTOR: Espera si alguien está escribiendo.
                    // - Se queda dormido unicamente si la `state.writing` es verdadero.
                    let mut _guard = cvar.wait_while(lock.lock().unwrap(), |state| {
                        println!("[Lector {}] Chequeando {:?}", me, state);
                        state.writing
                    }).unwrap();
                    
                    // En caso de que no haya un hilo escribiendo, anotamos que hay un lector más.
                    _guard.readers += 1;
                }

                // Este bloque permite al lector acceder a la memoria del `UnsafeCell` para leer el valor actual
                // e imprimirlo. 
                unsafe {
                    println!("[Lector {:?}] Leyendo {}", me, data_reader.data.get().read());
                }
                // Simula tiempo de lectura.
                thread::sleep(Duration::from_millis(thread_rng().gen_range(500, 1500)));
                println!("[Lector {:?}] Terminé", me);

                // Vuelve a tomar el candado para avisar que ya ha leido y por ende se va.
                lock.lock().unwrap().readers -= 1;
                
                // Avisa a todos que se va.
                cvar.notify_all();
            })
        })
        .collect();

    // --- HILOS ESCRITORES ---
        let writers: Vec<JoinHandle<()>> = (0..WRITERS)
        .map(|me| {
            let pair_writer = pair.clone();
            let data_writer = data.clone();

            thread::spawn(move || loop {
                let (lock, cvar) = &*pair_writer;
                
                // El  {} es crucial, define cuanto tiempo el lector sostiene el `Mutex`.
                {
                    // REGLA DEL ESCRITOR: Espera si alguien está escribiendo o si hay al menos un lector
                    // Exige exclusividad absoluta.
                    let mut _guard = cvar.wait_while(lock.lock().unwrap(), |state| {
                        println!("[Escritor {}] Chequeando {:?}", me, state);
                        state.writing || state.readers > 0
                    }).unwrap();
                    
                    // En caso de que no haya ningun hilo escribiendo o leyendo, anotamos que estamos escribiendo.
                    _guard.writing = true;
                }

                // Este bloque permite al escritor acceder a la memoria del `UnsafeCell` para escribir el valor actual
                // e imprimirlo.                 
                unsafe {
                    println!("[Escritor {:?}] Escribiendo", me);
                    
                    // Escribimos el ID
                    data_writer.data.get().write(me);
                }
                thread::sleep(Duration::from_millis(thread_rng().gen_range(500, 1500)));
                println!("[Escritor {:?}] Terminé", me);

                // Toma el `Mutex`, regresa la bandera wrinting = false y emite cvar.notify_all() para 
                // despertar a los lectores o escritores encolados.
                lock.lock().unwrap().writing = false;
                cvar.notify_all();
            })
        })
        .collect();

    // El iterador `chain` une los vectores de los hilos lectores y escritores aplicando `join` a todos para 
    // mantener el pograma en ejecucion perpetua en este punto de bucles infinitos.
    let _:Vec<()> = readers.into_iter()
        .chain(writers.into_iter())
        .flat_map(|x| x.join())
        .collect();

}