extern crate rand;
extern crate std_semaphore;

use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

use std_semaphore::Semaphore;
use rand::{thread_rng, Rng};

// ============= PROBLEMA DEL BARBERO =============
// Una barberia tiene una sala de espera con sillas.
// - Si la barberia esta vacia, el barbero se pone a dormir.
// - Si un cliente entra y el barbero esta durmiendo, lo despierta.
// - Si el barbero esta atendiendo, se sienta en una de las sillas y espera su turno.
// - El cliente espera sentado hasta que que le toque su turno para cortarse el pelo.

fn main() {
    // Definimos la cantidad de cleintes que vamos a simular.
    const N: usize = 5;

    //============= Creación de Variables Compartidas =============
    // Inicializamos todos los semáforos en 0. 
    // Un semáforo en 0 significa que:
    // El primer hilo que intente hacer un 'acquiere()' (esperar) se quedara bloqueado.
    // hasta que otro hilo haga un 'release()' (liberar/avisar). 

    
    // Semáforo para contar cuántos clientes están esperando en la sala.
    // - Los clientes lo usaran para avisarle al barbero que estan esperando.
    let customer_waiting = Arc::new(Semaphore::new(0));

    // Semáforo para que el barbero avise al cliente: "Ya estoy junto a la silla, puedes sentarte".
    let barber_ready = Arc::new(Semaphore::new(0));
    
    // Semáforo para que el barbero avise: "Ya terminé de cortarte, levántate".
    let haircut_done = Arc::new(Semaphore::new(0));
    
    // Mutex que guarda el ID del cliente actual. Empieza en -1 (silla vacia).
    // Usamos Mutex porque queremos leer y modificar este número de forma exclusiva. 
    let current_customer = Arc::new(Mutex::new(-1));

    // Semáforo para que el cliente avise: "Ya me senté y anote mi ID, puedes empezar".
    let current_customer_set = Arc::new(Semaphore::new(0));

    //============= Preparación para el Hilo del Barbero ============= 
    // Para que el hilo del barbero pueda usar estas variables, necesitamos darle sus
    // propias referencias. 
    // - '.clone()' en un 'Arc' no copia los datos, solo aumenta el contador de referencias
    //   hacias los mismos semáforos en memoria. 
    let customer_waiting_barber = customer_waiting.clone();
    let barber_ready_barber = barber_ready.clone();
    let haircut_done_barber = haircut_done.clone();
    let current_customer_barber = current_customer.clone();
    let current_customer_set_barber = current_customer_set.clone();
    let barber = thread::spawn(move || loop {
        println!("[Barbero] Esperando cliente");
        
        // 1) EL BARBERO DUERME
        // El hilo del barbero se bloquea (se duerme) en el semaforo, debido al 0, hasta que un cliente
        // avise que llego('.release()').
        customer_waiting_barber.acquire();

        // 2) EL BARBERO DESPIERTA Y SE PREPARA
        // El barbero libera el semaforo indicando que ya esta listo para cortar el pelo, debido a que un 
        // cliente lo desperto.
        barber_ready_barber.release();

        // 3) CONFIRMACIÓN DE ASIENTO
        // El barbero se bloquea nuevamente esperando a que el cliente se siente y registre su numero de ID.
        // Se queda bloqueado aca hasta que el cleinte haga 'release()' en este semaforo.
        current_customer_set_barber.acquire();
        
        // 4) A CORTAR EL PELO
        // El barbero toma temporalmente el candado Mutex para leer el ID del cliente actual.
        println!("[Barbero] Cortando pelo a {}", current_customer_barber.lock().unwrap());

        // Simula el corte de pelo.
        thread::sleep(Duration::from_secs(2));

        // 5) TERMINA EL CORTE
        // El barbero libera el semaforo para notificarle al cliente que el corte ha concluido.
        haircut_done_barber.release();
        println!("[Barbero] Terminé");
    });

    // Sirve para que cada hilo obtenga un número de cliente único
    let customer_id = Arc::new(Mutex::new(1));
    
    // Creamos 6 hilos de clientes iterando de 0 a 5 y guardamos sus manejadores en un vector.
    let customers: Vec<JoinHandle<()>> = (0..(N+1))
        .map(|_| {
            let barber_ready_customer = barber_ready.clone();
            let customer_waiting_customer = customer_waiting.clone();
            let haircut_done_customer = haircut_done.clone();
            let current_customer_id_customer = current_customer.clone();
            let current_customer_set_customer = current_customer_set.clone();
            let customer_id_customer = customer_id.clone();

            thread::spawn(move || loop {
                
                // 1) HACIENDO TIEMPO ANTES DE IR A LA BARBERIA
                // El cliente hace su vida durante 2 a 10 segundos antes de necesitar un corte.
                thread::sleep(Duration::from_secs(thread_rng().gen_range(2, 10)));

                
                // 2) SACANDO NÚMERO
                // Bloquea el Mutex, lee el número actual, le suma 1 para el siguiente y se guarda
                // su propio número en la variable local 'me'.
                let me = { 
                    let mut current = customer_id_customer.lock().unwrap();
                    *current += 1;
                    *current
                };
                
                println!("[Cliente {}] Entro a la barberia", me);
                
                // 3) AVISANDO QUE LLEGO
                // Incrementa el semáforo de la sala de espera.
                // El cliente libera el semaforo para avisar que esta en la sala de espera.
                // Si el barbero estaba bloqueado (durmiendo) en 'customer_waiting.acquire()'
                // este 'release()' lo despierta.
                customer_waiting_customer.release();

                println!("[Cliente {}] Esperando barbero", me);
                
                // 4) ESPERANDO SU TURNO EN LA SILLA
                // El cliente se bloquea hasta que el barbero indique que esta listo, es decir, que 
                // esperamos a que el barbero haga 'barber_ready.realice()'.
                // Si el barbero ya estaba listo pasa de largo.
                barber_ready_customer.acquire();

                println!("[Cliente {}] Me siento en la silla del barbero", me);
                
                // 5) REGISTRANDO SU LLEGADA A LA SILLA DE CORTE 
                // El cliente modifica la variable compartida de la silla asignandole su propio ID.
                *current_customer_id_customer.lock().unwrap() = me;
 
                // El cliente avisa que ya sento su ID en la silla para que el barbero comience a cortar.
                // Esto destraba el 'current_customer_set' del barbero.
                current_customer_set_customer.release();

                println!("[Cliente {}] Esperando a que me termine de cortar", me);

                // 6) RECIBIENDO EL CORTE
                // El cliente se bloquea esperando a que el barbero le notifique el fin del corte de pelo
                // mediente 'haircut_done.release()'
                haircut_done_customer.acquire();

                println!("[Cliente {}] Me terminaron de cortar", me);

            })
        })
        .collect();
    
    // El hilo principal espera a que todos los hilos de los clientes terminen iterando el vector
    // y aplicando `join()`.
    let _:Vec<()> = customers.into_iter()
        .flat_map(|x| x.join())
        .collect();
    
    // El hilo principal espera a que el hilo del barbero termine. 
    barber.join().unwrap();
}