extern crate std_semaphore;
extern crate rand;

use std_semaphore::Semaphore;
use std::thread;
use std::time::Duration;
use rand::{thread_rng, Rng};
use std::sync::Arc;
use std::thread::JoinHandle;

// ============= PROBLEMA DE LOS FILOSOFOS COMENSALES =============
// Cinco filósofos se sientan alrededor de una mesa y pasan su vida cenando y pensando.
// Cada filósofo tiene un plato de fideos y un palito chino a la izquierda de su plato.
// - Para comer los fideos son necesarios dos palitos.
// - Cada filósofo sólo puede tomar los que están a su izquierda y derecha.
// - Si cualquier filósofo toma un palito y el otro está ocupado, se quedará esperando, 
//  con el tenedor en la mano, hasta que pueda tomar el otro tenedor, para luego empezar 
//  a comer.

// Definimos la cantidad de filosofos (y de palitos) en la mesa.
const N:usize = 5;

fn main() {
    // 1) CREANDO LOS PALITOS (SEMÁFOROS)
    // Creamos un vector de cinco semaforos, inicializados en 1.
    // - Cada uno de estos semaforos se inicializa en 1.
    // - Cada uno de estos semaforos representa un palito (que el palito este en 1 significa que esta libre).
    // - Un semáforo en 1 actúa como un candado o Mutex, ya que solo UN filósofo lo puede usar.
    let chopsticks:Arc<Vec<Semaphore>> = Arc::new((0 .. N)
        .map(|_| Semaphore::new(1))
        .collect());

    // 2) SENTANDO A LOS FILÓSOFOS
    // Iteremos de 0 a 4 para crear cinco 5 hilos (filósofos), almacenando sus manejadores en un vector. 
    let philosophers:Vec<JoinHandle<()>> = (0 .. N)
        .map(|id| {
            // Creamos una referencia compartida de los palitos para el hilo actual.
            let chopsticks_local = chopsticks.clone();
            
            // Inicializamos el hilo ejecutando la funcion `philosopher` pasandole su ID y 
            // la referencia a los palitos.
            thread::spawn(move || philosopher(id, chopsticks_local))
        })
        .collect();

    // 3) ESPERADNO A QUE TERMINEN
    // El hilo principal espera indefinidamente a que terminen los hilos de los filosofos. 
    for philosopher in philosophers {
        let  _ = philosopher.join();
    }

}

fn philosopher(id: usize, chopsticks: Arc<Vec<Semaphore>>) {
    // Calculamos el indice del palito a la derecha del filosofo 
    let next = (id + 1) % N;
    let first_chopstick;
    let second_chopstick;

    // Para evitar el DEADLOCK rompemos la condicion de "espera circular" que bloquea el programa.
    // La estrategia consiste en cambiar el orden en que un unico filosofo levanta sus palitos.
    
    // ¿Por que esto soluciona el DEADLOCK?
    //
    // Supongamos que todos intentan comer exactamente al mismo tiempo y logran tomar su primer palito:
    // - El filosofo 0 toma el palito 0, el filosofo 1 toma el 1, el filosofo 2 toma el 2, 
    //   y el filosofo 3 toma el 3.
    // - Llega el turno del filosofo 4 de tomar su primer palito. 
    //   Por la regla invertida, intenta tomar el palito 0 (su derecha).
    // - Como el filosofo 0 ya tiene el palito 0 en su mano, el filosofo 4 
    //   se queda esperando sin haber levantado ningún palito.
    //
    // Gracias a esto, el palito 4 (el izquierdo del filósofo 4) queda libre en la mesa.
    // - El filósofo 3, que ya tenía el palito 3, ahora puede tomar el palito 4 (su derecha), 
    //   comer tranquilamente y luego devolver ambos a la mesa.
    // - Al liberar sus palitos, se genera una reacción en cadena que permite que el filósofo 2 coma,
    //   luego el 1, luego el 0, y finalmente el 4.


    // La excepcion: 
    // Al ultimo filosofo (4) se le invierten las reglas. 
    // - Se le obliga a intentar tomar primero el palito de su derecha 
    //   (`chopsticks[next]`, que corresponde al palito 0) 
    //   y después el de su izquierda (chopsticks[id], que es el palito 4).
    if id == (N-1) {
        first_chopstick = &chopsticks[next];
        second_chopstick = &chopsticks[id];
    
    // El comportamiento normal: 
    // La mayoria de los filósofos (del 0 al 3) siguen la regla estandar.
    // - Primero toman el palito de su izquierda (`chopsticks[id]`) y luego el de su derecha (`chopsticks[next]`).
    } else {
       first_chopstick = &chopsticks[id];
       second_chopstick = &chopsticks[next];
    }

    // Tratar de forzar tomar en el primer palito en el orden de id
    thread::sleep(Duration::from_millis(100 * id as u64));

    // El filosofo entra en un ciclo infinito entre pensar y comer.
    loop {
        println!("filosofo {} pensando", id);
        thread::sleep(Duration::from_millis(thread_rng().gen_range(500, 1500)));
        println!("filosofo {} esperando palito izquierdo", id);
        {
            // El filosofo adquiere en semaforo del palito izquierdo.
            // - Al asignarlo a una variable dentro de un bloque {...}, 
            //   se asegura que el semáforo se libere automaticamente cuando
            //   termine ese bloque (patrón RAII).
            let _first_access = first_chopstick.access();
            // Pausa despues del primer palito para forzar situacion de DEADLOCK.
            thread::sleep(Duration::from_millis(1000));
            println!("filosofo {} esperando palito derecho", id);
            {
                // El filosofo intenta adquirir el segundo palito. Aqui es donde el programa
                // se congela en caso de no tratar el DEADLOCK.
                let _second_access = second_chopstick.access();
                println!("filosofo {} comiendo", id);
                // Simulamos el tiempo que el filosofo tarda en comer.
                thread::sleep(Duration::from_millis(thread_rng().gen_range(500, 1500)));
            }
        }
    }
}