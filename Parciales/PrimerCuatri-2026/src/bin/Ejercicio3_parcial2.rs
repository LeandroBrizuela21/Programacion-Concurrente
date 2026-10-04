use std::sync::{Arc, Mutex};
use std::thread;
use rand::Rng;
use std::time::Duration;
const N: usize = 10;

// PROBLEMA: Cuando la condicion se cumple, procesa el dato, libera el lock y vuelve a iterar de inmediato
//           -> El hilo solicita muchas veces el lock por segundo (consumo ineficiente de recursos).
fn main() {
    let buffer = Arc::new(Mutex::new(Vec::<u32>::with_capacity(N)));
    let buffer_local = buffer.clone();
    let handle = thread::spawn(move || {
        loop {
            let mut buf = buffer_local.lock().unwrap();
            if buf.len() < N {
                buf.push(rand::thread_rng().r#gen());
            } else {
                // PROBLEMA: Se duerme con la guarda del lock.
                thread::sleep(Duration::from_secs(1));
                drop(buf);
            }
        }
    });

    loop {
        let mut buf = buffer.lock().unwrap();
        if !buf.is_empty() { 
            println!("{}", buf.pop().unwrap());
        } else {
            // PROBLEMA: Se duerme con la guarda del lock.
            thread::sleep(Duration::from_secs(1));
            drop(buf);
        }
    }

    handle.join().unwrap();
}

//a) PROPUESTA DE SOLUCION: Usar CondVar para coordinar el acceso al estado compartido mediante espera pasiva.
//   - Suspende al hilo y libera automaticamente el `Mutex` cuando la condición no se cumple.  
//   - El hilo solo se despierta cuando otro hilo le notifica explicitamente que el estado del buffer cambio,
//     evitando consultas periodicas.

// b) Se usa para forzar la liberacion explicita del lock antes de finalizar la iteracion del loop. Esto 
//    permite que otro hilo pueda adquirir el Mutex. 
//    - Seria mejor que el `drop` este antes de que el hilo se mande a dormir  para evitar que 
//      se duerma con la guarda del lock.  