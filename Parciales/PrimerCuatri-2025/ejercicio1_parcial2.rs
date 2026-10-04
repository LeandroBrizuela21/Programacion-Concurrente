// Identifique problemas y proponga soluciones.

// Inciso a)
struct Producer {
    buffer: Arc<(Condvar, Mutex<Vec<i32>>)>,
}

impl Actor for Producer {
    type Context = Context<Self>;
}

impl Handler<Produce> for Producer {
    type Result = ();

    fn handle(&mut self, msg: Produce, ctx: &mut Context<Self>) -> Self::Result {
        let (cvar, lock) = &*self.buffer;
        let mut guard = cvar.wait_while(lock.lock().unwrap(), |buf| buf.len() == 10).unwrap();
        guard.push(msg.value);
    }
}

impl Handler<Consume> for Producer {
    type Result = i32;

    fn handle(&mut self, msg: Consume, ctx: &mut Context<Self>) -> Self::Result {
        let (cvar, lock) = &*self.buffer;
        let mut guard = cvar.wait_while(lock.lock().unwrap(), |buf| buf.len() == 0).unwrap();
        return guard.pop().unwrap();
    }
}

// PROBLEMAS:
// 1) Violacion del Modelo de Actores: Compartir un estado de memoria compartida dentro del Actor rompe el 
//    aislamiento y privacidad del estado interno del mismo.
// 2) Rompe con el principio "Compartir memoria comunicando, no comunicar compartiendo memoria".
// 3) Falta de notificaciones: En `Produce` no se hace `cvar.notify_one()` al agregar un elemento, 
//    y en `Consume` tampoco se notifica al quitarlo.
// 4) Usa `cvar.wait_while(...)` y `lock.lock()` dentro del `handle` de un actor. 
// 5) En un sistema de actores, suspender/bloquear un hilo esperando una `Condvar` bloquea el ejecutor, impidiendo
//    que el actor (u otros actores en el mismo hilo) procese mensajes, pudiendo generar un deadlock.

// SOLUCIONES PROPUESTAS:
// 1) Eliminar el Mutex y la CondVar.
// 2) En el Modelo de Actores cada Actor tiene su propio buffer en su estado interno privado.  
// 3) Para la comunicación solo necesitamos enviar mensajes entre los actores y de esta forma podemos modificar el valor
//    del buffer de manera secuencial y segura.


// Inciso b)
impl Handler<PrepararCafe> for Barista {
    fn handle(&mut self, msg: PrepararCafe, ctx: &mut Context<Self>) {
        loop {
            for maquina in self.maquinas.iter() {
                match maquina.send(EstasLibre).await {
                    Ok(true) => {
                        println!("Barista. Máquina libre, preparando café");
                        // Preparar café (sin esperar resultado)
                        maquina.do_send(PrepararCafe);
                        return;
                    }
                    Ok(false) => {
                        println!("Barista. Sigo buscando máquina libre");
                    }
                    Err(e) => {
                        println!("Barista. Error: {}", e);
                    }
                }
            }
        }
    }
}

// PROBLEMAS:
// 1) "Busy Wait": El loop consulta continuamente la condicion de la maquina, no se detiene a esperar ni se suspende ignorando los mensajes que le llegan.
// 2) "Time of Check, Time of Use" (Race Condition): Mientras el Barista recibio el `true` de la maquina, hubo otro Barista que recibio un `true` antes de
//    la misma maquina y le envia en el medio el mensaje `PrepararCafe` antes que el otro.  
 
// SOLUCIONES PROPUESTAS:
// 1) Eliminar el loop, en su lugar hacer una consulta a un Actor que Coordine el acceso a las maquinas mediante un .await y el mismo le notifique cuando haya una disponible.
// 2) El Barista envia directamente `PreparCafe` a la maquina libre que le da el Coordinador. La maquina acepta o encola el pedido en su propio handler.