## Actores
- `Empleado`
- `CreditoArea`
- `Cafetera`
- `Gerente`          
 
### Estado Interno Empleado
```rust 
struct Tarjeta{
    id_tarjeta: usize,
    equipo: ActorRef<CreditoArea>
}

struct Empleado{
    id_empleado: usize,
    tarjeta: Tarjeta
}
``` 

### Mensajes que recibe Empleado

```rust 
// Se recibe desde `Cafetera`.
struct CafeAceptado{}
``` 
- El `Empleado` recibe un mensaje donde se le notifica que su pedido de café fue aceptado.

```rust 
// Se recibe desde `Cafetera`.
struct CafeRechazado{}
``` 
 - El `Empleado` recibe un mensaje donde se le notifica que su pedido de café fue rechazado.
---

### Estado Interno CreditoArea 
```rust 
struct CreditoArea {
    id_area: usize,
    saldo: u32
}
``` 

### Mensajes que recibe CreditoArea

```rust 
// Se envia desde `Gerente`
struct AcreditarSaldo(monto: u32)    
``` 
- El `CreditoArea` recibe un mensaje de `Gerente`, donde le aumenta el saldo disponible para el consumo de sus empleados. 
- Esta acreditación de debe a un bonus o cobro de inicio de mes.

```rust 
// Se envia desde `Cafetera` 
struct SolicitarCompra{cafetera: ActorRef<Cafetera>, monto: u32}
``` 
- El `CreditoArea` recibe un mensaje cuando en alguna `Cafetera` solicito la compra de un cafe.
- Dicha solicitud puede ser aceptada mediante el mensaje `CompraAceptada` (en caso de que se cuente con el `monto` necesario para la compra) o rechazada mediante el mensaje  `CompraRechazada` (en caso de que no se cuente con el `monto` necesario para la compra).

---

### Estado Interno Cafetera
```rust 
struct Cafetera {
    id_cafetera: usize,
    piso: u8
    cliente_actual: ActorRef<Empleado>
}
```

### Mensajes que recibe Cafetera  

```rust  
// Se envia desde `Empleado`
struct SolicitarCafe{monto: u32, tarjeta: Tarjeta, empleado: ActorRef<Empleado>}
``` 
- La `Cafetera` recibe un mensaje del empleando solicitando el monto de los creditos requeridos, su tarjeta donde se encuentra la dirección de su `CreditoArea` y su referencia para que le notifiquen el estado de su solicitud.


```rust 
// Se envia desde `CreditoArea`
struct CompraAceptada{}
``` 
- La `Cafetera` recibe un mensaje de `CreditoArea` donde se le notificará que la solucitud de compra fue aceptada.


```rust 
// Se envia desde `CreditoArea`
struct CompraRechazada{}
``` 
- La `Cafetera` recibe un mensaje de `CreditoArea` donde se le notificará que la solucitud de compra fue aceptada.

---

### Estado Interno Gerente 
```rust 
struct Gerente {
    id: usize,
    id_area: usize,
    equipo_a_cargo: ActorRef<CreditoArea>,
}
```
 
### Mensajes que recibe Gerente 
No recibe mensajes.
 
## ¿Cómo asegura el modelo el caso de "doble gasto"? 
1. **Unico dueño del estado:** El saldo vive exclusivamente dentro del actor `CreditoArea`. Ningún otro actor puede modificarlo directamente.
2. **Procesamiento secuencial (** **Mailbox** **):** Si varias cafeteras solicitan un cobro al mismo tiempo, los mensajes se encolan en el *mailbox* de `CreditoArea` y se procesan **de a uno a la vez**.
3. **Verificación atómica:** Al atender cada solicitud en orden, `CreditoArea` verifica y descuenta el saldo antes de pasar al siguiente mensaje. Si la primera compra agota el saldo, la segunda se rechaza automáticamente.

**Nota:** Asumimos que el `Gerente` es quien se encarga de cargar los creditos mensuales del area.

## Diseño
+---------------+        AcreditarSaldo(monto)       +----------------+
  |    Actor:     | ---------------------------------> |     Actor:     |
  |    Gerente    |                                    |  CreditoArea   |
  +---------------+                                    +----------------+
  (equipo_a_cargo)                                     (id_area, saldo)
                                                           ^      |
                                                           |      |
                             SolicitarCompra(RefCafetera,  |      | CompraAceptada() /
                                             monto)        |      | CompraRechazada()
                                                           |      v
  +---------------+      SolicitarCafe(monto,          +----------------+
  |    Actor:     |      tarjeta, RefEmpleado)         |     Actor:     |
  |   Empleado    | ---------------------------------> |    Cafetera    |
  +---------------+                                    +----------------+
  (id, tarjeta)     <--------------------------------- (cliente_actual)
                      CafeAceptado() / CafeRechazado()