extern crate actix;

use std::collections::HashMap;
use std::time::Duration;

use actix::{Actor, Addr, AsyncContext, Context, Handler, Message, System};
use actix::clock::sleep;
use actix_async_handler::async_handler;
use rand::{Rng, thread_rng};

// Este codigo resuelve el PROBLEMA DE LOS FILOSOFOS COMENSALES utilizando el Modelo de Actores con el 
// framework Actix y una variante del algoritmo distribuido de Chandy-Misra para evitar interbloqueos
// (DEADLOCKS). En esta arquitectura, cada filósofo es un Actor independiente que se comunica con sus
// vecinos a la izquierda y derecha pasandose mensajes para pedir o entregar los palillos.
// La clave para evitar el DEADLOCK radica en el estado higienico de los palillos: 
// - pueden estar "Sucios" (se ceden inmediatamente si alguien los pide). 
// - O "Limpios"(el filosofo los necesita para comer y postergara su entrega hasta terminar).

// Cantidad total de filósofos en la mesa.
const N: usize = 5;

// Alias de tipo para mapear cada palillo con la dirección del Actor (vecino) que lo comparte.
type Neighbours = HashMap<ChopstickId, Addr<Philosopher>>;

// === DEFINICIÓN DE MENSAJES ===

// Mensaje de inicialización para decirle a un filósofo quien son sus vecinos.
#[derive(Message)]
#[rtype(result = "()")]

struct SetNeighbours(Neighbours);

// El filósofo decide empezar a pensar.
#[derive(Message)]
#[rtype(result = "()")]
struct Think;

// El filósofo tiene hambre y va a intentar conseguir los palillos.
#[derive(Message)]
#[rtype(result = "()")]
struct Hungry;

// Un vecino pide un palillo.
#[derive(Message)]
#[rtype(result = "()")]
struct ChopstickRequest(ChopstickId);

// Un vecino nos entrega un palillo que le habíamos pedido. 
#[derive(Message)]
#[rtype(result = "()")]
struct ChopstickResponse(ChopstickId);

// El filósofo verifica si ya tiene todo para comer.
#[derive(Message)]
#[rtype(result = "()")]
struct TryToEat;

// El filósofo avisa que termino su comida.
#[derive(Message)]
#[rtype(result = "()")]
struct EatingDone;


// === ESTADOS Y ESTRUCTURAS PRINCIPALES ===

// Estados posibles de un palillo según el algorimo de Chandy-Misra
#[derive(PartialEq)]
enum ChopstickState {
    DontHave,   // No tengo el palillo, debo pedirlo.
    Dirty,      // Lo tengo pero ya lo úse (no tengo prioridad, si me lo piden lo doy).
    Clean,      // Lo recibi recién (tengo prioridad para usarlo).
    Requested   // Lo tengo (estoy comiendo) pero un vecino ya me lo pidio. Se lo doy al terminar.
}


// Identificador único para cada palillo (basado en numeros enteros).
#[derive(PartialEq, Eq, Hash, Copy, Clone)]
struct ChopstickId(usize);

// Definición del Actor: Filósofo
struct Philosopher {
    id: usize,                                          // Número del filósofo.
    chopsticks: HashMap<ChopstickId, ChopstickState>,   // Estado de sus dos palitos.
    neighbours: Neighbours                              // Referencia a los actores vecinos.
}

// Implementación base para que Actrix reconozca a 'Philosopher' como un Actor.
impl Actor for Philosopher {
    type Context = Context<Self>;
}

// === HANDLER - MANEJANDO EVENTOS ===

// Inicialización: Configura los vecinos y dispara el estado inicial "Pensar" a los filosofos.
impl Handler<SetNeighbours> for Philosopher {
    type Result = ();

    fn handle(&mut self, msg: SetNeighbours, ctx: &mut Context<Self>) -> Self::Result {
        println!("[{}] recibi a mis vecinos", self.id);
        self.neighbours = msg.0;                // Guarda el mapa de vecinos.
        ctx.address().try_send(Think).unwrap(); // Inicia el ciclo manándose el mensaje "Think".
    }
}

// ESTADO: PENSANDO
#[async_handler]
impl Handler<Think> for Philosopher {
    type Result = ();

    async fn handle(&mut self, msg: Think, ctx: &mut Context<Self>) -> Self::Result {
        println!("[{}] pensando", self.id);
        // Simula el tiempo que pasa pensando pausando la ejecución de 2 a 5 segundos.
        sleep(Duration::from_millis(thread_rng().gen_range(2000, 5000))).await;
        // Al terminar de pensar le da hambre. Colocamos "Hungry" en nuestro propio buzón de entrada. 
        ctx.address().try_send(Hungry).unwrap();
    }
}

// INTENTO DE COMER: VERIFICA SI TIENE LOS RECURSOS
#[async_handler]
impl Handler<TryToEat> for Philosopher {
    type Result = ();

    async fn handle(&mut self, msg: TryToEat, ctx: &mut Context<Self>) -> Self::Result {
        // Chequea si tiene AMBOS palillos (ninguno está en estado DontHave).
        if self.chopsticks.iter().all(|(_id, state)| *state != ChopstickState::DontHave) { // si los tengo todos
            println!("[{}] comiendo", self.id);
            // Simulo el acto de comer de 2 a 5 segundos.
            sleep(Duration::from_millis(thread_rng().gen_range(2000, 5000))).await;
            // Al terminar dispara el evento EatingDone.
            ctx.address().try_send(EatingDone).unwrap();
        } else {
            // Si le faltan palillos, simplementa avisa y espera
            println!("[{}] aun no puedo comer", self.id);
        }
    }
}

// ESTADO: HAMBRIENTO (pide los palillos que le faltan)
#[async_handler]
impl Handler<Hungry> for Philosopher {
    type Result = ();

    async fn handle(&mut self, _msg: Hungry, ctx: &mut Context<Self>) -> Self::Result {
        println!("[{}] por comer", self.id);
        // Itera sobre sus dos palillos correspondientes.
        for (chopstick_id, state) in self.chopsticks.iter() {
            
            // Si le falta alguno, le manda un mensaje ´ChopstickRequest´ al vecino correspondinte
            if *state == ChopstickState::DontHave {
                println!("[{}] pido palito {}", self.id, chopstick_id.0);
                self.neighbours.get(chopstick_id).unwrap().try_send(ChopstickRequest(*chopstick_id)).unwrap();
            }
        }
        // Inmediatamente después de pedir (o si ya tenía los dos), intenta comer.
        ctx.address().try_send(TryToEat).unwrap();
    }
}

// MANEJO DE PETICIONES: ¿Que hago si un vecino me pide un palillo?
impl Handler<ChopstickRequest> for Philosopher {
    // Define que procesar este mensaje no devuelve ningun valor de retorno sincronico directo al emisor. 
    // - Devuelve la unidad vacia `()`.
    type Result = ();

    // Funcion que se ejecuta al recibir el mensaje.
    fn handle(&mut self, msg: ChopstickRequest, _ctx: &mut Context<Self>) -> Self::Result {
        println!("[{}] me piden palito {}", self.id, msg.0.0);
        let chopstick = msg.0;
        // Busca en el diccionario interno del filósofo (`self.chopsticks`) cual es el estado actual del 
        // palillo que le acaban de pedir. 
        // - Esto devuelve un envoltorio `Option` (que contendrá `Some` con el estado,
        //   o None si la clave no existiera)
        let chopstick_state = &self.chopsticks.get(&chopstick);
        
        match chopstick_state {
            // PRIMER CASO:
            // Evalua si el filosofo tiene el palillo y su estado es "Sucio" 
            // (esto significa que ya termino de usarlo o no tiene intención inmediata de comer, 
            // por lo que no tiene prioridad para retenerlo).
            Some(ChopstickState::Dirty) => {
                println!("[{}] se lo doy ahora", self.id);
                // Busca en su mapa de vecinos (`self.neighbours`) la direccion del buzon del filosofo asociado
                // a ese palillo especifico, y le envía de forma no bloqueante (`try_send`) un mensaje 
                // `ChopstickResponse` confirmando la entrega del palillo.
                self.neighbours.get(&chopstick).unwrap().try_send(ChopstickResponse(msg.0)).unwrap();
                // Actualiza su propio diccionario de palillos, registrando que acaba de entregarlo 
                // y por lo tanto su nuevo estado es "No lo tengo"
                self.chopsticks.insert(chopstick, ChopstickState::DontHave);
            },
            // SEGUNDO CASO: 
            // Evalua si el filosofo tiene el palillo y su estado es "Limpio" (esto significa que lo 
            // necesita para comer y el algoritmo le otorga prioridad para retenerlo).
            Some(ChopstickState::Clean) => {
                println!("[{}] se lo doy cuando termine", self.id);
                // En lugar de entregarlo, actualiza el estado interno del palillo a "Solicitado" (`Requested`).
                self.chopsticks.insert(chopstick, ChopstickState::Requested);
            },
            // : CASO POR DEFECTO: 
            // Se ejecuta si el palillo esta en cualquier otro estado,
            // como por ejemplo si un vecino le pide un palillo que ya marco como `DontHave` o `Requested`.
            _ => {
                println!("[{}] no deberia pasar", self.id);
            }
        }
    }
}


// REPUESTA A PETICIÓN: Recibi un palillo que pedí 
#[async_handler]
impl Handler<ChopstickResponse> for Philosopher {
    type Result = ();

    async fn handle(&mut self, msg: ChopstickResponse, ctx: &mut Context<Self>) -> Self::Result {
        println!("[{}] recibi palito {}", self.id, msg.0.0);
        // Cuando me dan un palillo, siempre llega ´Clean´
        self.chopsticks.insert(msg.0,ChopstickState::Clean);
        // Como me llegó un recurso nuevo, intento comer nuevamente. 
        ctx.address().try_send(TryToEat).unwrap();
    }
}

// LIMPIEZA: Termine de comer
#[async_handler]
impl Handler<EatingDone> for Philosopher {
    type Result = ();

    async fn handle(&mut self, _msg: EatingDone, ctx: &mut Context<Self>) -> Self::Result {
        println!("[{}] terminé de comer", self.id);
        for (chopstick, mut state) in self.chopsticks.iter_mut() {
            if *state == ChopstickState::Requested {
                // Si mientras comia un vecino me pidio el palillo, se lo envio ahora.
                println!("[{}] entrego palito {}", self.id, chopstick.0);
                self.neighbours.get(chopstick).unwrap().try_send(ChopstickResponse(*chopstick)).unwrap();
                // Me quedo sin el palillo.
                *state = ChopstickState::DontHave
            } else {
                // Si nadie me lo pidio, me lo quedo pero ahora está Sucio (pierdo prioridad)
                println!("[{}] marco como sucio palito {}", self.id, chopstick.0);
                *state = ChopstickState::Dirty
            }
        }
        // Vuelvo al estado inicial, pensar.
        ctx.address().try_send(Think).unwrap();
    }
}


fn main() {
    // Levantamos el entorno de Actix.
    let system = System::new();
    system.block_on(async {
        // Creamos un vector para guardar los Actores Filosofos.
        let mut philosophers = vec!();

        // -- CREACION ASIMETRICA --
        // Instanciamos a los Filosofos.
        // Para evitar el DEADLOCK, inicializamos el estado de los palillos de forma asimetrica
        // basandonos en el ID. 
        for id in 0..N {
            // Deadlock avoidance forcing the initial state
            philosophers.push(Philosopher {
                id,
                chopsticks: HashMap::from([
                    // - Para el palillo izquierdo: Solo el filosofo 0 empieza con este palillo (`Dirty`),
                    //   el resto empieza si el (`DontHave`).
                    (ChopstickId(id), if id == 0 { ChopstickState::Dirty } else { ChopstickState::DontHave }),
                    // - Para el palillo derecho:  Todos los filosofos excepto el ultimo empiezan con este
                    //   palillo (`Dirty`), excepto el ultimo que empieza sin el (`DontHave`).
                    (ChopstickId((id + 1) % N), if id == N-1 { ChopstickState::DontHave } else { ChopstickState::Dirty })
                ]),
                neighbours: HashMap::with_capacity(2)
            // Le hacemos `start()` a cada Actor y guardamos su direccion en el vector `philosophers`.
            }.start()) 
        }

        // -- CONEXION DE VECINOS --  
        // Iniciamos un bucle que itera 5 veces, recorriendo cada ID de Filosofo para asegurar
        // individualmente su red de contactos.
        for id in 0..N {
            // Calculamos el ID del Filosofo sentado inmediatamente a la izquierda.
            // - Si se trata del primer Filosofo, la logica circular determina que su vecino izquierdo 
            //   es el ultimo de la mesa
            // - Para cualquier otro, su vecino izquierdo es el ultimo de la mesa.  
            let prev = if id == 0 { N - 1 } else { id - 1 };
            // Calculamos el ID del Filosofo sentado inmediatamente a la derecha utilizando aritmetica modular.
            // - Si es el ultimo Filósofo, la operación da como resultado 0, cerrando el circulo.
            // - Para el resto, simplemente devuelve el ID siguiente.
            let next = (id + 1) % N;
            // Accedemos a la direccion del Filosofo actual y le enviamos de forma no bloqueante (`try_send`)
            // el mensaje inicial `SetNeighbours`.
            // - Dentro del mensaje, construimos el diccionario que guardara las direcciones de los vecinos. 
            philosophers[id].try_send(SetNeighbours(HashMap::from([
                (ChopstickId(id), philosophers[prev].clone()),
                (ChopstickId(next), philosophers[next].clone())
            ]))).unwrap();
        }
    });
    // Mantenemos el programa en ejecucion indefinidamente procesando la red de Actores.
    system.run().unwrap();
}