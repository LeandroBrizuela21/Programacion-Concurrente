extern crate actix;

use std::collections::HashSet;
use actix::{Actor, Context, Handler, System, Message, AsyncContext, Recipient};
use rand::{thread_rng, Rng};
use actix::clock::sleep;
use std::time::Duration;
use actix_async_handler::async_handler;

// Este codigo implemnta una solucion al problema SETI utilizando el Modelo de Actores con el framework Actix.
// En este diseño todo el flujo se basa en el envío asincrono de mensajes. Un Actor Coordinator (Coordinador)
// se encarga de administrar el estado global de la epoca, dividiendo la señal y enviando un mensaje de trabajo
// a multiples Actores Worker (Trabajadores). Los trabajadores procesan su parte de forma asincrona y le envian
// un mensaje de vuelta con el resultado. Cuando el Coordinador detecta que todos los trabajadores le han
// respondido, inicia automaticamente la siguiente epoca enviándose un mensaje a si mismo.

const WORKERS:usize = 5;

// 1. === LOS MENSAJES ===

// Mensaje que el Coordinador le envía al Trabajador.
#[derive(Message)]
#[rtype(result = "()")]
struct Process {
    amount: f64,
    // Una dirección de retorno generica.
    // Quien reciba eso puede usar 'sender' para devolver un mensaje de tipo 'Result'.
    sender: Recipient<Result>
}

// Mensaje que el Trabajador le devuelve al Coordinador.
// Contiene: (ID del worker, resultado calculado).
#[derive(Message)]
#[rtype(result = "()")]
struct Result(usize, f64);

// Mensaje que el Coordinador se envía a sí mismo para iniciar una nueva época.
#[derive(Message, Debug)]
#[rtype(result = "()")]
struct Epoch(f64);



// 2. === EL ACTOR COORDINADOR ===
struct Coordinator {
    // La suma parcial de la época actual
    signal: f64,
    // Lista de las direcciones de los 5 Workers.
    workers: Vec<Recipient<Process>>,
    // Lista de asistencia (quien ya entregó)
    results: HashSet<usize>
}

// Convertimos el `struct` normal en un Actor de Actix.
impl Actor for Coordinator {
    type Context = Context<Self>;
}

// Handler para INICIAR UNA NUEVA EPOCA
// Implementamos la interfaz `Handler` para el mensaje específico 'Epoch'.
impl Handler<Epoch> for Coordinator {
    type Result = ();

    fn handle(&mut self, _msg: Epoch, _ctx: &mut Context<Self>) -> Self::Result {
        let signal = _msg.0;
        
        // Dividimos la señal en partes iguales.
        let signal_local = signal / self.workers.len() as f64;
        
        // Limpiamos las variables.
        self.signal = 0.;
        self.results.clear();

        println!("[COORDINADOR] empieza con señal {}", signal);
        
        // Enviamos el trabajo a cada Worker
        for worker in self.workers.iter() {
            // Empaquetamos el trabajo y metemos su propia dirección de retorno en el mensaje usando '_ctx.address().recipient()'.
            worker.try_send(Process { amount: signal_local, sender: _ctx.address().recipient()}).unwrap();
        }
    }
}

// Handler para RECIBIR RESULTADOS
// Implementamos la interfaz `Handler` para el mensaje específico 'Result'.
impl Handler<Result> for Coordinator {
    type Result = ();

    fn handle(&mut self, msg: Result, _ctx: &mut Context<Self>) -> Self::Result {

        println!("[COORDINADOR] recibí resultado de worker {}", msg.0);

        // Si el Worker no habia entregado todavia en esta ronda, lo insertamos a la lista
        if !self.results.contains(&msg.0) {
            self.signal += msg.1; // Sumamos el pedazo al total global.
            self.results.insert(msg.0); // Anotamos a este wortes en la lista de presentes.

            
            // En caso de que llegaran todos los workers terminamos la época.
            if self.results.len() == self.workers.len() {
                println!("[COORDINADOR] fin de la epoch, resultado final {}", self.signal);
                
                // Si terminamos, nos auto-enviamos el mensaje Epoch con la suma total.
                // Esto desencadenará automáticamente otra ronda (el Hanlder de Epoch).
                _ctx.address().try_send(Epoch(self.signal)).unwrap();
            }
        }

    }
}


// 3. === EL ACTOR WORKER ===
struct Worker {id: usize }

// Convertimos el `struct` normal en un Actor de Actix.
impl Actor for Worker {
    type Context = Context<Self>;
}

// Definimos el Handler del Trabajador como asincrono para no bloquear los hilos del sistema operativo
// mientras procesa.
#[async_handler]
impl Handler<Process> for Worker {
    type Result = ();

    fn handle(&mut self, msg: Process, _ctx: &mut Context<Self>) -> Self::Result {
        println!("[WORKER {}] recibo {}", self.id, msg.amount);
        // Simula el procesamiento. 
        sleep(Duration::from_millis(thread_rng().gen_range(500, 1500))).await;
        // Realiza el calculo aleatorio sobre la porcion que recibio.
        let resultado = msg.amount * thread_rng().gen_range(0., 1.);
        println!("[WORKER {}] devuelvo {}", self.id, resultado);
        // Usa la direccion de retorno `msg.sender` que el Coordinador puso en el sobre inicial
        // para mandarle directamente el mensaje `Result` con su `ID` y el valor calculado.
        msg.sender.try_send(Result(self.id, resultado)).unwrap();
    }
}


fn main() {
    // Creamos el entorno del motor principal de Actix.
    let system = System::new();
    // Ejecutamos el bloque de inicializacion.
    system.block_on(async {
        // Creamos un vector local vacio para guardar las direcciones de los Trabajadores.
        let mut workers = vec!();
        
        // Instanciamos los Actores Workers (Trabajadores), hacemos que arranquen, extraemos su direccion
        // generica (`recipient()`) y la guardamos en el vector `workers`.
        for id in 0..WORKERS {
            workers.push(Worker { id }.start().recipient())
        }

        // Instanciamos al Actor Coordinator (Coordinador), entregandole el vector con las 5 direcciones de 
        // los Trabajadores para que sepa con quien comunicarse.
        Coordinator { signal: 0.0, workers, results: HashSet::with_capacity(WORKERS) }.start()
        // Encadenamos el envio del primer mensaje al Coordinador instanciado para arrancar la epoca 
        // con una señal de 1000.0.     
        .do_send(Epoch(1000.0));
    });

    // Podemos al mortor Actix en un bucle infinito para procesar todos los buzones de mensajes de actores.
    system.run().unwrap();

}