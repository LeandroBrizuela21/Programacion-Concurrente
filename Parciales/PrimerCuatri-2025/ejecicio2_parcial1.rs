fn main() {
    const N: usize = 5;
    
    let producers_waiting = Arc::new(Semaphore::new(0));
    let consumer_done = Arc::new(Semaphore::new(0));
    let data = Arc::new(Mutex::new(None));

    let producers_waiting_clone = producers_waiting.clone();
    let consumer_done_clone = consumer_done.clone();

    let consumer = thread::spawn(move || loop {
        println!("[Consumer] Sleeping...");
        producers_waiting_clone.acquire();
        
        println!("Doing my job with data {:?}", data.lock().expect("can read"));
        thread::sleep(Duration::from_secs(2));
        
        consumer_done_clone.release();
        println!("Job finished");
    });

    let producers: Vec<_> = (0..N).map(|id| {
        let producers_waiting_clone = producers_waiting.clone();
        let consumer_done_clone = consumer_done.clone();
            
        thread::spawn(move || loop {
            let delay = rand::thread_rng().gen_range(3000..7000);
            thread::sleep(Duration::from_millis(delay));
            println!("[Producer {}] Arrived", id);
            producers_waiting_clone.release();
            
            *data.lock().expect("can't set data") = Some(id);
            
            println!("[Producer {}] waiting to be consumed", id);
            consumer_done_clone.acquire();
                
            println!("[Producer {}] Leaving", id);
            })
    }).collect();

    consumer.join().unwrap();
    for p in producers {
        p.join().unwrap();
    }
}

// Problema Modelado: Multiples Productores y un Unico Consumidor.
// 
// Problemas en la Implementacion Detectados:
// - `producers_waiting_clone.release()` se realiza antes de que el productor cambie el valor de `data`, lo
//   que puede ociacionar que el consumidor lea el valor anterior al que va a poner el productor.
// - Se puede pisar los `id` que se escriben en `data`, antes que sean consumidos.
//   - Si el hilo A escribe su `id` en `data` y apenas suelta el lock, el hilo B podra acceder al lock y escribir su `id` antes de que el hilo consumidor
//     consuma el valor de A. En consecuencia el hilo procesa otro hilo.
// - Como se hace uso de semaforos, no podemos saber con certeza que producto consumio el hilo consumidor, por ende un hilo productor se puede ir aunque su 
//   no producto no se haya procesado. Y esto se debe a que los release() solamente suman +1, indicando que se pueda consumir, no define quien.
// 
 
// Mejoras Propuestas: 
// - Podemos hacer uso de los channels mpsc.
//   - Se puede hacer uso de dos canales:
//     - Canal 1: En donde su `rx` lo tenga el `Consumidor` para recibir los datos de los `Productores` que tendran el `tx` correspondiente.
//     - Canal 2: En donde su `tx` se lo enviara el `Productor` al `Consumidor` (mediente el `tx del Canal 1`) para que pueda enviarle 
//                notificarle que su dato ya fue procesado y el `rx` lo tendra cada `Productor` para recibir la notificacion.
// - No haria falta un estado mutable compartido ya que cada productor mandaria su dato producido por el canal. Estos serian encolandos
//   y consumidos secuencialmente por el consumidor.




// Alternativa manteniendo semaforos:
let buffer_vacio = Arc::new(Semaphore::new(1));       // Garantiza exclusion mutua y buffer de cap 1
let dato_listo = Arc::new(Semaphore::new(0));         // Avisa al consumidor que hay un dato
let consumo_finalizado = Arc::new(Semaphore::new(0)); // Avisa al productor que ya se leyó

// Productor:
buffer_vacio.acquire();                               // 1. Pide acceso exclusivo al buffer
*data.lock().unwrap() = Some(id);                     // 2. Escribe el dato PRIMERO
dato_listo.release();                                 // 3. Avisa al consumidor
consumo_finalizado.acquire();                         // 4. Espera a que el consumidor termine
buffer_vacio.release();                               // 5. Libera el buffer para el siguiente productor

// Consumidor:
dato_listo.acquire();                                 // 1. Espera a que haya un dato
let val = data.lock().unwrap().take();                // 2. Lee y remueve el dato
consumo_finalizado.release();                         // 3. Avisa al productor que termino
 