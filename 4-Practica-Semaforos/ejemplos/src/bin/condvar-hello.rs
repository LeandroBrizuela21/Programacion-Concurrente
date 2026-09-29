use std::sync::{Arc, Mutex, Condvar};
use std::thread;
use std::time::Duration;

// `CondVars` es un mecanismo utilizado para bloquear un hilo hasta que un dato especifico cambie de estado
// a diferencia de un `Mutex` simple que solo protege el acceso a datos, la combinacion de un `Mutex` con un 
// `CondVar` permite que un hilo se vaya a dormir y sea despertado activamente por otro hilo cuando ocurre
// un evento especifico (en este caso, que una variable booleana cambie de true a false).

// Tambien, en este codigo demuestra de como protegerse contra "despertares espurios" (cuando un hilo recibe
// una señal de despertar pero la condicion logica aun no se ha cumplido) mediante el metodo `wait_while`.

fn main() {
    // El dato protegido es un simple booleano, donde 'true' significa "estoy trabajando, espera".
    let pair = Arc::new((Mutex::new(true), Condvar::new()));

    // --- EL HILO TRABAJADOR ---
    let pair_clone = pair.clone();
    thread::spawn(move || {
        // Desempaquetamos la tupla para obtener referencias separadas y comodas
        // hacia el `Mutex` (lock) y la Variable de Condicion (cvar).
        let (lock, cvar) = &*pair_clone;

        // Hacemos sonar la alarma antes de hacer el trabajo.
        // - Enviamos una señal de despertar a cualquier hilo que este escuchando.
        // Esto simula un "Spurious Wakeup" (un despertar falso).
        cvar.notify_all(); 

        println!("[awaited] doing expensive computation");
        thread::sleep(Duration::from_millis(1000));
        println!("[awaited] done");

        // El hilo worker toma control del `Mutex`, cambiandolo a `false` 
        // para indicar que el trabajo termino.
        let mut pending = lock.lock().unwrap();
        println!("[awaited] got lock");
        // Cambiamos el estado a "ya termine".
        *pending = false;

        // Enviamos el aviso verdadero.
        // Vuelve a emitir la señal para despertar a los hilos dormidos, 
        // ahora si con la condicion correcta ya modificada.
        println!("[awaited] notifying");
        cvar.notify_all();
    });


    let (lock, cvar) = &*pair;

    // Usar `wait_while` nos protege del "Spurious Wakeup" de arriba.
    // Cuando el trabajador hace el primer `notify_all()` falso, este hilo se despierta revisa el
    // valos de `*pending`, ve que sigue siendo `true` (porque el trabajo no ha terminado), y se 
    // vuelve a dormir automaticamente.
    let _guard = cvar.wait_while(lock.lock().unwrap(), |pending| {
        println!("[waiter] checking condition {}", *pending);
        *pending
    }).unwrap();

    println!("[waiter] current mutex content {}", *_guard);
    println!("[waiter] done");

}
