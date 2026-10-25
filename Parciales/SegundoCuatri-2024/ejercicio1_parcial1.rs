//Inciso A
loop {
    match TcpStream::connect("127.0.0.1:8080") {
        Ok(mut stream) => {
            stream.write_all(message.as_bytes()).expect("error")
        }
        Err(_) => {
            let random_result: f64 = rand::thread_rng().gen();
            thread::sleep(
                Duration::from_millis((5000 as f64 * random_result) as u64)
            );
        }
    }
}

// ¿ES BUSY WAIT?: 
//  NO ya que:
// - Hay una operacion de red bloqueante: `connect` es una llamada bloqueante -> El hilo queda bloqueado hasta que obtenga una respuesta.
// - En caso de que no se pueda conectar a la red, el hilo se dormmira por un derterminado tiempo.
//  - Si no tuvieramos ese sleep, preguntaria constantemente.


// Inciso B
loop {
    let random_result: f64 = rand::thread_rng().gen(); // `gen()` da numeros de [0.0, 1.0)
    thread::sleep(Duration::from_millis(random_result as u64)); // el `as u64` trunca a 0. 
    let mut items = self.pending_acks.lock().unwrap();
    let now = Instant::now();
    let mut i = 0;
    while i < items.len() {
        if items[i].expiration <= now {
            if items[i].item_type == "ACK" {
                let _ = items.remove(i);
                drop(items);
                self.send_result_interfaces();
                break;
            }
        } else {
            i += 1;
        }
    }
}

// ¿ES BUSY WAIT?: 
// SI ya que:
// - Debido a que random_result nos devuelve un resultado entre 0 (incluido) y 1 (excluido) y en la instrucción donde deberiamos dormir al hilo
//   lo casteamos a un `u64` provocamos que todo numero que obtengamos de random_result sea 0. 
// - Por ende, como nunca se durmio el hilo, estamos esperando constantemente que llegan acks.

// El BUSY WAIT ocurre únicamente cuando un hilo necesita que pase algo 
// (por ejemplo, que una variable pase a true o que aparezca un elemento en una lista) 
// y se queda en un bucle apretado consultando la memoria RAM sin parar.



// Inciso C
for _ in 0..MINERS {
    let copper = Arc::clone(&resource);
    thread::spawn(move || loop {
        let mined_amount = rand::thread_rng().gen_range(1..10);
        *copper.write().expect("failed to mine") += mined_amount;
        let delay = rand::thread_rng().gen_range(3000..7000);
        thread::sleep(Duration::from_millis(delay));
    });
}

// ¿ES BUSY WAIT?: 
// NO, ya que:
// - El minero NO se va a dormir con la guarda del Lock.
// - El lock se obtiene y se libera en la misma linea, por ende lo suelta de inmediato
//   y no hace falta explicitar su `drop()` o que salga del {} (scope).
// - En caso de que otros hilos quieran acceder al recurso y el mismo este ocupado, el hilo se suspende esperando la 
//   señal del sistema operativo.
// - Aunque los hilos se ejecuten en un loop infinito, al final del minado realizan una espera pasiva en 
//   `thread::sleep(Duration::from_millis(delay))`, duante ese tiempo el SO manda a dormir al hilo, evitando que 
//   pregunten repetidamente, para que el lock los vuelva a dormir sin consumir CPU en caso de no tener el recurso disponible.  
// - El hilo no esta esperando a que otro hilo haga algo ni depende del estado de nadie mas.
//      - Simula un proceso recurrente en el tiempo (picar cobre -> descansar -> picar cobre).
