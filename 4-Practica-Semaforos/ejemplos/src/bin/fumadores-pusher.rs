extern crate rand;
extern crate std_semaphore;
extern crate num_derive;
extern crate num_traits;

use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;

use rand::{thread_rng};
use rand::seq::SliceRandom;
use std_semaphore::Semaphore;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use std::thread::JoinHandle;

// Este codigo implementa la solución de David Parnas al problema de los fumadores, utilizando hilos
//  intermediarios conocidos como "pushers" (empujadores). Esta arquitectura permite resolver el posible
// DEADLOCK manteniendo la restricción original del problema: el agente reparte los ingredientes a ciegas
// sin comunicarse directamente con los fumadores. Para lograrlo, los pushers monitorean los ingredientes
// que el agente deja en la mesa y registran este estado en un "marcador" (scoreboard) central. 
// Cuando un pusher detecta que ya hay dos ingredientes disponibles en el marcador, deduce qué fumador 
// tiene el turno, limpia los registros y le envía una señal directa al fumador correcto para que despierte 
// y fume.

const N:usize = 3;

#[derive(Clone, Copy, Debug, FromPrimitive)]
enum Ingredients {
    Tobacco = 0,
    Paper,
    Fire
}

const ALL_INGREDIENTS: [Ingredients; N] = [Ingredients::Tobacco, Ingredients::Paper, Ingredients::Fire];

fn main() {

    // 1. EL SEMÁFORO DEL AGENTE        
    // Creamos un semaforo para controlar al Agente.
    // Empieza en 1 para que tire los primeros ingredientes (le damos permiso para actuar en la 1era ronda).
    let agent_sem = Arc::new(Semaphore::new(1));
    
    // 2. SEMÁFOROS DE INGREDIENTES (la mesa)
    // Creamos un vector de tres semaforos (inicializados en 0).
    // El agente los usa para avisar que ingredientes dejo en la mesa.
    // El agente hace release aca, pero los fumadores YA NO los tocan, los escuchan los Pushers.
    let ingredient_sems: Arc<Vec<Semaphore>> = Arc::new((0..N)
        .map(|_| Semaphore::new(0)).collect());

    
    // 3. SEMÁFORO PRIVADOS DE LOS FUMADORES
    // Creamos un vector de tres semaforos (inicializados en 0).
    // Cada fumador duerme en su propio semaforo.
    // Solo los Pushers los pueden despertar.
    let smoker_sems: Arc<Vec<Semaphore>> = Arc::new((0..N)
        .map(|_| Semaphore::new(0))
        .collect());

    // 4. LA PIZARRA 
    // Un `RwLock` que guarda 3 booleanos para recordar qué ingredeintes ya cayeron en la mesa.
    let scoreboard = Arc::new(RwLock::new(vec!(false, false, false)));

    let agent_sem_a = agent_sem.clone();
    let ingredients_sem_a = ingredient_sems.clone();

    //--- EL HILO DEL AGENTE ---
    // El Agente se ejecuta en un ciclo infinito.
    let agent = thread::spawn(move || loop {
        println!("[Agente] Esperando sem");
        agent_sem_a.acquire(); // Espera su turno para repartir.

        // Mezcla el arreglo constante de ingredientes y selecciona dos al azar.
        let mut ings = ALL_INGREDIENTS.to_vec();
        ings.shuffle(&mut thread_rng());
        let selected_ings = &ings[0..N-1];
        
        // Avisa que puso los ingredientes (esto despertara a un Pusher, no a un fumador).
        // -  Libera a los dos semaforos correspondientes a los dos ingredientes seleccionados y 
        //    vuelve a dormir.
        for ing in selected_ings {
            println!("[Agente] Pongo {:?}", ing);
            ingredients_sem_a[*ing as usize].release();
        }
    });

    // --- LOS HILOS PUSHER ---
    // Creamos los tres hilos intermediarios. Cada Pusher vigila un unico ingrediente.
    let pushers:Vec<JoinHandle<()>> =  (0..N)
        .map(|i|  {
            let ingredient_sems_pusher = ingredient_sems.clone();
            let scoreboard_pusher = scoreboard.clone();
            let smoker_sems_pusher = smoker_sems.clone();
            
            thread::spawn(move || loop {
                let me = Ingredients::from_usize(i).unwrap();
                println!("[Pusher {:?}] Esperando ", me);
                
                // El Pusher duerme esperando SU ingrediente especifico.
                ingredient_sems_pusher[i].acquire();
                println!("[Pusher {:?}] Mi ingrediente esta en la mesa ", me);
                
                // Toma la pizarra para anotar de forma segura (write lock).
                if let Ok(mut scores) = scoreboard_pusher.write() {
                    
                    // Anota que su ingrediente acaba de llegar.
                    scores[i] = true;
                   
                    // Llama a una funcion auxiliar para verificar si, con este nuevo ingrediente, ya se
                    // formo un par completo.
                    if !notify_smokers_if_possible(&mut *scores, &smoker_sems_pusher) {
                        println!("[Pusher {:?}] Lo pongo en el tablero", me);
                    } else {
                        println!("[Pusher {:?}] Despierto fumador", me);
                    }
                }
            })
        })
        .collect();
    
    
    // --- LOS HILOS FUMADORES ---
    // Creamos los tres hilos Fumadores.
    let smokers:Vec<JoinHandle<()>> =  (0..N)
        .map(|i|  {
            let agent_sem_smoker = agent_sem.clone();
            let smoker_sems_smoker = smoker_sems.clone();
            thread::spawn(move || loop {
                let me = Ingredients::from_usize(i).unwrap();
                println!("[Fumador {:?}] Esperando", me);
                // Cada Fumador simplemente espera su semaforo personal.
                // No intenta tomar ingredientes de la mesa.
                smoker_sems_smoker[i].acquire();
                
                // Simula fumar.
                println!("[Fumador {:?}] Fumando", me);
                thread::sleep(Duration::from_secs(2));
                
                // Termina y le da luz verde al Agente para otra ronda mas.
                agent_sem_smoker.release();
                println!("[Fumador {:?}] Terminé", me);
            })
        })
        .collect();

    let _:Vec<()> = pushers.into_iter()
        .chain(smokers.into_iter())
        .flat_map(|x| x.join())
        .collect();

    agent.join().unwrap();
}

// Esta función revisa la pizarra y decide a quién despertar.
// Se ejecuta mientras el `RwLock` de la pizarra está bloqueado (nadie puede interrumpir)
fn notify_smokers_if_possible(scores:&mut Vec<bool>, smoker_sems_pusher: &Arc<Vec<Semaphore>>) -> bool{
    
    // ¿Hay Tabaco en la mesa?
    if scores[Ingredients::Tobacco as usize] {
        
        // ¿Y tambien hay Papel?
        if scores[Ingredients::Paper as usize] {
            // Despertamos al Fumador que tiene Fuego.
            smoker_sems_pusher[Ingredients::Fire as usize].release();
            
            // Limpiamos la pizarra borrando los ingredientes que se van a usar.
            scores[Ingredients::Paper as usize] = false;
            scores[Ingredients::Tobacco as usize] = false;
            true // Avisamos que sí despertamos a alguien.
        
        // ¿Y tambien hay Fuego?
        } else if scores[Ingredients::Fire as usize] {
            // Despertamos al Fumador que tiene Papel.
            smoker_sems_pusher[Ingredients::Paper as usize].release();
            
            // Limpiamos la pizarra borrando los ingredientes que se van a usar.
            scores[Ingredients::Fire as usize] = false;
            scores[Ingredients::Tobacco as usize] = false;
            true // Avisamos que sí despertamos a alguien.
        
        } else {
            // Solo hay Tabaco. Aún falta otro ingrediente.
            false
        }
    
    // No hay tabaco, pero sí hay Papel y Fuego.
    } else if scores[Ingredients::Paper as usize] && scores[Ingredients::Fire as usize] {
        // Despertamos al Fumador que tiene Tabaco.
        smoker_sems_pusher[Ingredients::Tobacco as usize].release();

        // Limpiamos la pizarra borrando los ingredientes que se van a usar.
        scores[Ingredients::Paper as usize] = false;
        scores[Ingredients::Fire as usize] = false;
        true // Avisamos que sí despertamos a alguien.

    } else {
        false // No hay pares completos todavia. 
    }
}
