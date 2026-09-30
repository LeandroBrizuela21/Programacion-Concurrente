extern crate actix;

use actix::{Actor, Context, Handler, System, Message};


// Este codigo introduce el Modelo de Actores, implementado a través del framework actix de Rust.
//  - En el modelo de actores la memoria no se comparte. En su lugar, el sistema se divide en entidades
//    independientes llamadas "Actores" (en este caso, un Greeter) que se comunican exclusivamente enviándose
//    "Mensajes" (en este caso, SayHello) de forma asincrona. Este código muestra la estructura básica de como
//    definir un mensaje, crear un actor, enseñarle a manejar ese mensaje y ejecutar el sistema asíncrono 
//    de Actix.

// 1. DEFINIMOS EL MENSAJE
// En Actix, todo lo que se envia entre Actores debe ser un mensaje.
struct SayHello {
    name: String
}


// Definimos que el mensaje `SayHello` es un mensaje valido.
// A su vez definimos que cuando un Actor reciba este mensaje, el resultado que se espera es un `String`.
impl Message for SayHello {
    type Result = String;
}

// 2. DEFINIMOS EL ACTOR
// Primero creamos un `struct` normal. Aca iria el estado interno del actor.
// En este caso esta vacio.
struct Greeter {}

// Convertimos el `struct` normal en un Actor de Actix.
impl Actor for Greeter {
    // Cada Actor necesita un "Contexto" -> Su entorno de ejecucion / buzon.
    type Context = Context<Self>;
}

// 3. ENSEÑAMOS AL ACTOR A LEER EL MENSAJE 
// Implementamos la interfaz `Handler` para el mensaje específico 'SayHello'.
// Un Actor puede manejar muchos tipos de mensajes distintos, creando un `Handler` para cada uno.
impl Handler<SayHello> for Greeter {
    type Result = String;

    // Esta funcion se ejecuta automaticamente cuando el Actor recibe 'SayHello'.
    fn handle(&mut self, msg: SayHello, _ctx: &mut Context<Self>) -> Self::Result {
        
        // Tomamos el nombre que venía dentro del mensaje, le pegamos "Hello " adelante y lo 
        // devolvemos. Actix se encargara de enviarle esta respuesta al remitente.
        "Hello ".to_owned() + &msg.name
    }
}

// Macro que transforma la funcion tradicional `main` para inicializar y arrancar el motor asincrono (runtime)
// de Actix en segundo plano.
#[actix_rt::main]
// Definimos la funcion principal como asincrona, lo que nos permite usar `await` en su interior. 
async fn main() {
    // Instanciamos el Actor `Greeter` y llamados a `.start()` para ponerlo a correr dentro del sistema.
    // Esto NO devuelve el actor en si, sino una "direccion", que es un canal de comunicacion para enviarle
    // mensajes.  
    let addr = Greeter {}.start();  

    // Utilizamos la "direccion" para enviarle el mensaje `SayHello` con el nombtre "world!".
    // - La instruccion `.await` pausa temporalmente la funcion `main` hasta que el Actor reciba el 
    //   mensaje, lo procese y devuelva la respuesta.
    let mensaje = SayHello { name: String::from("world!")};
    let res = addr.send(mensaje).await;

    // Imprimimos en la consola el resultado devuelto por el actor ("Hello world!").
    // Se usa unwrap() porque .send() devuelve un `Result` que podria contener un error 
    // (por ejemplo, si el buzon del Actor estuviera lleno o el Actor hubiera fallado).
    println!("{}", res.unwrap());
    // Le ordenamos al sistema de Actix que apague todos los actores y cierre el programa de forma limpia.
    System::current().stop();
}