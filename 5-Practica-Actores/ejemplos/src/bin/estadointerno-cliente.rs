extern crate actix;

use std::time::Duration;

use actix::{Actor, Addr, AsyncContext, Context, Handler, Message, System};
use actix_async_handler::async_handler;
use rand::{Rng, thread_rng};
use tokio::time::sleep;

// Este codigo expande el Modelo de Actores de Actix para construir un ecosistema interactivo con multiples 
// Actores que se comunican entre si de forma concurrente. El sistema centraliza el estado en un Actor
// "Calculadora" (`Calc`), mientras que múltiples actores "Productores" (`Producer`) generan numeros 
// aleatoriamente y se los envían para sumarlos. A su vez, Actores "Reporteros" (`Reporter`) le consultan 
// periodicamente a la calculadora cual es el valor actual para imprimirlo en pantalla. 
// 
// Este codigo introduce el uso de manejadores asíncronos (`#[async_handler]`), 
// permitiendo que un Actor se pause a si mismo (simulando trabajo) 
// sin bloquear el hilo del sistema operativo, y muestra como un Actor puede enviarse mensajes
// a si mismo de manera recursiva para mantener un ciclo de vida infinito.



// 1. MENSAJES DE LA CALCULADORA

// Mensaje para sumar. 
#[derive(Message)]
#[rtype(result = "i32")]
struct Add(i32);

// Mensaje para restar. 
#[derive(Message)]
#[rtype(result = "i32")]
struct Sub(i32);

// Mensaje para consultar el valor. 
#[derive(Message)]
#[rtype(result = "i32")]
struct Get();


// 2. ACTOR CALCULADORA
// Este Actor tiene memoria, guarda el resultado actual de la calculadora.
struct Calc {
    current: i32,
}

// Convertimos el `struct` normal en un Actor de Actix.
impl Actor for Calc {
    // Cada Actor necesita un "Contexto" -> Su entorno de ejecucion / buzon.
    type Context = Context<Self>;
}

// 3. HANDLERS PARA ACTOR CAlCULADORA
// Implementamos la interfaz `Handler` para el mensaje específico 'Add'.
impl Handler<Add> for Calc {
    type Result = i32;

    fn handle(&mut self, msg: Add, _ctx: &mut Context<Self>) -> Self::Result {
        println!("add {}", msg.0);
        self.current += msg.0;
        self.current
    }
}

// Implementamos la interfaz `Handler` para el mensaje específico 'Sub'.
impl Handler<Sub> for Calc {
    type Result = i32;

    fn handle(&mut self, msg: Sub, _ctx: &mut Context<Self>) -> Self::Result {
        println!("sub {}", msg.0);
        self.current -= msg.0;
        self.current
    }
}

// Implementamos la interfaz `Handler` para el mensaje específico 'Get'.
impl Handler<Get> for Calc {
    type Result = i32;

    fn handle(&mut self, _msg: Get, _ctx: &mut Context<Self>) -> Self::Result {
        self.current
    }
}

// 4. DEFINIMOS LOS MENSAJES DEL ACTOR PRODUCTOR
#[derive(Message)]
#[rtype(result = "()")]
struct Produce(); // Mensaje vacio, solo sirve como trigger (disparador).

// 5. ACTOR PRODUCTOR
// Este Actor tiene memoria, guarda la librteta de direcciones de la Calculadora.
struct Producer {
    id: i32,
    calc: Addr<Calc>,
}
// Convertimos el `struct` normal en un Actor de Actix.
impl Actor for Producer {
    // Cada Actor necesita un "Contexto" -> Su entorno de ejecucion / buzon.
    type Context = Context<Self>;
}

// 6. HANDLERS PARA ACTOR PRODUCTOR
#[async_handler] // Permite usar 'await' dentro de este bloque.
impl Handler<Produce> for Producer {
    type Result = ();

    fn handle(&mut self, msg: Produce, ctx: &mut Context<Self>) -> Self::Result {
        // Finge hacer un trabajo pesado, durmiento un rato (asincronamente)
        sleep(Duration::from_millis(thread_rng().gen_range(500, 1500))).await;
        
        // Genera un numero al azar entre -100 y 100
        let amount = thread_rng().gen_range(-100, 100);
        println!("[Producer {}] - sending {}", self.id, amount);
        
        // Le envia la orden de sumar a la Calculadora.
        // - Es un metodo de tipo "dispara y olvida", pero devuelve un `Result` que permite verificar
        //   si hubo un error al enviar el mesaje.
        self.calc.try_send(Add(amount)).unwrap();

        // OCASIONAR UN BUCLE
        // - Se envía el mensaje `Produce()` a sí mismo.
        // - `ctx.address()` devuele la dirección de este mismo acto.
        // - Esto hace que el actor vuelve a ejecutarse casi de inmediato.
        ctx.address().try_send(Produce()).unwrap();
    }
}

// 7. DEFINIMOS LOS MENSAJES DEL ACTOR REPORTERO
#[derive(Message)]
#[rtype(result = "()")]
struct Report();

// 8. ACTOR REPORTERO
struct Reporter {
    id: i32,
    calc: Addr<Calc>,
}

// Convertimos el `struct` normal en un Actor de Actix.
impl Actor for Reporter {
    type Context = Context<Self>;
}

// 8. HANDLERS PARA ACTOR REPORTERO
#[async_handler]// Permite usar el '.await' dentro de este bloque.
impl Handler<Report> for Reporter {
    type Result = ();

    async fn handle(&mut self, msg: Report, ctx: &mut Context<Self>) -> Self::Result {
        // Duerme un rato, reporta menos frecuentemente que los productores.
        sleep(Duration::from_millis(thread_rng().gen_range(1000, 3000))).await;
        
        // Consulta asincrona, le manda el mensaje 'Get' a la Calculadora y se queda esperando
        // el resultado con '.await'. Durante esta espero, no bloquea al SO.
        let result = self.calc.send(Get()).await;
        println!("[Reporter {}] - {}", self.id, result.expect("no vino!"));
        
        // Se envía a si mismo el trigger (disparador) para repetir el ciclo.
        ctx.address().try_send(Report()).unwrap();
    }
}


fn main() {
    // Creamos explicitamente el sistema de ejecucion asincrono de Actix.
    let system = System::new();
    // Ejecutamos un bloque de codigo asicrono inicial para levantar y configurar todos los Actores antes
    // de arrancar el motor.
    system.block_on(async {
        // Instanciamos y hacemos arrancar el Actor Central de la Calculadora.
        let calc = Calc { current: 0 }.start();

        // Instanciamos y hacemos arrancar tres Actores Productores distintos. 
        // A cada uno le pasamos su propio ID y un clon de la direccion de la Calculadora. 
        // Inmediatamente despues de iniciar cada uno, se les envia el primer mensaje `Produce`
        // con `try_send` para que inicien su cilo.
        let producer1 = Producer { id: 1, calc: calc.clone() }.start();
        producer1.try_send(Produce()).unwrap();
        let producer2 = Producer { id: 2, calc: calc.clone() }.start();
        producer2.try_send(Produce()).unwrap();
        let producer3 = Producer { id: 3, calc: calc.clone() }.start();
        producer3.try_send(Produce()).unwrap();

        // Instanciamos y hacemos arrancar dos Reporteros.
        // A cada uno le pasamos su propio ID y un clon de la direccion de la Calculadora. 
        // Les enviamos el mensaje `Report` con `try_send` a cada uno para que inicien su ciclo.
        let reporter1 = Reporter { id: 1, calc: calc.clone() }.start();
        reporter1.try_send(Report()).unwrap();
        let reporter2 = Reporter { id: 2, calc: calc.clone() }.start();
        reporter2.try_send(Report()).unwrap();
    });

    // Ponemos en marcha el sistema de Actix para que gestione los buzones de mensajes y corra
    // el ecosistema hasta que se interrumpa el programa manualmente.
    system.run().unwrap();
}