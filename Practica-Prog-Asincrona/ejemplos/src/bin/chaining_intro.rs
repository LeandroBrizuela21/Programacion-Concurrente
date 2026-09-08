use std::time::Duration;
use async_std::task;

// La idea de este codigo es mostrarnos que aunque utilicemos asyc / .await, este NO se ejecuta en paralelo
// sino de forma estrictamente secuencial. 

async fn hello() -> String {
    println!("before hello");
    task::sleep(Duration::from_secs(2)).await;
    println!("after hello");
    String::from("Hello")
}

async fn world() -> String {
    println!("before world");
    task::sleep(Duration::from_secs(1)).await;
    println!("after world");
    String::from(" World!")
}

async fn async_main() -> String {
    // Estas dos lineas NO ejecutan el codigo de las funciones. Solo contruyen las Maquinas de Estado 
    // (Feature) en memoria.
    // Ambas quedan "dormidas" esperando su primer `poll`.
    let hello1 = hello();
    let world1 = world();
    
    // Rust evalua la expresion de izquierda a derecha
    // - Se ejecuta `hello1.await`, esperamos 2 segundos y devolvemos "Hello"
    // - Cuando finalizamos `hello1.await`, se ejecuta `world1.await`, esperamos 1 segundo y devolvemos "World!"
    // Tienen comportamineto secuencial.
    hello1.await + world1.await.as_str()
}

// main es una funcion sincronica (hilo principal).
fn main() {
    // - `async_main` produce un `Future` pero no lo ejecuta. 
    // - `block_on` funciona como el adaptador del mundo sincrono al asincrono.
    // -  Toma el `Future` y congela el hilo principal y se dedica exclusivamente a 
    //    hacer `poll` a `async_main` hasta que termine y entregue un resultado.       
    println!("{}", task::block_on(async_main()))
}