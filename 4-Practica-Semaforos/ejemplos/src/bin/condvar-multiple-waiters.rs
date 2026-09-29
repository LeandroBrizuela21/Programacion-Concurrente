extern crate rand;

use std::sync::{Arc, Mutex, Condvar};
use std::thread;
use std::time::Duration;
use std::thread::JoinHandle;

// En este codigo se implementa un Productor-Consumidor utilizando `CondVars` para coordinar varios hilos a la 
// vez. En este diseño, un unico hilo "productor" genera tareas o datos y los coloca en un bufer central 
// (vector compartido) mietras que multiples hilos "consumidores" se quedan dormindos esperando a que haya 
// elementos disponibles.
// El codigo demuestra como sincronizar el acceso a esa lista compartida y como el productor puede emitir
// una señal general (`notify_all()`) para despertar a los consumidores y que estos compitan de forma segura
// para extraer los datos. 

fn main() {

    // Tenemos 4 consumidores y 1 productor.
    const N:i32 = 5;

    // Un `Mutex` (que protege el buffer/vector) y un `Condvar` (para avisar).
    // Los agrupamos en una tupla y los envolvemos en un `Arc` para compartirlos.
    let pair = Arc::new((Mutex::new(Vec::new()), Condvar::new()));

    // -- EL PRODUCTOR --
    let pair_clone = pair.clone();
    let producer = thread::spawn(move || {
        let mut produced:i32 = 0;
        loop {
            // Desempaqueta la tupla para usar el `lock` y el `cvar`.
            let (lock, cvar) = &*pair_clone;

            // Simula un computo intensivo para producir un producto.
            println!("[PRODUCER] doing expensive computation");
            thread::sleep(Duration::from_millis(1000));
            produced+=1;
            println!("[PRODUCER] done");

            // Bloquea el `Mutex` para poner el producto de forma segura.
            let mut buffer = lock.lock().unwrap();
            println!("[PRODUCER] got lock");
            buffer.push(produced);
            println!("[PRODUCER] notifying");

            // Despertamos a todos los hilos dormidos, anunciado que ya contamos
            // con productos para consumir.
            cvar.notify_all();
            
        }// Al salir del scope, el `Mutex` se libera automaticamente.
    });

    // -- LOS CONSUMIDORES -- 
    let consumers:Vec<JoinHandle<()>> = (1..N).map(|i| {
        let pair_clone_waiter = pair.clone();
        thread::spawn(move || {
            loop {
                let (lock, cvar) = &*pair_clone_waiter;

                let to_consume = {
                    // `wait_while` hace 3 cosas por detras:
                    // 1. Bloquea el `Mutex` y revisa la condición (¿El vector está vacio?)
                    // 2. Si está vacio (`true`), SUELTA EL MUTEX y pone a dormir este hilo.
                    // 3. Cuando el productor hace `notify`, este hilo se despierta.
                    //    - Vuelve a bloquear el `Mutex` automaticamente, y revisa la condición de nuevo.
                    let mut _guard = cvar.wait_while(lock.lock().unwrap(), |buffer| {
                        println!("[CONSUMER {}] checking condition {}", i, buffer.len());
                        buffer.is_empty() // "Mientras esté vacio sigo durmiendo"
                    }).unwrap();
                    
                    // Si llegamos aca, es porque el buffer NO está vacío y tenemos el `Mutex` bloqueado.
                    // Sacamos el último elemento.
                    _guard.pop().unwrap()
                }; // Soltamos el Mutex.

                println!("[CONSUMER {}] woke up - consuming {}", i, to_consume);
                thread::sleep(Duration::from_millis(1000));
                println!("[CONSUMER {}] done", i);

            }
        })
    }).collect();

    // Esperamos a que los hilos terminen.
    let _:Vec<()> = consumers.into_iter()
        .flat_map(|x| x.join())
        .collect();

    producer.join().unwrap();
}