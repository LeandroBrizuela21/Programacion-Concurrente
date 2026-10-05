## Actores
- `Invitado` 
- `Juego`
- `PoolDinero`

### Estado Interno Invitado
```rust 
struct Tarjeta{
    ref_poolDinero: ActorRef<PoolDinero>
}

struct Invitado{
    id_invitado: usize,
    tarjeta: Tarjeta,
}
``` 

### Mensajes que recibe Invitado

```rust 
// Se recibe un mensaje de `Juego`
struct JuegoAceptado{}
``` 
- El `Invitado` recibe un mensaje de `Juego`, comunicandole que su solicitud a dicho juego fue aceptada.

```rust 
// Se recibe un mensaje de `Juego`
struct JuegoRechazado{}
``` 
- El `Invitado` recibe un mensaje de `Juego`, comunicandole que su solicitud a dicho juego fue rechazada.

---

### Estado Interno Juego
```rust
struct Juego{ 
    id_juego: usize
    monto_a_pagar: u32
}
```

### Mensajes que recibe Juego

```rust  
// Se recibe un mensaje de `Inivitado`
struct SolicitarJuego{ref_invitado: ActorRef<Invitado>, tarjeta: Tarjeta}
``` 
- El `Juego` recibe mensajes de `Invitado`, comunicandole que quiere disponer del juego por lo que le envía `SolicitarPago` al `PoolDinero` asociado a la `Tarjeta` del `Invitado`. 
    - En caso de que se pueda acceder al juego, se le enviará al `Invitado` el mensaje de `JuegoAceptado`.
    - En caso de que se **NO** pueda acceder al juego, se le enviará al `Invitado` el mensaje de `JuegoRechazado`.

```rust 
// Se recibe un mensaje de `PoolDinero`
struct PoolAceptaJuego{ref_invitado: ActorRef<Invitado>}
``` 
- El `Juego` recibe un mensaje de `PoolDinero`, comunicandole que la solicitud del `Invitado` para que se habilite el juego fue aceptada.  


```rust 
// Se recibe un mensaje de `PoolDinero`
struct PoolRechazaJuego{ref_invitado: ActorRef<Invitado>}
``` 
- El `Juego` recibe un mensaje de `PoolDinero`, comunicandole que la solicitud del `Invitado` para que se habilite el juego fue rechazada.

---

### Estado Interno PoolDinero
```rust 
struct PoolDinero {
    saldo: u32
}
```
 
### Mensajes que recibe PoolDinero

```rust 
// Se recibe un mensaje de `Juego`
struct SolicitarPago{ref_juego: ActorRef<Juego>, ref_invitado: ActorRef<Invitado>, monto: u32}
```   
- El `PoolDinero` recibe mensajes de `Juego`, comunicandole que se quiere disponer de un juego en especifico:
    - En caso de que quede `saldo` disponible para cubrir el `monto` del juego, se le enviará al `Juego` el mensaje de `PoolAceptaJuego` y se disminuirá el `saldo`.
    - En caso de que **NO** quede `saldo` disponible para cubrir el `monto` del juego, se le enviará al `Juego` el mensaje de `PoolRechazaJuego` y el `saldo` se mantendra igual.
    