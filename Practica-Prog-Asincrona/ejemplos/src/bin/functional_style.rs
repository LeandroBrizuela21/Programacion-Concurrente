use std::future::Future;
use std::time::Duration;

use async_std::task;
use futures::FutureExt;

// Este codigo muestra el estilo funcional de trabajar con asicronia en Rust mediante combinadores
// (`map` y `flatten`). 

async fn hello() -> String {
    task::sleep(Duration::from_secs(2)).await;
    String::from("Hello")
}

async fn world() -> String {
    String::from(" World!")
}

// Debemos declarar explicitamente que está función retorna un `Future` cuyo valor final será un String, esto se debe 
// a que no usamos la palabra `async func` y por ende el compilador no genera la Maquina de Estados.
fn async_main() -> impl Future<Output=String> {
    hello()
        // No extrae el valor de inmediato. Deja la instruccion programada, cuando el future de `hello` termine,
        // guarda su resultado en 'h'
        .map(|h| {
            // Generamos el Future de `world` y programamos su concatenacion.
            // Como esta clausura retorna un Future, el tipo de dato resultante es un Future adentro de otro Future.
            world().map(|w| h + w.as_str())
        })
        // Como tenemos una estructura el tipo `Future<Future<String>>` la aplastamos convirtiendola en un `Future<String>`.
        // Internamente hacemos un poll al primer Future y cuando termina se hace un segundo poll de forma inmediata.
        .flatten()
}

fn main() {
    println!("{}", task::block_on(async_main()));
}