## Actores
- `Auto`
- `Surtidor`
- `GestorTanques`
- `Cisterna`

### Estado Interno Auto
```rust 
struct Auto {
    id: usize
}
``` 

### Mensajes que recibe Auto   

```rust 
// Se envia desde `Surtidor`
struct CargaCompleta{}
``` 
- El `Auto` es notificado de que su petición de carga de combustible ha sido completada.

```rust 
// Se envia desde `Surtidor`
struct CargaRechazada{}

``` 
- El `Auto` es notificado de que su petición de carga de combustible ha sido rechazada.

---

### Estado Interno Surtidor  
```rust 
struct Surtidor {   
    id_surtidor: usize,
    ref_gestor: ActorRef<GestorTanques>
    cliente_actual: Option<ActorRef<Auto>>
}    
```

### Mensajes que recibe Surtidor  

```rust  
// Se envia desde `Auto`
struct SolicitarCarga{auto: ActorRef<Auto>, cantidad: f64}
``` 
- El `Surtidor` recibe una solicitud de carga de combustible un cliente. 
    - Si no tiene un cliente actual, se guarda la dirección del cliente y envía a `GestorTanques` el mensaje `SolicitarTanque`.
    - En caso de tener un cliente actual que esta siendo atendido, rechaza la solicitud enviandole el mensaje `CargaRechazada`.   

```rust 
// Se envia desde `GestorTanques`
struct TanqueAsignado{id_tanque: usize}
``` 
- El `Surtidor` es notificado de que se le asignó un tanque para extraer el combustible, simula la carga, le envía el mensaje `CargaCompleta` a su cliente y modifica `cliente_actual` a `None`. Por último, le envía el mensaje `TanqueLiberado` a `GestorTanques`.

```rust 
// Se envia desde `GestorTanques`
struct SinCombustible{}
``` 
- El `Surtidor` es notificado de que se NO se le asignó un tanque para extraer el combustible por lo que le envía el mensaje `CargaRechazada` a su cliente y modifica `cliente_actual` a `None`.
---

### Estado Interno GestorTanques
```rust 

struct Tanque {
    id: usize,
    capacidad_maxima: f64,
    cantidad_disponible: f64,
    en_uso: Bool
}

enum Estado{
    EN_SERVICIO,
    RECARGA,
}

struct GestorTanques{
    cola_surtidores: Queue<(ActorRef<Surtidor>, cantidad)>,
    cisterna: ActorRef<Cisterna>,
    tanques: Vec<Tanque>,
    estado: Estado
}
```
 
### Mensajes que recibe GestorTanques

```rust 
// Se envia desde `Surtidor`
struct SolicitarTanque{surtidor: ActorRef<Surtidor>, cantidad: f64}
``` 
-  El `GestorTanques` recibe un mensaje de `Surtidor`, para que le habilite la cantidad de combustible solicitado.
-  En caso de que el estado sea `REECARGA`, entonces se esta realizando una recarga del combustible disponible en los tanques. 
   la solitud se encola en la `Queue`.
-  En caso de que el estado sea `EN_SERVICIO` atendemos:
    -  Si la cola > 0 entonces el `GestorTanques` almacena en `Queue` al surtidor para que pueda ser atendido.
    -  Si la cola == 0:
        - Y tenemos gasolina sufuciente entonces el `GestorTanques` pone un tanque en uso (`True`) para que cumpla con lo pedido y le enviará al surtidor `TanqueAsignado`.
        - Y NO tenemos gasolina sufuciente entonces el `GestorTanques` le enviará al surtidor `SinCombustible`.

```rust 
// Se envia desde `Surtidor`.
struct TanqueLiberado{id_tanque: usize}
```
- Si esta en estado `EN_SERVICIO`:
    -  El `GestorTanques` recibe un mensaje de `Surtidor`, pone en dispocisión (`False`) el tanque para que podamos atender al proximo cliente en la fila (si es que hay).
- Si esta en estado `RECARGA`:
    - El `GestorTanques` recibe un mensaje de `Surtidor`, pone en dispocisión el tanque (`False`) y se fija que no haya otros tanques en uso.
        - En caso de que no haya tanques en uso, le manda el mensaje `RecargaAceptada` a la cisterna.

```rust 
// Se envia desde `Cisterna`
struct RecargaFinalizada{}
``` 
- El `GestorTanques` recibe un mensaje de `Cisterna`, para que recarge los tanques.
    - El estado de `GestorTanques` pasa a `EN_SERVICIO`.
    - Los tanques del `GestorTanques` se ponen en capacidad máxima.
    - Hace un `pop()` de la `cola_surtidores()`, para atender al próximo cliente:
        - Si tenemos gasolina sufuciente, entonces el `GestorTanques` pone un tanque en uso (`True`) para que cumpla con lo pedido y le enviará al surtidor `TanqueAsignado`.
        - Si NO tenemos gasolina sufuciente, entonces el `GestorTanques` le enviará al surtidor `SinCombustible`.

```rust 
// Se envia desde `Cisterna`
struct PeticionRecarga{}
``` 
- La `GestorTanques` recibe un mensaje de `Cisterna` para que modifique su estado a `RECARGA` y verifica que todos los taques no estén siendo utilizados, en caso de ser así, le envía a la cisterna el mensaje `RecargaAceptada`.

--- 

### Estado Interno Cisterna
```rust 
struct Cisterna {
    id: usize 
    gestor_tanques: ActorRef<GestorTanque>
}
```
### Mensajes que recibe Cisterna

```rust
struct RecargaAceptada{}
```
- La `Cisterna` recibe un mensaje de `GestorTanques` enviando el mensaje de `RecargaFinalizada`.