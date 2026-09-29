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

// Este codigo implementa una versión modificada del problema de "Lectores-Escritores" diseñada para evitar
// el problema de STARVATION de los escritores. En la versión anterior, si un flujo constante de lectores
// seguía llegando, un escritor podía quedarse esperando infinitamente porque el contador de lectores nunca 
// llegaba a cero. Para solucionar esto, el código introduce un sistema de turnos secuenciales 
// (como sacar un número en un banco) mediante las variables queue y next.
// De esta forma, las peticiones se atienden en el orden exacto en el que llegan; 
// si un escritor saca su número, cualquier lector que llegue después deberá esperar su turno, 
//garantizando un acceso "justo" (fair) para todos los hilos.




// --- ESTADO (Maquina de tickets) ---
#[derive(Debug)]
struct ReadWrite {
    // Cuantos estan leyendo en este momento.
    readers: i32,
    // ¿Hay alguien escribiendo en este momento?
    writing: bool,
    // Guarda el número del que se esta ejecutando en ese instante.
    queue: u32,
    // Número que sacará la proxima persona que llegue.
    next: u32,
}

//--- EL DATO COMPARTIDO ---
struct DataHolder {
    // Permite lectura/escritura saltándose los seguros estrictos de Rust
    data: UnsafeCell<u32>
}

unsafe impl Sync for DataHolder {}


fn main() {
    const READERS: i32 = 5;
    const WRITERS: i32 = 2;

    // Mutex protegue solo la máquina de tickets, no el dato en sí.
    let pair = Arc::new((Mutex::new(ReadWrite { readers: 0, writing: false, queue: 0, next: 0 }), Condvar::new()));
    let data = Arc::new(DataHolder { data: UnsafeCell::new(42) } );

    // --- HILOS LECTORES ---
    let readers: Vec<JoinHandle<()>> = (0..READERS)
        .map(|me| {
            let pair_reader = pair.clone();
            let data_reader = data.clone();

            thread::spawn(move || loop {
                let (lock, cvar) = &*pair_reader;
                
                // LLEGAR Y SACAR NÚMERO
                // Tomamos el lock de forma exclusiva.
                let mut _guard = lock.lock().unwrap();
                let id = _guard.next;
                
                // Tomamos el siguiente turno para hacer atendidos
                _guard.next += 1;
                
                {
                    let mut _guard = cvar.wait_while(_guard, |state| {
                        println!("[Lector {}] Chequeando {:?}", id, state);
                        
                        // REGLA: Espera si alguien está escribiendo, o si su ticket aún no aparece.
                        state.writing || id != state.queue
                    }).unwrap();
                    
                    // Si es el turno correspondiente, aumentamos la cantidad de lectores.
                    _guard.readers += 1;
                    
                    // Como nosotros solo vamos a leer, no nos importa si entran otros lectores.
                    // Así que, antes de irnos a leer, APRETAMOS EL BOTÓN para llamar al siguiente 
                    // número de la fila.
                    _guard.queue += 1;
                } // Suela la máquina de tickets para que otros saquen npumero o avancen.

                // ZONA DE LECTURA (En paralelo con otros lecotres)
                unsafe {
                    println!("[Lector {}] Leyendo {}", id, data_reader.data.get().read());
                }
                thread::sleep(Duration::from_millis(thread_rng().gen_range(500, 1500)));
                println!("[Lector {}] Terminé", id);

                // Terminamos la operación.
                let mut state = lock.lock().unwrap();
                // Disminuimos la cantidad de lectores.
                state.readers -= 1;
                // Despertamos a todos, ya que es vital por si un escritor estaba esperando que salgan lectores.
                cvar.notify_all();
            })
        })
        .collect();

    let writers: Vec<JoinHandle<()>> = (0..WRITERS)
        .map(|me| {
            let pair_writer = pair.clone();
            let data_writer = data.clone();

            thread::spawn(move || loop {
                let (lock, cvar) = &*pair_writer;

                // Al igual que los lectores, el escritor obtiene su boleto al llegar.
                let mut _guard = lock.lock().unwrap();
                let id = _guard.next;
                _guard.next += 1;
                {
                    // El escritor se queda esperando si otro escritor esta trabajando, si hay lectores activos
                    // o si no es su turno. 
                    // - Gracias a esto, por ejemplo, si el escritor es el número 3 en la fila, 
                    //   los lectores que sacaron los números 4 y 5 no podrán adelantarse y tendran
                    //   que esperar a que el escritor termine.
                    let mut _guard = cvar.wait_while(_guard, |state| {
                        println!("[Escritor {}] Chequeando {:?}", id, state);
                        state.writing || state.readers > 0 || id != state.queue
                    }).unwrap();
                    _guard.writing = true;
                }

                // El escritor modifica el estado compartido para asignandole su ID.
                unsafe {
                    println!("[Escritor {}] Escribiendo", id);
                    data_writer.data.get().write(id);
                }
                // Simula la escritura.
                thread::sleep(Duration::from_millis(thread_rng().gen_range(500, 1500)));
                println!("[Escritor {}] Terminé", id);
                
                // Al terminar, el escritor apaga su bandera de escritura y avanza el turno general de la 
                // fila para cederle el paso al siguiente hilo.
                // Luego emite `var.notify_all()` para despertar al siguiente hilo.
                let mut state = lock.lock().unwrap();
                state.writing = false;
                state.queue += 1;
                cvar.notify_all();
            })
        })
        .collect();

    let _:Vec<()> = readers.into_iter()
        .chain(writers.into_iter())
        .flat_map(|x| x.join())
        .collect();

}

