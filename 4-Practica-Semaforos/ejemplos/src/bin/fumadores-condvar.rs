extern crate rand;
extern crate num_derive;
extern crate num_traits;

use std::sync::{Arc, Mutex, Condvar};
use std::thread;
use std::time::Duration;

use rand::{thread_rng};
use rand::seq::SliceRandom;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use std::thread::JoinHandle;

// Este codigo implementa una solución robusta al "Problema de los fumadores" que elimina por completo el 
// riesgo de DEADLOCK. En lugar de usar múltiples semáforos descoordinados, utiliza un único `Mutex` 
// combinado con una Variable de Condición (`Condvar`). El estado centralizado es simplemente 
// un arreglo de tres valores booleanos que representan "la mesa", indicando qué ingredientes 
// están disponibles en ese momento. Los fumadores ya no toman ingredientes a ciegas ni de a uno por vez;
// en su lugar, despiertan, evaluan atómicamente si sus dos ingredientes necesarios están en la mesa y,
// de ser así, los consumen juntos.

fn main() {

    const N:usize = 3;

    #[derive(Clone, Copy, Debug, FromPrimitive)]
    enum Ingredients {
        Tobacco = 0,
        Paper,
        Fire
    }

    // Creamos una estructura de sincronizacion central.
    // - El estado protegido por `Mutex` es un arreglo de tres booleanos, todos inicializados en `false`,
    //   lo que significa que la mesa comienza vacia.
    // - El `CondVar` se usara para coordinar los turnos entre el Agente y los Fumadores.
    let pair = Arc::new((Mutex::new([false, false, false]), Condvar::new()));

    // --- EL HILO DEL AGENTE ---
    let pair_agent = pair.clone();
    let agent = thread::spawn(move || loop {
        let (lock, cvar) = &*pair_agent;

        println!("[Agente] Esperando a que fumen");
        
        // Revisa la mesa (el arreglo). any(|i| *i) devuelve `true` si HAY ALGO en la mesa.
        // Como `wait_while()` funciona dicicendo "espera MIENTRAS esto sea `true`", el agente se
        // duerme si la mesa está sucia/ocupada.
        let mut state = cvar.wait_while(lock.lock().unwrap(), |ings| {
            let full_table = (*ings).iter().any(|i| *i);
            println!("[Agente] Esperando a que fumen {:?} - {}", ings, full_table);
            full_table
        }).unwrap();

        // Una vez que la mesa esta vacia el Agente crea un vector temporal con los tres ingredientes,
        // lo mezcla aleatoriamente y selecciona los dos primeros para ponerlos en juego.
        let mut ings = vec!(Ingredients::Tobacco, Ingredients::Paper, Ingredients::Fire);
        ings.shuffle(&mut thread_rng());
        let selected_ings = &ings[0..N-1];

        // Cambia el valor booleano a true en los indices correspondientes a los ingredientes seleccionados,
        // "colocandolos en la mesa"
        for ing in selected_ings {
            println!("[Agente] Pongo {:?}", ing);
            state[*ing as usize] = true;
        }
        
        // Envia una señal global para despertar a los tres fumadores que estan dormidos esperando.
        cvar.notify_all();
    });
    // -- LOS HILOS DE LOS FUMADORES---
    // Creamos los tres hilos de los Fumadores, dandole a cada uno su `fumador_id`.
    let smokers:Vec<JoinHandle<()>> =  (0..N)
        .map(|fumador_id|  {
            let pair_smoker = pair.clone();
            let me = Ingredients::from_usize(fumador_id).unwrap();

            thread::spawn(move || loop {
                let (lock, cvar) = &*pair_smoker;

                // El Fumador se va a dormir y se despierta cada vez que el Agente hace `notify_all()`.
                let mut _guard = cvar.wait_while(lock.lock().unwrap(), |ings| {
                    // Esta parte es la que evita el DEADLOCK.
                    // Para todos los ingredientes, la regla exige que:
                    // - O bien el ingrediente es el que ya posee el Fumador.
                    // - O bien el ingrediente esta sobre la mesa.
                    // Si se cumplen ambas condiciones para los dos ingredientes que le faltan, la variable
                    // `my_turn` sera `true`.  
                    let my_turn = (0..N).all(|j| j == fumador_id || ings[j]);
                    println!("[Fumador {:?}] Chequeando {:?} - {}", me, ings, my_turn);
                    
                    // El Fumador se mantendra dormido en el `wait_while()` mientras no sea su turno.
                    !my_turn
                }).unwrap();
                
                // Llegar aca significa que el Fumador tiene el `Mutex` y que los dos ingredientes que necesita
                // estan en la mesa.
                // Como el Fumador tiene la posesión del `Mutex`, ningun otro Fumador puede robar los ingredientes.
                println!("[Fumador {:?}] Fumando", me);
                thread::sleep(Duration::from_secs(2)); // 🚬🚬🚬
                
                // Limpia la mesa: vuelve a poner los 3 booleanos en false
                for ing in (*_guard).iter_mut() {
                    *ing = false;
                }
                println!("[Fumador {:?}] Terminé", me);
                
                // Le avisa al agente, que estaba dormido esperando que la mesa se vaciara.
                cvar.notify_all();
            })
        })
        .collect();

    let _:Vec<()> = smokers.into_iter()
        .flat_map(|x| x.join())
        .collect();

    agent.join().unwrap();
}