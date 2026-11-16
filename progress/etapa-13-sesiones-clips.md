# Etapa 13.3.a — Sesiones de clips y comandos del servicio de audio

Fecha: 2026-11-13 (México). **Implementado localmente**, sobre `ac37226` y los
incrementos 13.1/13.2 sin publicar conservados en el árbol. Etapa 13 abierta.

## Entrega

- Sesiones manuales/CPAL opt-in mediante `initialize_manual_clip_launcher` y
  `initialize_native_clip_launcher`, con banco propio, mezcla/tempo explícitos,
  detenidas y sin clips activos/pendientes ni Play automático.
- `request_clip_launch` acepta un payload Copy por la misma FIFO de 128 comandos,
  32 por callback y 64 observaciones. Escenas de cuatro filas en un comando,
  orden compartido con transporte/mezcla/manual triggers, sin drenar entre chunks.
- `ServiceSnapshot.launcher`: estado aplicado completo más último reporte que
  relaciona ID de comando del servicio con ID/resultado de solicitud del motor.
  Aceptar/enviar, procesar y aplicar una transición musical son estados distintos.
- Keep, supersesión por fila y colas naturales conservan la semántica 13.2.
  Notas programadas no se confunden con outcomes de triggers manuales. Picos de
  mezcla y estado capturados después de todo el callback.
- Fuente nativa exige rate/layout exactos y usa la selección de formato existente.
  Feature desactivada y Busy tienen errores explícitos; ningún banco/PCM entra
  por la cola realtime. Error de modo interno falla sin acknowledgment falso.
- Pérdida de observaciones conserva ejecución, sin replay ni éxito inferido.
  Cierre/fallo descarta pendientes/cola; reapertura limpia sin restaurar clips,
  con IDs del servicio monotónicos e IDs de motor nuevos.
- Propiedad de PCM conservada: expiración y cierre manual en el propietario;
  destrucción de callback nativo devuelve el procesador antes de liberar samples.
- Ajuste mínimo del fixture IPC existente para el nuevo campo interno. No cambia
  el DTO público ni los bindings: aplicación/UI de clips siguen pendientes.

Detalle: [contrato de servicio](../docs/launcher-service.md) y
[contrato musical](../docs/clips-scenes-contract.md).

## Verificación

**17 casos nuevos únicos:** cinco del servicio público (uno condicionado a
native-audio desactivado), cuatro realtime internos (uno instrumentado), cuatro
de lifecycle/control (uno instrumentado) y cuatro del adaptador nativo.

- Referencia nativa sin hardware: 32 combinaciones de mono/estéreo, scratch
  1/127/256/513 y f32/f64/i16/u16, siete callbacks por combinación. PCM convertido,
  fases, contenido, IDs/resultados, presupuesto 32/33 y medidores de
  callback completo; escenas, tempo, Pause/Stop/reanudación y colas.
- Realtime instrumentado: 1/800/48000 Hz × mono/estéreo; cola llena, drenado,
  render, pérdida/publicación y transporte sin asignaciones/liberaciones.
  Control positivo: exactamente dos buffers PCM se liberan al destruir el motor
  fuera de la ventana de procesamiento. Retorno nativo y cierre manual también
  verifican liberación en el propietario.
- Separación de aceptación/procesamiento/transición, errores de modo, Busy,
  gate nativo, full queue sin consumir ID, supersesión/Keep y reapertura limpia.
- Pérdida de snapshots, overflow del callback completo sin audio parcial,
  comandos internos inválidos, IDs de servicio agotados y recursos expirados.

Verificación final correcta:

- `cargo test -p daw-audio --locked --lib launcher --test launcher_service_contract`:
  seis casos internos + cinco públicos correctos, en debug y con `--release`.
- Con `--features native-audio,rt-diagnostics`: doce internos + cuatro públicos
  correctos, en debug y release; incluye matrices instrumentadas y retorno de PCM.
  La unión son los 17 casos descritos; las ejecuciones/variantes se solapan.
- Clippy de `daw-audio --all-targets --locked -- -D warnings` correcto tanto
  headless como con `native-audio,rt-diagnostics`.
- `npm run check`: código 0, 2026-11-14 03:15:03–03:17:55 UTC (13 de noviembre
  en México). Bindings, tipos, lint, 63 frontend, formato, build web, nueve
  escenarios CLI, Rust headless/workspace, audio instrumentado, diagnósticos,
  memoria de carga y Clippy. No se suman suites solapadas como pruebas únicas.

Logs en `target/stage13-3a-20261113/`: `check.log`, `release-headless.log` y
`release-native-adapter.log`. El [manifiesto](evidencia-sesiones-clips-13.3a.json)
registra base Git, diez fuentes heredadas 13.1/13.2 sin cambios y diez fuentes
añadidas/modificadas en este incremento, contratos y hashes de logs.
Dependencias/lockfiles y bindings intactos; enlaces locales de los once
documentos revisados correctos. El check compila pruebas de desktop y build web;
no equivale a un bundle completo de escritorio.

## Continuidad y límites

Siguiente: **13.3.b, Project/formato v2 y exportación de escena**. Integrar modo,
launcher, cuantización y escena inicial con migración v1 explícita, round-trip,
assets relativos y errores atómicos; exportar una escena con el mismo motor.
Después: coordinación de aplicación (c), IPC/UI (d), aceptación local (e) y 13.4.

No se abrió hardware ni se midió latencia/carga física o escucha de clips.
CPAL compilado y adaptador probado no certifican audibilidad. Application/CLI/UI
todavía usan los modos previos; archivo v1 y bindings conservan su contrato.
Sin commit/push ni CI nueva; no se atribuyen resultados de entrega anteriores
a este árbol local. Los manifiestos 13.1/13.2 conservan su alcance histórico.
