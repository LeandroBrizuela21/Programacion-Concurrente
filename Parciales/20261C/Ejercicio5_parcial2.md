## Actores
- `Paciente` 
- `CabinaEstudio`
- `Estacion`     

### Estado Interno Paciente
```rust
enum Estudio {
    ORINA,
    EXTRACCION,
    ELECTROCARDIOGRAMA,
    ESPIROMETRIA
}

struct Paciente {
    id_paciente: usize,    
    tramite_actual: Estudio,
    ref_estacion: RefEstacion 
}
```

### Mensajes que recibe Paciente 
```rust
// Se envia desde `CabinaEstudio`.
struct EstudioCompletado{ref_sig_estacion: RefEstacion}
```
- Si `ref_sig_estacion` es válida (no es el final del circuito):
    1. Actualiza `ref_estacion`.
    2. Envía a la nueva estacion `SolicitarAtencion`.         
- Si `ref_sig_estacion` no es válida, concluye el circuito. 

--- 

### Estado Interno CabinaEstudio
```rust
struct CabinaEstudio {
    tipo: Estudio
    id_cabina: u8,
    ref_estacion: RefEstacion,
    ref_sig_estacion: RefEstacion
}

```

### Mensajes que recibe CabinaEstudio
```rust
// Se envia desde `Estacion`
struct AtenderPaciente{refPaciete: RefPaciente} 
```
- Se obtiene la refencia del paciente que va a ser atendido.

--- 

### Estado Interno Estacion 
```rust
struct Estacion {
    id_estacion: u8,         
    cabinas_de_estudios: HashMap< id_cabina, (RefEstudio, atendiendo: Bool)>,
    pacientes_esperando: Queue<(RefPaciente)>
}
```

### Mensajes que recibe Estacion
```rust
// Se envia desde `Cabina`
struct PacienteAtendido {id_cabina: u8} 
```
- La estacion buscara a la cabina correspondiente mediante su `id_cabina`.
- Cuando la cabina termina de atender a un paciente, envia un mensaje a su estación correspondiente:
  - La estación actualiza el `atendiendo` de la cabina, para ponerlo en `False`, esto hace que pueda seguir atendiendo pacientes.


```rust
// Se envia desde `Paciente`
struct SolicitarAtencion {refPaciente: RefPaciente}
```
- Si tiene alguna cabina libre en `cabinas_de_estudios`, la marca como ocupada (`True`) y le envía el mensaje `AtenderPaciente`.    
 
## Diagrama
```
[ PACIENTES (Nuevos) ]
              │
              ▼ (NuevoPaciente)
   ╔══════════════════════════════╗
   ║        ESTACIÓN ORINA        ║
   ║  ├─ Sala Espera: [P][P]      ║ <─────────────────┐
   ╚══╦════════════╦════════════╦═╝                   │
      │            │            │                     │ (CabinaLibre)
      ▼            ▼            ▼                     │
 [Cabina 1]   [Cabina 2]   [Cabina 3] ────────────────┘
      │            │            │
      └──────┬─────┴────────────┘ 
             │ (NuevoPaciente)
             ▼
   ╔══════════════════════════════════════════════════╗
   ║               ESTACIÓN EXTRACCIÓN                ║
   ║  ├─ Sala Espera: [P][P][P]                       ║ <──────┐
   ╚═╦══════╦══════╦══════╦══════╦══════╦═════════════╝        │
     │      │      │      │      │      │                      │ (CabinaLibre)
     ▼      ▼      ▼      ▼      ▼      ▼                      │
   [C 1]  [C 2]  [C 3]  [C 4]  [C 5]  [C 6] ───────────────────┘
     │      │      │      │      │      │
     └──────┴──────┴───┬──┴──────┴──────┘
                       │ (NuevoPaciente)
                       ▼
   ╔══════════════════════════════╗
   ║ ESTACIÓN ELECTROCARDIOGRAMA  ║
   ║  ├─ Sala Espera: [P]         ║ <─────────────────┐
   ╚═══════════════╦══════════════╝                   │
                   │                                  │ (CabinaLibre)
                   ▼                                  │
               [Cabina 1] ────────────────────────────┘
                   │
                   │ (NuevoPaciente)
                   ▼
   ╔══════════════════════════════╗
   ║    ESTACIÓN ESPIROMETRÍA     ║
   ║  ├─ Sala Espera: [ ]         ║ <─────────────────┐
   ╚═══════════════╦══════════════╝                   │
                   │                                  │ (CabinaLibre)
                   ▼                                  │
               [Cabina 1] ────────────────────────────┘
                   │
                   ▼
         [ FIN DEL RECORRIDO ]
```
 