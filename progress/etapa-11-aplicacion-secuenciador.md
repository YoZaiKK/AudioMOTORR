# Etapa 11.3.b — Coordinación del patrón en la aplicación

Fecha: 2026-11-12. Implementado localmente sobre
`2d4cc285434383b451116925f4227219d4bc5171`, entrega de servicio 11.3.a.
La etapa 11 sigue abierta; siguiente: **11.3.c, DTOs/IPC/ACL y cuadrícula**.
Este incremento no hace commit/push.

## Entrega

- `get_pattern` y `PatternEdit`: selección del Project separada de aceptación
  opcional por ID y de la observación real en `ServiceSnapshot.sequencer`.
- `sequencer_set_pattern`, `sequencer_set_step` y `sequencer_set_row_slot`:
  validación antes de mutar, edición sin sesión con `accepted: None`, o
  confirmación del documento solo tras aceptación de la cola en sesión secuenciada.
- Sustitución completa atómica y ediciones basadas en el documento seleccionado,
  evitando perder cambios anteriores que el motor todavía no aplicó.
- Inicialización manual/nativa con banco preparado y patrón, mezcla y BPM
  actuales, tomados al instalar. Sin reabrir archivos, copiar PCM o dar Play.
- Loading/Cancelling/Ready, cancelación, errores y reapertura conservan selección
  y lifecycle. Rechazos/cola llena no cambian el documento ni consumen IDs.
- Timeout inconcluso no se interpreta como rechazo: documento sin confirmación
  permanece igual, sin compensación ni ack inventado, aunque el motor pueda aplicar.

API y reglas: [contrato de aplicación](../docs/sequencer-application.md).
No se modificaron motor, servicio de audio, bindings, IPC ni UI.

## Verificación

Doce pruebas nuevas, contando la unión de features:

- Dos unitarias de política de error/estado y respuesta perdida después de un
  enqueue real, con aplicación posterior y reconciliación explícita por el llamador.
- Una unitaria sincronizada de preparación bloqueada: editar durante Loading y
  Cancelling sin esperar al decoder, conservación tras cancelar o instalar.
- Nueve casos de aplicación con WAV reales: selección offline/ready, instalación
  tras borrar originales, selección/aceptación/aplicación, FIFO y reemplazo atómico,
  pausa/frontera pendiente/BPM/rutas, cola llena, índices/modos inválidos,
  reinicio con intención aceptada, preparación fallida/cancelada/obsoleta y
  rechazos de nota sin rollback. El caso de apertura nativa deshabilitada requiere
  build sin `native-audio`.

El fixture real verifica además picos f32 tras gain/pan/headroom/master, slot y
destino capturado, frames de evento, voces activas, metadata de preparación e IDs exactos.
Son pruebas de coordinación; los oráculos de PCM/eventos y memoria del motor y
adaptador se conservan en 11.2/11.3.a.

Verificación final correcta:

- `npm run check`, código 0, 21:35:56–21:38:09 UTC: bindings, tipos, lint,
  formato, build web, 39 casos frontend, nueve escenarios CLI, suites Rust
  headless/workspace/audio instrumentado/diagnósticos/memoria y Clippy.
- Release sin features: 12 unitarias y nueve casos de aplicación correctos.
- Release con `native-audio`: 12 unitarias y ocho casos de aplicación correctos;
  el caso de apertura nativa deshabilitada queda fuera por su gate de feature.

Las suites se solapan; no se suman sus ejecuciones como casos distintos. Ninguna
de estas comprobaciones abre hardware. El check compila pruebas desktop y build
web, sin realizar un bundle/build completo de la aplicación.

Resultados finales y hashes en el
[manifiesto de 11.3.b](evidencia-aplicacion-secuenciador-11.3b.json).
Logs locales: `target/stage11-3b-20261112/`. Los manifiestos anteriores conservan
su captura histórica, incluido el estado local previo a sus commits.

## Siguiente paso y alcance

11.3.c debe añadir DTOs/bindings/IPC/ACL para selección, recibos y estado aplicado,
cuadrícula de cuatro filas × 16 pasos, slots por fila e inicialización explícita.
Representar frames/IDs/fase `u128` sin pérdida, y usar observaciones del motor
para la posición: ningún temporizador de UI dispara notas. Preservar modo
navegador, recuperación/lifecycle y orden de respuestas/eventos.

Aceptación local integrada en 11.3.d; verificación física/diagnósticos/CI de la
entrega en 11.4. No se abrió hardware ni se atribuyen mediciones/escucha/CI de
revisiones anteriores a este cambio. Guardar/abrir/exportar beats siguen en 12.
