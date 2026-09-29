extern crate rand;
extern crate std_semaphore;
extern crate num_derive;
extern crate num_traits;

use std::sync::Arc;
use std::thread;
use std::time::Duration;

use rand::{thread_rng};
use rand::seq::SliceRandom;
use std_semaphore::Semaphore;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use std::thread::JoinHandle;

// ============= PROBLEMA DE LOS FUMADORES =============
// Consideremos que para fumar un cigarrillo se necesitan 3 ingredientes: tabaco, papel y fósforos.
// - Hay 3 fumadores alrededor de una mesa. 
// - Cada uno tiene una cantidad infinita de uno solo de los ingredientes y necesita los
//   otros dos para fumar. 
// - Cada fumador posee un ingrediente distinto.
// Existe además un agente que pone aleatoriamente dos ingredientes en la mesa.
// - El fumador que los necesita los tomará para hacer su cigarrillo y fumará un rato.
// - Cuando el fumador termina, el agente pone otros dos ingredientes.


// Definimos la cantidad de fumadores e ingredientes totales.
const N:usize = 3;

// Creamos un `enum` que representa los tres recursos.
// - Al asignarle 0 al tabaco, el papel y el fuego toman automaticamente los valores 1 y 2.
#[derive(Clone, Copy, Debug, FromPrimitive)]
enum Ingredients {
    Tobacco = 0,
    Paper,
    Fire
}

// Un arreglo con todos los ingredintes para facilitar la obtención.
const ALL_INGREDIENTS: [Ingredients; N] = [Ingredients::Tobacco, Ingredients::Paper, Ingredients::Fire];


fn main() {
    // 1. El SEMÁFORO DEL AGENTE
    // Creamos un semaforo para el agente inicializado en 1. 
    // - Esto significa que el agente tiene permiso inmediato para actuar en la primera ronda.
    let agent_sem = Arc::new(Semaphore::new(1));
    
    // 2. LOS SEMÁFOROS DE LOS INGREDIENTES
    // Creamos un vector de 3 semáforos, todos inicializados en 0.
    // - Un 0 significa "este ingredietne NO está en la mesa".
    let ingredient_sems: Arc<Vec<Semaphore>> = Arc::new((0..N)
        .map(|_| Semaphore::new(0)).collect()
    );

    let agent_sem_a = agent_sem.clone();
    let ingredients_sem_a = ingredient_sems.clone();

    // --- EL HILO DEL AGENTE ---
    let agent = thread::spawn(move || loop {
        println!("[Agente] Esperando sem");
        // El agente adquiere su semaforo. En la primera vuelta pasa de largo, pero luego se bloqueara
        // aqui esperando que un fumador deje de fumar.
        agent_sem_a.acquire();

        
        // Toma la lista de los 3 ingredientes y los baraja al azar.
        let mut ings = ALL_INGREDIENTS.to_vec();
        ings.shuffle(&mut thread_rng());
        
        
        // Tomamos los primeros dos ingredientes del vector mezclado
        // simulando la eleccion aleatoria de los ingredientes a proveer.        
        let selected_ings = &ings[0..N-1];
        
        for ing in selected_ings {
            // Hace un release() (suma 1) al semáforo correspondiente a ese ingrediente
            // avisando que ya está disponible para ser tomado.
            println!("[Agente] Pongo {:?}", ing);
            ingredients_sem_a[*ing as usize].release();
        }
    });

    // --- LOS HILOS DE LOS FUMADORES ---
    // Creamos 3 hilos de fumadores usando un ciclo.
    // - Cada fumador recibe un identificardor i del 0 al 2. 
    let smokers:Vec<JoinHandle<()>> =  (0..N)
        .map(|i|  {
            let agent_sem_smoker = agent_sem.clone();
            let ingredient_sems_smoker = ingredient_sems.clone();
            
            thread::spawn(move || loop {
                // Convierte el numero i del fumador en su ingrendiente principal.
                let me = Ingredients::from_usize(i).unwrap();
                
                // El fumador itera por los 3 ingredintes posibles (0, 1, 2)
                for ing_id in 0..N {
                    // Si el ingrediente NO es el que él ya posee intentara tomarlo de la mesa.
                    if ing_id != i {
                        let ing = Ingredients::from_usize(ing_id).unwrap();
                        println!("[Fumador {:?}] Esperando {:?}", me, ing);
                        
                        // Recordemos que lo hace de a uno por vez, y siempre en orden numérico.
                        //
                        // -- CASO DE DEADLOCK --
                        // 1. El Agente pone Tabaco (Semáforo 0) y Fuego (Semáforo 2) en la mesa.
                        // 2. El Fumador que debería despertar es el de Papel (necesita Tabaco y Fuego).
                        // 3. El Fumador de Fuego (que necesita Tabaco y Papel) despierta rápido, y como su 
                        //    ciclo dice "primero agarra Tabaco", ¡hace acquire del Tabaco y se lo roba!
                        // 4. El Fumador de Fuego ahora intenta agarrar Papel, pero no hay. Se queda bloqueado 
                        //    con el Tabaco en la mano.
                        // 5. El Fumador de Papel despierta. Intenta agarrar el Tabaco, pero el Fumador de Fuego 
                        //    ya se lo robó. Se queda bloqueado.
                        
                        // Problema Subyacente: Los fumadores agarran los ingredintes de a uno. Si un fumador toma un
                        // ingreditne sin saber si el segundo está disponiblem se lo roba a quien realmente lo necesita
                        ingredient_sems_smoker[ing_id].acquire();
                        println!("[Fumador {:?}] Obtuve {:?}", me, ing);
                    }
                }
                println!("[Fumador {:?}] Fumando", me);
                // Simulamos que el fumador esta armando y fumando su cigarrillo.
                thread::sleep(Duration::from_secs(2));
                // El fumador finaliza su tarea y le devuelve el permiso al semaforo del agente,
                // despertandolo para que ponga nuevos ingredientes y comience el siguiente ciclo.
                agent_sem_smoker.release();
                println!("[Fumador {:?}] Terminé", me);
            })
        })
        .collect();

    let _:Vec<()> = smokers.into_iter()
        .flat_map(|x| x.join())
        .collect();

    agent.join().unwrap();
}

// POSIBLES SOLUCIONES:
// ======== Notificación Directa ========
// La forma mas eficiente y directa de arreglar este codigo es cambiar el proposito del vector de semaforos.
// - En lugar de tener un semaforo por cada "ingrediente", se debe tener un semaforo por cada "fumador".
// - Dado que el Agente es quien genera el par de ingredientes, tiene la información suficiente para saber
// quien es el unico que puede fumar en esa ronda.
// 
// Modificación en el Agente: 
// - En lugar de hacer dos .release() sobre los ingredientes que selecciono,
//   el Agente debe calcular cual es el ingrediente faltante (el que no fue seleccionado)
//   y hacer un único .release() directamente sobre el semaforo de ese fumador.
//
// Modificación en los Fumadores: 
// - Se debe eliminar el bucle for interno. 
// - Cada hilo de fumador solo debe hacer un único .acquire() sobre su semaforo personal.
//   Al despertar, tiene la garantia absoluta de que sus dos ingredientes están en la mesa,
//   por lo que simplemente simula fumar y le devuelve el control al Agente.
//
//
// ======== Hilos Intermediarios ========  
// - Se requiere crear tres hilos adicionales llamados "Pushers" o intermediarios, asociados a los semaforos
//   de los ingredientes.
// - Se añade un estado global (variables booleanas) protegido por un Mutex que registra que ingredientes
//   estan actualmente en la mesa.
// - Cuando el Agente libera el `Tabaco`, el hilo `Pusher_Tabaco` se despierta.
//   Este hilo revisa el estado global: 
//      - Si ve que el Papel ya está registrado en la mesa, limpia ambas variables globales y 
//        emite una señal directamente al semaforo del Fumador de Fuego.
//      - Si no hay nada más en la mesa, simplemente marca la variable de `Tabaco` como `true`
//        y vuelve a dormir.
// - Bajo este esquema, los fumadores ya no compiten "a ciegas" por los ingredientes,
//   sino que esperan pacificamente a que los intermediarios armen las parejas y les 
//   notifiquen de forma segura.