# Etapa 13.2 — Transiciones y reproducción de clips en el motor

Fecha: 2026-11-13 (México). **Implementado localmente**, sobre `ac37226` más
el modelo 13.1 conservado en el árbol de trabajo. La etapa 13 sigue abierta.

## Comportamiento implementado

- `AudioEngine::with_clip_launcher`: modo opt-in detenido y sin clips activos,
  con el mismo banco, pool global de 16 voces, mezcla, headroom y rampas.
- Reloj racional del secuenciador reutilizado; un origen entero por fila representa
  su fase local exacta sin acumular intervalos redondeados. No modifica el modo
  anterior de patrones ni el comportamiento de aplicaciones/archivos v1.
- Cuantización Step/Beat/Bar sobre el próximo límite global no procesado,
  incluso límites pendientes por redondeo de frame y varias fronteras por frame.
- `ClipLaunchRequest` Copy validado, con Keep/Stop/Launch y snapshots propios;
  `from_scene` captura contenido y conserva Keep como acción. Una solicitud
  pendiente por fila, reemplazo en orden de llamadas y escenas sin aplicación
  parcial. Relanzar el mismo clip reinicia solo esa fila.
- ID monotónico de solicitud del propietario de procesamiento, distinto de los
  futuros IDs de servicio. Snapshots de contenido/identidad/origen/pendientes,
  transiciones, fase exacta, contadores y último evento/resultado real.
- Transición antes de notas de la frontera; orden musical y por fila, ventanas
  semiabiertas. Cambiar/detener una fila conserva colas naturales y rutas
  capturadas; ausencias/pool lleno rechazan solo cada nueva ocurrencia.
- Pause congela fase y pendientes; BPM cambia futuros incrementos sin reiniciar
  música. Stop cancela pendientes inmediatamente; al asentarse reinicia fases
  y voces, conservando clips activos, IDs y contadores de vida de la sesión.
- Preflight del bloque completo: relojes, notas exactas antes/después de cada
  transición y contadores. Overflow/alineación deja salida silenciosa y todo
  estado intacto; buffers vacíos no consumen eventos ni transiciones.

API, semántica y límites: [contrato actualizado](../docs/clips-scenes-contract.md).

## Verificación

**17 pruebas nuevas:** siete unitarias internas, siete de contrato de motor,
dos de referencia/invariancia y una instrumentada de memoria.

- Referencia racional independiente por descomposición IEEE-754: tasas
  1/7/800/44100, BPM 20/95.5/127.25/300 y un ULP junto a 120; los tres grids y
  bloques 1/127/256/4096/completo. Referencia por segmentos para cambio de BPM.
- PCM independiente mono/estéreo con cuatro fuentes distintas, gain/pan/mute,
  tasas 1/800 y cuatro tempos; 48 configuraciones × cinco particiones. Compara
  muestras, contadores/rechazos, estado final y picos acumulados entre particiones.
- Fronteras exactas/anteriores/posteriores, escenas, fases independientes,
  supersesión/Keep, relanzamiento, Pause/resume, Stop interrumpido/asentado,
  colas/rutas, recursos ausentes y pool compartido lleno.
- Errores atómicos de IDs/targets/relojes/notas/transiciones, incluso overflow
  que aparecería después del primer frame. Conteo periódico exacto hasta u64::MAX.
- Memoria: seis combinaciones, tasas 1/800/48000 × mono/estéreo. Solicitudes,
  notas/rechazos, cambios de mezcla, transporte y snapshots registran cero
  asignaciones/liberaciones; al destruir el motor se liberan exactamente cuatro
  buffers PCM fuera del procesamiento.

Verificación final correcta:

- `cargo test -p daw-engine --locked --lib --test launcher_contract --test launcher_reference`:
  25 casos correctos, incluidos los 16 nuevos del motor. La misma selección con
  `--release` también pasa: 16 unitarios totales + siete de contrato + dos de referencia.
- `cargo test -p daw-audio --features rt-diagnostics --locked --test launcher_memory_contract`:
  un caso correcto en sus seis combinaciones, tanto debug como `--release`.
- `cargo clippy -p daw-engine --all-targets --locked -- -D warnings`: correcto.
- `npm run check`: código 0, 2026-11-14 02:17:45–02:20:27 UTC (13 de noviembre
  en México). Bindings, tipos, lint, 63 frontend, formato, build web, nueve
  escenarios CLI, Rust headless/workspace, audio instrumentado, diagnósticos,
  memoria de carga y Clippy. Las suites se solapan; no se suman como casos únicos.

Logs en `target/stage13-2-20261113/`: `check.log`, `release-engine.log` y
`release-memory.log`. El [manifiesto](evidencia-transiciones-clips-13.2.json)
identifica base Git, tres fuentes locales heredadas de 13.1 y siete fuentes
añadidas/modificadas en 13.2, contrato y logs. Las tres fuentes de dominio 13.1
conservan sus hashes; dependencias y lockfiles permanecen iguales.
Enlaces locales de los nueve documentos revisados correctos. El check compila
pruebas de desktop y build web; no equivale a un bundle completo de escritorio.

## Siguiente incremento y límites

**13.3.a:** sesiones launcher manuales/CPAL, solicitudes por FIFO compartida y
snapshots observados; validación por callback/chunks/conversión, pérdida/IDs,
cierre/reapertura y memoria/propiedad. Después siguen Project/formato v2 y
exportación de escena (13.3.b), aplicación (c), desktop (d) y aceptación (e).

Este cambio ofrece reproducción de clips a llamadores directos del motor.
Servicio, aplicación, CLI, IPC y UI todavía usan los modos anteriores. No se
añaden datos de clips a Project ni se escriben bajo versión 1. Sin apertura de
hardware, nueva escucha, medición de tiempos físicos ni CI de una entrega nueva.
No se hizo commit/push; no se atribuye CI anterior al árbol local.
