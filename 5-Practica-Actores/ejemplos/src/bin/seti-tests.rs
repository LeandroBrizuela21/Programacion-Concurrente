extern crate actix;

use std::collections::HashSet;
use actix::{Actor, Context, Handler, System, Message, Addr, AsyncContext, Recipient};
use rand::{thread_rng, Rng};
use actix::clock::sleep;
use std::time::Duration;
use actix_async_handler::async_handler;

// En este codigo se refactoriza la implementacion de Actores con el objetivo principal de hacerla testeable 
// (sometible a pruebas unitarias y de integración). En un sistema asincrono y basado en Actores reales, 
// probar el codigo es dificil porque los actores operan en segundo plano de manera infinita y 
// los numeros aleatorios impiden predecir los resultados.
// Para solucionar esto, este codigo introduce la inyección de dependencias
// (permitiendo que el Coordinador envie el resultado final a un Actor "Espia" o falso en lugar de a si mismo)
// y envuelve la generacion de numeros aleatorios en una funcion que puede ser reemplazada por un comportamiento
// predecible durante los tests. Ademas, utiliza la herramienta `Mocker` de Actix para crear Actores falsos
// que interceptan y verifican los mensajes.

const WORKERS:usize = 5;

// 1. MENSAJES
#[derive(Message)]
#[rtype(result = "()")]
struct ProcessSignal {
    amount: f64,
    sender: Recipient<Result>
}

#[derive(Message)]
#[rtype(result = "()")]
struct Result(usize, f64);

#[derive(Message, Debug)]
#[rtype(result = "()")]
struct Epoch(f64);


// === 2. ACTOR COORDINADOR === 
struct Coordinator {
    signal: f64,
    workers: Vec<Recipient<ProcessSignal>>,
    results: HashSet<usize>,
    
    // Opcionalmente, podemos decirle al Coordinador a quién avisarle cuando
    // termine la ronda. Si es 'None', seguira el bucle infinito normal.
    next_epoch_recipient: Option<Recipient<Epoch>>
}

impl Actor for Coordinator {
    type Context = Context<Self>;
}


// Si NO estamos compilando en modo test usamos el generador de números aleatorios de verdad.
#[cfg(not(test))]
fn gen_range(l: f64, h: f64) -> f64 {
    thread_rng().gen_range(l, h)
}

// Si SÍ estamos corriendo test, ignoramos la función de arriba e importamos una función mock
// desde nuestro archivo de pruebas. Así nos aseguramos de que el Worker siempre multiplique
// por el mismo número, haciendo que la prueba sea deterministica. 
#[cfg(test)]
use tests::mock_gen_range as gen_range;


// === Handler de COORDINADOR para INICIAR ÉPOCA ===
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
            worker.try_send(ProcessSignal { amount: signal_local, sender: _ctx.address().recipient()}).unwrap();
        }
    }
}

// === Handler de COORDINADOR para RECIBIR RESULTADOS ===
// Implementamos la interfaz `Handler` para el mensaje específico 'Result'.
impl Handler<Result> for Coordinator {
    type Result = ();

    fn handle(&mut self, msg: Result, _ctx: &mut Context<Self>) -> Self::Result {

        println!("[COORDINADOR] recibí resultado de worker {}", msg.0);

        // Si el Worker no habia entregado todavia en esta ronda, lo insertamos a la lista
        if !self.results.contains(&msg.0) {
            self.signal += msg.1;// Sumamos el pedazo al total global.
            self.results.insert(msg.0);// Anotamos a este wortes en la lista de presentes.

            // En caso de que llegaran todos los workers terminamos la época.
            if self.results.len() == self.workers.len() {
                println!("[COORDINADOR] fin de la epoch, resultado final {}", self.signal);
                
                // INVERSIÓN DE CONTROL
                // - 'as_ref()' nos permite mirar dentro del Option sin destruirlo.
                // - 'unwrap_or()' dice: "Si next_epoch_recipient tiene un destino, úsalo"
                // - Si es None, una '_ctx.address().recipient()', la propia dirección. 
                self.next_epoch_recipient.as_ref().unwrap_or(&_ctx.address().recipient())
                    .try_send(Epoch(self.signal)).unwrap();
            }
        }

    }
}

// === EL ACTOR WORKER ===
struct Worker {
    id: usize,
}

// Convertimos el `struct` normal en un Actor de Actix.
impl Actor for Worker {
    type Context = Context<Self>;
}

// Definimos el Handler del Trabajador como asincrono para no bloquear los hilos del sistema operativo
// mientras procesa.
#[async_handler]
impl Handler<ProcessSignal> for Worker {
    type Result = ();

    fn handle(&mut self, msg: ProcessSignal, _ctx: &mut Context<Self>) -> Self::Result {
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

// Separamos la lógica de inicialización del 'main' para poder llamarla idénticamente desde 
// las funciones de Test.
async fn setup(next_epoch_recipient: Option<Recipient<Epoch>>) -> Addr<Coordinator> {
    
    // Nace el vector de direcciones de los Trabajadores.
    let workers = (0..WORKERS).into_iter()
        .map(|id| Worker { id }.start().recipient())
        .collect::<Vec<Recipient<ProcessSignal>>>();

    // Nace el Coordinador reciendo el parámetro inyectado. 
    let addr = Coordinator { signal: 0., workers, results: HashSet::with_capacity(WORKERS), next_epoch_recipient }.start();

    // Arrancamos la primera época usando .await para asegurar que llegue.
    addr.send(Epoch(100.)).await.unwrap();

    addr
}

fn main() {
    let system = System::new();
    
    // Le pasamos 'None' porque en producción queremos ue sea un bcle infinito.
    // Si estubieramos en un test, le pasariamos 'Some(direccion_del_test)'
    system.block_on(setup(None));

    system.run().unwrap();

}

#[cfg(test)]
mod tests {
    use super::*;
    use ntest::timeout;
    use ntest::assert_about_eq;
    use actix::actors::mocker::Mocker;

    pub(crate) fn mock_gen_range(l: f64, h: f64) -> f64 {
        (l + h) * 0.7
    }

    #[actix_rt::test]
    async fn test_scenario_coordinador_envia() {

        // 1. GIVEN: PREPARA EL ESCENARIO (Coordinador y 3 Trabajadores falsos)
        
        // Creamos un canal de Tokio (asíncrono) con capacidad para 3 mensajes.
        // Es un "tubo secreto".
        // `tx` = Transmisor (lo tendrán los espías)
        // `rx` = Receptor (lo tiene el test)
        let (tx, mut rx) = tokio::sync::mpsc::channel(3);
        let mut workers = vec!();

        
        for id in 0..3 {
            // Clonamos el transmisor para dárselo a este Espía en especifico.
            let mut my_tx = tx.clone();

            // MOCKING: Creamos un Actor falso que intercepta 'ProcesssSignal'
            workers.push(Mocker::<ProcessSignal>::mock(Box::new(move |_msg, _ctx| {
                println!("[{}] recibi {:?}", id, _msg);
                
                // El espionaje, en lugar de procesar la señal, el mock mete el mensaj entero (_msg) en el tubo secreto
                // hacia el test principal.
                my_tx.try_send(_msg);
                
                // Devuelve un "vacio" simulando que cumplio con el Result esperado.
                Box::new(Some(()))
            })).start().recipient()); //Obtenemos su dirección y la guardamos en la lista.
        }

        // SUT = System Under Test 
        // Creamos al Coordinador Real, pero le pasamos la lista de nuestros 3 Espías.
        // Le pasamos 'next_epoch_recipient: None' porque en esta prueba no nos interesa evaluar si termina la ronda, solo
        // si sabe Repartir al principio.
        let sut = Coordinator {
            signal: 0.,
            workers,
            results: HashSet::new(),
            next_epoch_recipient: None
        }.start();

        
        // Disparamos el Coordinador enviandole una señal de 900.0 
        // Usamos '.await' para esperar a que el Coordinador termine de procesar si handler y reparta la señal.
        sut.send(Epoch(900.)).await;

        
        // Si el Coordinador funcioan bien, debió dividir 900/3 = 300
        for _ in 0..3 {
            // 1. Escuchamos por el tubo secreto 'rx.recv().await'.
            // 2. El Mocker envuelve los mensajes en un tipo genérico 'Any', así que usamos 'downcast_ref' para 
            //    volver a transformarlo en un 'ProcessSignal'.
            // 3. 'assert_eq!' Comparamos que el 'amount' dentro de esa orden sea EXACTAMENTE 300.0.
            assert_eq!(300., rx.recv().await.unwrap().downcast_ref::<ProcessSignal>().unwrap().amount)
        }

    }

    // Indicamos que es una prueba unitaria asincrona que debe ejecutarse dentro del motor de Actix.
    // - Nos permite usar `.await`
    #[actix_rt::test]
    async fn test_scenario_coordinador_envia_promesa() {
        // Creamos dos vectores vacios.
        // - `promises`: Guarda los Receptores (promesas de que llegara un mensaje).
        // - `workers`: Guarda las direcciones de los buzones de los Actores falsos.
        let mut promises = vec!();
        let mut workers = vec!();

        for _ in 0..3 {
            // Por cada Trabajador, creamos un canal asincrono de un solo tiro (one shot).
            // - `tx`: Es el lado del transmisor.
            // - `rx`: Es el lado del receptor.
            let (tx, rx) = futures_channel::oneshot::channel();
            // Envolvemos el transmisor en un `Option` (para poder consumirlo de forma segura mas adelante).
            let mut tx_once = Some(tx);
            // Guardamos el receptor en la lista de promesas.
            promises.push(rx);
            // -- CREACION DEL ACTOR FALSO --
            // Creamos un Acto Espia que interceptara mensajes de tipo `ProcessSignal`.
            // Extraemos su direccion y la guardamos en `workers`.   
            workers.push(Mocker::<ProcessSignal>::mock(Box::new(move |_msg, _ctx| {
                // Cuando el Coordinador real le envie un mensaje a este Actor falso, tomara el transmisor `tx` 
                // y le enviará el mensaje interceptado (_msg). 
                // - Se usa `take()` porque los canales one shot se destruyen al usarse. 
                //   Esto garantiza que se consuma una única vez.
                tx_once.take().expect("should be called just once").send(_msg);
                Box::new(Some(()))
            })).start().recipient());
        }

        // Instanciamos el Sistema Bajo Prueba.
        // Creamos el Coordinador real, pero le inyectamos `workers` (que tiene los Actores falsos). 
        let sut = Coordinator {
            signal: 0.,
            workers,
            results: HashSet::new(),
            next_epoch_recipient: None
        }.start();

        // Le enviamos al Coordinador la orden de iniciar una epoca con señal total de 900.0 y 
        // esperamos a que termine de procesar.
        sut.send(Epoch(900.)).await;

        // El Coordinador real debio haber dividido los 900.0 entre los 3 Trabajadores falsos, enviandole
        //  un mensaje a cada uno.
        // El bucle itera sobre las 3 promesas que guardamos al principio.
        for mut p in promises {
            // - `p.await.unwrap()`: Se queda esperando hasta que el Actor falso correspondiente mande el mensaje
            //   interceptado por el canal.
            // - Como el mensaje viaja de forma generica, lo forzamos a convertirse de vuelta a la
            //   estructura `ProcessSignal` para poder leer sus propiedades. 
            assert_eq!(300., p.await.unwrap().downcast_ref::<ProcessSignal>().unwrap().amount)
        }

    }


    #[test]
    // Aseguramos que si el sistema se cuelga (deadlock) o el bucle se vuelve infinito, la prueba
    // falle a los 5 segundos.
    #[timeout(5000)]
    fn test_integration() {
        // Inicializamos un nuevo entorno de ejecucion para el motor de Actores Actix.   
        let system = System::new();
        // Ejecutamos un bloque de codigo asincrono de manera sincronica en el hilo actual.
        // Esto se usa para realizar toda la configuracion y levantar los Actores antes de poner a correr
        // el motor del sistema.
        system.block_on(async {
            // Creamos y hacemos arrancar un Actor Espia Falso (`Mocker`) diseñado especificamente para
            // interceptar un mensaje de tipo `Epoch`.
            // - La funcion anonima (closure) define que hara este Actor falso cuando reciba dicho mensaje.
            let retorno = Mocker::<Epoch>::mock(Box::new(move |_msg, _ctx| {
                // Esta es la primera instruccion dentro del Actor falso.
                // Apenas recibe el mensaje de fin de ronda, emite la orden de detener todo el motor de Actix.
                // Esto es vital para romper el ciclo infinito de la aplicacion original y permite que la 
                // prueba termine.
                System::current().stop();
                // Nucleo de la validacion matematica.
                // Convertimos el mensaje generico recibido a su tipo real `Epoch`, extraemos su valor numerico,
                // y verificamos que sea aproximadamente igual a 1400.0.
                // - Este numero es el resultado de la integracion de todos los Actores reales usando 
                //   la formula matematica predecible (mock) inyectada en las pruebas.       
                assert_about_eq!(_msg.downcast_ref::<Epoch>().unwrap().0, 1400.);
                // Es el valor de retorno obligatorio que exige la herramienta `Mocker` para 
                // indicar que proceso el mensaje correctamente.
                // - Devuelve una unidad vacia envuelta en memoria dinamica.
                Box::new(Some(()))
            })).start();
            // Llamamos a la funcion auxiliar que crea a los Trabajadores y al Coordinador reales, pero le 
            // inyectamos la direccion del Actor falso (`retorno.recipient()`) 
            // Esto cambia el flujo:
            // - Cuando el Coordinador termine la epoca, en lugar de enviarse el mensaje a si mismo para 
            //   continuar, se lo envia a nuestro Actor falso. 
            setup(Some(retorno.recipient())).await;
        });
        // Arrancamos oficialmente el bucle de eventos de Actix.
        // - El hilo de prueba se bloqueara aqui, procesando el intercambio de mensajes entre los Trabajadores
        //   y el Coordinador, hasta que el Actor falso reciba el resultado, ejecute `System::current().stop()` 
        // y libere el programa.
        system.run().unwrap();

    }

}