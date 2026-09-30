extern crate actix;

use actix::{Actor, Context, Handler, System, Message};

// El objetivo principal de este codigo es demostrar como un Actor puede mantener y modificar de 
// manera segura un estado interno a lo largo del tiempo (en este caso, funcionando como una calculadora
// con un valor acumulado). Ademas, ilustra como un unico Actor puede estar preparado para recibir y procesar
// diferentes tipos de mensajes (`Add` y `Sub`), y explora tres metodos distintos que tiene el sistema para
// enviarle esos mensajes al actor: 
// - Disparar y olvidar sin chequeo (`do_send`).
// - Disparar y olvidar con chequeo (`try_send`).
// - Esperar la respuesta (`send`). 
// Una ventaja clave de este modelo es que, aunque el Actor muta su estado, no necesita usar candados (Mutex)
// porque procesa los mensajes de su buzon estrictamente de a uno por vez.




// 1. DEFINICIÓN DE MENSAJES 
// - #[derive(Message)] convierte el struct en un mensaje valido.
// - #[rtype(result = "i32")] define que la respuesta del Actor sera un entero.

// Mensaje para sumar. 
#[derive(Message)]
#[rtype(result = "i32")]
struct Add(i32);

// Mensaje para restar.
#[derive(Message)]
#[rtype(result = "i32")]
struct Sub(i32);

// 2. EL ACTOR Y SU ESTADO INTERNO
// Este Actor tiene memoria, guarda el resultado actual de la calculadora.
struct Calc {
    current: i32
}

// Convertimos el `struct` normal en un Actor de Actix.
impl Actor for Calc {
    // Cada Actor necesita un "Contexto" -> Su entorno de ejecucion / buzon.
    type Context = Context<Self>;
}


// 3. LOS HANDLERS 
// Implementamos la interfaz `Handler` para el mensaje específico 'Add'.
impl Handler<Add> for Calc {
    type Result = i32;

    fn handle(&mut self, msg: Add, _ctx: &mut Context<Self>) -> Self::Result {
        // `msg.0` accede al valor dentro de la tupla Add(20).
        println!("add {}", msg.0);
        // Mutamos el estado interno sin necesidad de `Mutex.lock`.
        self.current += msg.0;     
        // Devolvemos el estado actualizado.
        self.current
    }
}

// Implementamos la interfaz `Handler` para el mensaje específico 'Sub'.
impl Handler<Sub> for Calc {
    type Result = i32;

    fn handle(&mut self, msg: Sub, _ctx: &mut Context<Self>) -> Self::Result {
        // `msg.0` accede al valor dentro de la tupla `Add(20)`.
        println!("sub {}", msg.0);
        // Mutamos el estado interno sin necesidad de `Mutex.lock`.
        self.current -= msg.0;
        // Devolvemos el estado actualizado.
        self.current
    }
}

// Inicializamos el entorno asincrono de Actix y definimos la funcion principal.
#[actix_rt::main]
async fn main() {
    // Instanciamos a un Actor `Calc` estableciendo su estado inicial (`current`) en 0.
    // Lo arrancamos con `start()` y obtenemos la direccion de su buzon en `addr`.
    let addr = Calc { current: 0 }.start();  

    // -- PRIMER METODO DE ENVIO --
    // Enviamos un mensaje para sumar 20 utilizando `do_send`.
    // - Este metodo es de tipo "dispara y olvida" -> Ingora por completo la respuesta del Actor
    //                                                y no pausa el hilo principal.
    // - El programa imprime inmediatamente "do_send done".  
    addr.do_send(Add(20));
    println!("do_send done");

    // -- SEGUNDO METODO DE ENVIO --
    // Enviamos un mensaje para sumar 15 utilizando `try_send`.
    // - Tambien es un metodo de tipo "dispara y olvida", pero devuelve un `Result` que permite verificar
    //   si hubo un error al enviar el mesaje.
    addr.try_send(Add(15)).unwrap();
    println!("try_send done");

    // -- TERCER METODO DE ENVIO --
    // Enviamos un mensaje para sumar 5 utilizando `send`.
    //- El `.await` PAUSA la ejecución de esta función `main()` hasta que el actor termine y nos devuelva el resultado final.
    //- Es un metodo de tipo "dispara y espera respuesta", donde devolvera un `Result` que permite verificar el estado del mensaje.
    let res = addr.send(Add(5)).await;
    println!("{}", res.unwrap());

    // Enviamos un mensaje para restar 3 utilizando `send`.
    // Esperamos el resultado con `.await`.
    let res = addr.send(Sub(3)).await;

    println!("{}", res.unwrap());
    // Cerramos el sistema de Actores y finalizamos el programa.
    System::current().stop();
}
