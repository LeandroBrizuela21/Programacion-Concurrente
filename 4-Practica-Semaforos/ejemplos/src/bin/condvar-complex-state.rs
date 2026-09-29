extern crate rand;

use std::sync::{Arc, Mutex, Condvar};
use std::thread;
use std::time::Duration;
use rand::{thread_rng, Rng};

// En este codigo se utiliza la `CondVars` para gestionar un estado de sincronizacion mas complejo mediante
// una estructura de datos personalizada (`struct`), en lugar de usar una variable simple.
// En este escenario el hilo principal necesita coordinar y esperar a que dos tareas asincronas e 
// independientes finalicen. Para lograrlo, agrupa multiples banderas de estado dentro de un unico `Mutex`
// y utiliza un solo `CondVar`. Cuando cualquiera de los hilos trabajadores termina su tarea, modifica su 
// bandera correspondiente y despierta al hilo principal, el cual vuelve a dormir hasta verificar que todas
// las condiciones (ambas banderas en `false` se hayan cumplido). 

fn main() {
    // 1) ====== ESTADO COMPARTIDO ======
    // Agrupamos las tareas en un `struct` para saber exactametne qué hilo ya termino.
    #[derive(Debug)]
    struct State {
        pending_first: bool,
        pending_second: bool
    }
    
    // Creamos el `Mutex` protegiendo nuestro struct (ambas tareas inician en `true`).
    // Lo juntamos con un `Condvar` en una tupla, y lo envolvemos en un `Arc` para compartirlo.
    let pair = Arc::new((Mutex::new(State { pending_first: true, pending_second: true} ), Condvar::new()));

    // ====== HILO TRABAJADOR 1 ======
    let pair_clone = pair.clone();
    thread::spawn(move || {
        let (lock, cvar) = &*pair_clone;

        // Simulamos un tabajo que toma un tiempo aleatorio.
        println!("[awaited 1] doing expensive computation");
        thread::sleep(Duration::from_millis(thread_rng().gen_range(500, 1500)));
        println!("[awaited 1] done");
        
        // El trabajo terminó. Pedimos el candado para actualizar el estado.
        let mut state = lock.lock().unwrap();
        println!("[awaited 1] got lock");
        
        // Marcamos ÚNICAMENTE nuestra tarea como completada (`false`).
        state.pending_first = false;

        // Despertamos al hilo principal para que revise si ya puede avanzar.
        println!("[awaited 1] notifying");
        cvar.notify_all();
    
    }); // <- Liberamos el Mutex.

    // ====== HILO TRABAJADOR 2 ======
    let pair_clone2 = pair.clone();
    thread::spawn(move || {
        let (lock, cvar) = &*pair_clone2;

        // Simulamos un tabajo que toma un tiempo aleatorio.
        println!("[awaited 2] doing expensive computation");
        thread::sleep(Duration::from_millis(thread_rng().gen_range(500, 1500)));
        println!("[awaited 2] done");
        
        // Pedimos el candado. Si el Hilo 1 lo está usando juesto ahora, el Hilo 2 esperará.
        let mut state = lock.lock().unwrap();
        println!("[awaited 2] got lock");
        
        // Marcamos ÚNICAMENTE nuestra tarea como completada (`false`).
        state.pending_second = false;
        
        // Despertamos al hilo principal para que revise si ya puede avanzar.
        println!("[awaited 2] notifying");
        cvar.notify_all();
    });

    //====== HILO PRINCIAPAL ======
    let (lock, cvar) = &*pair;

    // El hilo principal inicia su espera segura. `wait_while` evalúa la condición: 
    // - Si da `True`, se va a dormir.
    // - Si da `False`, avanza.
    let _guard = cvar.wait_while(lock.lock().unwrap(), |pending| {
        println!("[waiter] checking condition {:?}", *pending);
        // El hilo principal seguira esperando mientras que la primera tarea o la segunda tarea sigan siendo
        // verdaderas (es decir, mietras alguna siga estando pendiente).
        pending.pending_first || pending.pending_second
    }).unwrap();
    // Se retiene el candado y permitimos que el programa finalice exitosamente.
    println!("[waiter] done");

}