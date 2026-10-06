// Inciso A
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


// Inciso B
fn philosopher(id: usize, first_chopstick: Arc<Semaphore>, second_chopstick: Arc<Semaphore>) {
    loop {
        println!("Philosopher {}: thinking", id);
        thread::sleep(Duration::from_millis(rng.gen_range(500..1500)));
        loop {
            println!("Philosopher {}: taking first chopstick", id);
            let first_access = first_chopstick.acquire();
            println!("Philosopher {}: attempting second chopstick", id);
            match second_chopstick.try_acquire() {
                Ok(second_access) => {
                    println!("Philosopher {}: eating", id);
                    let delay = rand::thread_rng().gen_range(3000..7000);
                    thread::sleep(Duration::from_millis(delay));
                    break;
                }
                Err(_) => {
                    println!("Philosopher {} was unsuccessful", id);
                    drop(first_access);
                }
            }
        }
    }
}


// ¿ES BUSY WAIT?:
// SI ya que:
// - La función `try_acquire()` es no bloqueante, entonces si el palito esta ocuapdo, el hilo no se suspende ni se duerme, por ende evaluara
//   su resultado y en caso de que de error vovlera al loop para intentar nuevamente.
//


// Inciso C
struct Data {
    value: Option<i32>,
}

fn main() {
    let data = Arc::new(Mutex::new(Data { value: None }));

    let c1 = Arc::clone(&data);
    let t1 = thread::spawn(move || {
        // Does some work
        let mut lock = c1.lock().unwrap();
        lock.value = Some(42);
    });

    let c2 = Arc::clone(&data);
    let t2 = thread::spawn(move || loop {
        let lock = c2.lock().unwrap();
        if lock.value.is_some() {
            println!("Valor obtenido: {:?}", lock.value.unwrap());
            break;
        }
    });

    t1.join().unwrap();
    t2.join().unwrap();
}


// ¿ES BUSY WAIT?:
// SI ya que:
// - En caso de `t2` se ejecute primero, el hilo se quedara preguntante repetidamente si el lock tiene algún valor.
//   - En caso de que `t1` logre obtener el lock, `t2` ya hizo "busy wait".
// - Los hilos se ejecutan de forma no determinitica, por ende no se puede garantizar que `t1` se ejecute primero y `t2` no haga busy wait.
// - El objetivo del hilo t2 es reaccionar inmediatamente cuando otro hilo termine de generar un dato.