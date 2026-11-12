# Etapa 11.3.a — Servicio de audio secuenciado

Fecha: 2026-11-12. Implementado localmente sobre `1effd485ad157a9a9058e911b849e26dc4903de6`,
que contiene 11.1–11.2. No se hizo commit/push en este incremento. La etapa 11
completa sigue abierta; siguiente: **11.3.b, aplicación**.

## Entrega

- Sesiones secuenciadas manuales/CPAL con banco, patrón, BPM y mezcla explícitos;
  inicialización detenida y selección nativa que exige rate/layout exactos.
- Reemplazo completo del patrón y cambios de celda/slot por la FIFO Copy existente.
  128 comandos, 64 snapshots y hasta 32 comandos por callback, antes de las notas.
- Validación antes de encolar: modo/estado, índices y slots. Slots vacíos válidos;
  rechazos/cola llena sin consumir IDs ni alterar el estado aplicado.
- Snapshot de patrón/fase/última frontera, contadores y último evento del motor,
  capturado tras todos los chunks. Los eventos programados no usan IDs ni
  sustituyen resultados de triggers manuales; ambos comparten pool de voces.
- Conserva rampas y release, agregación de picos del callback completo, errores
  antes de audio parcial, recuperación de observaciones perdidas y cierre del PCM
  en propietario. Reiniciar descarta comandos/patrón de la sesión anterior;
  los IDs del servicio conservan monotonía.

API y semántica: [contrato del servicio](../docs/sequencer-service.md).
La única adaptación de desktop es `sequencer: None` en un fixture Rust de IPC;
no se añadieron DTOs, comandos IPC ni controles visuales.

## Verificación

Dieciséis pruebas nuevas, contando la unión de variantes con/sin features:

- Seis casos públicos de instalación, ediciones/aceptación/aplicación, cola
  compartida, índices/modos/Busy, transporte/BPM, reinicio y gate nativo.
- Tres casos realtime: observaciones perdidas, errores internos sin ack falso y
  preflight que conserva música/voces incluso ante overflow del bloque completo.
- Tres casos de controlador: fallo/reinicio, solicitud expirada y liberación PCM
  en el propietario. El último requiere instrumentación.
- Cuatro casos del adaptador nativo: PCM/snapshots contra callback sin dividir,
  layout/scratch/ventana vacía, fábrica con configuración exacta y cierre desde
  trabajador con devolución del PCM al propietario.

Matriz del adaptador: dos layouts de salida × cuatro chunks (1/127/256/513) ×
cuatro formatos (f32/f64/i16/u16), dos callbacks por combinación. El comando 33
queda para el segundo callback y sus notas conservan frames exactos. Con
instrumentación, cero asignaciones/liberaciones en procesamiento; control
positivo de PCM liberado fuera del trabajador/callback.

Verificación final correcta:

- `npm run check`, código 0, 08:27:22–08:28:58 UTC: bindings, tipos, lint,
  formato, build web, 39 casos frontend, nueve escenarios CLI, suites Rust
  headless/workspace/audio instrumentado/diagnósticos/memoria y Clippy.
- Release con `native-audio,rt-diagnostics`: 37 unitarias y cinco casos públicos
  del secuenciador correctos, incluida la matriz instrumentada del adaptador.
- Release sin features: seis casos públicos correctos, incluido el gate nativo.

Las suites se solapan; no se suman sus ejecuciones como casos distintos. El check
incluye compilación de pruebas desktop y build web, sin generar un bundle/build
completo de escritorio. No se abrió un dispositivo durante estas comprobaciones.

Resultados finales y hashes se registran en
[evidencia de 11.3.a](evidencia-sesiones-secuenciador-11.3a.json).
Logs locales: `target/stage11-3a-20261112/`. Los manifiestos de 11.1–11.2
conservan sus capturas históricas, previas al commit del usuario.

## Siguiente paso y alcance

11.3.b coordina Project seleccionado, aceptación de la cola y patrón aplicado,
inicializa con patrón/mezcla/BPM actuales y conserva preparación/cancelación,
errores y lifecycle. No debe convertir timeout/pérdida de observación en una
confirmación inventada ni mutar el documento al rechazar una cola llena.

IPC/cuadrícula siguen en 11.3.c; aceptación local integrada en 11.3.d. No se abrió
hardware ni se atribuyen tiempos/escucha o CI de versiones anteriores a esta
fuente nueva. Diagnósticos nativos y CI de la entrega siguen en 11.4. Guardar,
abrir y exportar beats permanecen en 12.
