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

// Este codigo implementa una variante del problema de "Lectores-Escritores" diseñada para otorgarle
// prioridad absoluta a los escritores. En un escenario estandar, un flujo incesante de lectores puede 
// provocar que el contador de lectores nunca llegue a cero, dejando a los escritores esperando infinitamente
// STARVATION. Para solucionar esto, el código introduce un registro de la "intención" de escribir. 
// Apenas un escritor anuncia que quiere escribir, se le bloquea el acceso a cualquier lector nuevo.
// Los lectores que ya estaban adentro pueden terminar su tarea, y cuando el último sale, el escritor 
// tiene el paso garantizado. Esta solución traslada el problema de STARVATION a los lectores en caso 
// de que haya demasiados escritores activos simultáneamente.

// Define el estado del sistema.
#[derive(Debug)]
struct ReadWrite {
    // Lleva la cuenta de quienes estan leyendo.
    readers: i32,
    // Indica si alguien esta escribiendo.
    writing: bool,
    // Lleve la cuenta de cuantos escritores estan trabajando o esperando su turbo.
    writers: i32,
}

// Envuelve el dato entero de un UnsafeCell, lo que permite modificar el dato interno saltandose las 
// protecciones por defecto de Rust.
struct DataHolder {
    data: UnsafeCell<i32>
}
unsafe impl Sync for DataHolder {}


fn main() {
    // Definimos la cantidad de lectores y escritores a crear.
    const READERS: i32 = 5;
    const WRITERS: i32 = 2;
    
    // Inicializamos las tres variables de estado en 0 o falso y las protegemos con un `Mutex`. Ademas,
    // les asocuamos una `CondVar` para las pausas y envolvemos todo en un `Arc`.
    let pair = Arc::new((Mutex::new(ReadWrite { readers: 0, writing: false, writers: 0 }), Condvar::new()));
    
    // Inicializamos el dato con el numero 42 y lo envolvemos en un `Arc`.
    let data = Arc::new(DataHolder { data: UnsafeCell::new(42) } );
    
    // --- HILOS LECTORES ---
    // Iteramos de 0 a 4 para crear 5 hilos, asignandole el indice actual a la variable `me` (ID del lector).
    let readers: Vec<JoinHandle<()>> = (0..READERS)
        .map(|me| {
            let pair_reader = pair.clone();
            let data_reader = data.clone();
            
            thread::spawn(move || loop {
                let (lock, cvar) = &*pair_reader;

                {
                    // Bloquea el `Mutex` y entra en la comprobacion del `CondVar` para ver si debe dormirse.
                    let mut _guard = cvar.wait_while(lock.lock().unwrap(), |state| {
                        println!("[Lector {}] Chequeando {:?}", me, state);
                        // La REGLA DE PRIORIDAD:
                        // - El lector devuelve true (irse a dormir) si un escritor esta escribiendo,
                        //   o si cualquier escritor anuncio que quiere escribir.
                        state.writing || state.writers > 0
                    }).unwrap();

                    // Si pasa la condicion, retiene el candado.
                    _guard.readers += 1;
                }

                // Accede a la memoria protegida del `UnsafeCell` para leer el numero e imprimirlo.
                unsafe {
                    println!("[Lector {:?}] Leyendo {}", me, data_reader.data.get().read());
                }
                // Simula la lectura.
                thread::sleep(Duration::from_millis(thread_rng().gen_range(500, 1500)));
                println!("[Lector {:?}] Terminé", me);

                // Adquiere el candado un instante para restarse el conteo de lectores activos, indicando que salio.
                lock.lock().unwrap().readers -= 1;
                
                // Envia una señal para despertar a todos los hilos dormidos para que revisen si ya pueden pasar.
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

                // Si quitamos este sleep, los escritores llegarán tan rápido a la puerta que 
                // 'writers', nunca será 0, matando de STARVATION a los lectores.
                thread::sleep(Duration::from_millis(thread_rng().gen_range(500, 1500)));
                
                // Antes de evaluar si puede entrar, anuncia que llegue a la fila
                // Toma el `Mutex` rápido, suma 1 a 'writers' y lo suelta
                // A partir de este microsegundo, NINGUN LECTOR NUEVO PODRA ENTAR.
                lock.lock().unwrap().writers += 1;
                {
                    // Esperamos a que se vacien los lectores viejos.
                    let mut _guard = cvar.wait_while(lock.lock().unwrap(), |state| {
                        println!("[Escritor {}] Chequeando {:?}", me, state);
                        
                        // Espera si alguien más está escribiendo, o si todavia quedan lectores
                        // adentro que entraron antes que diera el aviso.
                        state.writing || state.readers > 0
                    }).unwrap();
                    
                    // La sala está vacia y es su turno.
                    _guard.writing = true;
                }
                
                // Zona de escritura exclusiva.
                unsafe {
                    println!("[Escritor {:?}] Escribiendo", me);
                    data_writer.data.get().write(me);
                }
                thread::sleep(Duration::from_millis(thread_rng().gen_range(500, 1500)));
                println!("[Escritor {:?}] Terminé", me);
                
                // Sale y baja las banderas.
                let mut state = lock.lock().unwrap();
                state.writing = false; // Ya no se está escribiendo.
                state.writers -= 1; // Disminuyo la cantidad de escritores.
                
                // Aviso para despertar a los demás:
                // - Si witers llego a 0, los lectores por fin podrán entrar.
                // - Si sigue siendo > 0, otro escritor entrará.
                cvar.notify_all();
            })
        })
        .collect();

    // Recolección de hilos.
    let _:Vec<()> = readers.into_iter()
        .chain(writers.into_iter())
        .flat_map(|x| x.join())
        .collect();

}