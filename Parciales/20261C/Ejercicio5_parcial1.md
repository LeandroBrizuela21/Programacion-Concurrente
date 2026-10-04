## Actores
- `Cliente`
- `LocalComida`
- `LocalJuegos`

### Estado Interno Cliente 
```rust
enum EstadoCliente {
    PASEANDO,
    COMIENDO, 
    YENDO_A_JUGAR,
    JUGANDO,
}

struct Cliente {
    id_telefono: String
    estado: EstadoCliente 
}
``` 

### Mensajes que recibe Cliente 

```rust
// Se envia desde `LocalComida` 
struct ComidaLista{id_pedido: u64}
```
- Cambia el estado de `PASEANDO` A `COMIENDO`.


```rust
// Se envia desde `LocalJuego` 
struct PistaDisponible{id_local: usize}
```
- Cambia el estado de `PASEANDO` A `YENDO_A_JUGAR`.


```rust
// Se envia desde `LocalJuego`
struct ReservaCancelada{id_local: usize}
``` 
- Cambia el estado de `YENDO_A_JUGAR` A `PASEANDO`.

---

### Estado Interno LocalComida 
```rust
struct LocalComida {
    id_local: usize
    pedidos: HashMap <id_pedido, ClienteRef>
}
```

### Mensajes que recibe LocalComida 

```rust
// Se envia desde `Cliente`
struct NuevoPedido{cliente_ref: ClienteRef}
```
- El local de comida registra el pedido mediando un `id` y al cliente mediante una referencia a su `mailbox`.

```rust
// Se envia desde `LocalComida` (Evento interno).
struct CocinaTerminoPedido{id_pedido: u64}
```
- El local de comida se auto-envia un mensaje para simular el "pedido" con su repectivo `id` y de esta forma obtener el `mailbox` de cliente que lo solicitó para avisarle que su comida ya esta lista mediante `ComidaLista`.

---

### Estado Interno LocalJuegos
```rust
struct LocalJuegos {
    id_local: usize
    pistas_disponibles: u32
    cola_de_espera: Queue<ClientRef>
    reservas_pendientes: HashMap<ClienteRef, Tiempo>
}
```
 
### Mensajes que recibe LocalJuegos

```rust
// Se envia desde `Cliente`
struct ReservaNueva{cliente_ref: ClienteRef}
```
- El local de juegos registra una nueva reserva, la cual será almacenada para saber el orden de los clientes según el momento en el cual sacaron la reserva (caso cola no vacía).
- Si la cola está vacía y hay pistas disponibles, inicia el temporizador y le envía `PistaDisponible`.
 

```rust
// Se envia desde `Cliente`
struct ConfirmarLlegada{cliente_ref: ClienteRef}
```
- El local de juegos confirma la llegada del cliente y le disminuye `pistas_disponibles`.

```rust
// Se envia desde `Cliente`
struct PistaLiberada{}
```
- Si la cola de espera no esta vacia, desencola al siguiente cliente, asigna la pista, inicia el temporizador y le envía `PistaDisponible`.
- Si la cola esta vacia simplemente aumenta `pistas_disponibles`.  

```rust
// Se envia desde `LocalJuegos` (autoevento)
struct TimeoutTolerancia {cliente_ref: ClienteRef}
```
- Remueve la reserva del cliente y le envía `ReservaCancelada`.
- Si la cola de espera no esta vacia, desencola al siguiente cliente, asigna la pista, inicia el temporizador y le envía `PistaDisponible`.
- Si la cola esta vacia simplemente aumenta `pistas_disponibles`. 