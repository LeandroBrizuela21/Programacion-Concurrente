// Script 1
struct ResourceGuard {
    guard: Semaphore,
}

impl Actor for ResourceGuard {
    type Context = Context<Self>;
}

impl Handler<Acquire> for ResourceGuard {
    type Result = ();

    fn handle(&mut self, msg: Acquire, _ctx: &mut Context<Self>) -> Self::Result {
        self.guard.acquire();
    }
}

impl Handler<Release> for ResourceGuard {
    type Result = ();

    fn handle(&mut self, msg: Release, _ctx: &mut Context<Self>) -> Self::Result {
        self.guard.release();
    }
}

// Problemas detectados:
// - Violación del Modelo de Actores: Comparte memoria entre actores, en vez de comunicarse con mensajes.
//      - El Actor ya garantiza exclusion mutua sobre su estado interno al procesar los mensajes de a uno a la vez 
//        de manera secuencial (mailbox). 
// - Deadlock: 
//      - Los actores procesan los mensajes de manera secuencial, si el semaforo esta en 0 y un actor intenta adquirirlo, el 
//        ResourceGuard queda bloqueado y no puede procesar otros mensajes, inclusos si otros actores le enviaran un Release.
//      - Como `acquire()` es una opracion bloqueante, si el semaforo no tiene permisos, suspende el hilo del sistema operativo
//        del ejecutor de Actores, provoncando un deadlock.

// Solución propuesta:
// - Eliminar el Semaforo.
// - En caso de querer alterar el estado interno de un Actor, basta con enviarle mensajes provenientes de diferentes Actores. El mismo
//   Actor se encarga de procesarlos de manera secuencial y segura. 

// Script 2
impl Handler<Process> for Resource {
    type Result = ();

    fn handle(&mut self, msg: Process, _ctx: &mut Context<Self>) -> Self::Result {
        if !self.resourceReady {
            _ctx.address().do_send(msg);
        }
        //... 
    }
}

// Problemas detectados:
// - La retroalimentacion se puede cortar debido al uso de `do_send()`, que encola el mensaje en el mailbox del Actor, 
//   pero no garantiza su procesamiento.
// - Busy Wait: Si `self.resourceReady` es `false`, el Actor se envia el mismo mensaje asi mismo inmediatamente. Esto 
//   puede inundar su mailbox procesando el mensaje una y otra vez sin descanso.  

// Solución propuesta:
// - Usar `try_send()` para asegurar el procesamiento del mensaje.
// - Usar un actor que le notifique si el recurso esta listo, en vez de enviarse a si mismo el mensaje.
