# Etapa 13.3.c — Coordinación de clips en la aplicación

Fecha: 2026-11-13 (México). **Implementado localmente**, sobre `ac37226` más
los incrementos 13.1–13.3.b conservados en el árbol. Etapa 13 abierta.

## Entrega

- `LauncherSettings` y operaciones validadas de documento: clips/celdas/sample,
  escenas/acciones, grafo completo, cuantización, escena inicial y modo.
  Editar contenido no relanza clips ni modifica snapshots activos/aceptados.
- Cambio de modo con audio cerrado; patrón e inicialización secuenciada requieren
  modo Pattern, launcher requiere Launcher. Error de modo antes de consumir banco.
- Inicialización genérica según modo actual, además de entradas launcher explícitas
  manual/CPAL. Usa PCM preparado y mezcla/BPM actuales, sin releer archivos,
  sin escena automática, Play ni replay al abrir/reabrir.
- Lanzamiento explícito de escena/clip, Stop por fila/total y operación compuesta
  reemplazar clip+lanzar. Snapshot Copy con contenido/cuantización actuales;
  la compuesta confirma documento y recibo solo tras aceptación de cola.
- Último `LauncherLaunch` confirmado en estado transitorio acotado; no entra en
  Project/JSON. Lanzar no cambia escena inicial guardada ni dirty status.
- `LauncherStatus`: selección, recibo y observación separados. Accepted,
  Processed con ID de motor, Rejected y Unknown se basan en el ID exacto de servicio;
  procesar no demuestra que una transición cuantizada ya ocurrió.
- `project_set_tempo` permite selección offline y aceptación FIFO en vivo,
  conservando el API de transporte anterior y la coordinación de mezcla existente.
- Cola llena, errores y timeout inconcluso no confirman una edición compuesta
  ni reemplazan el recibo anterior. Sin compensación/reintento automático; usuario
  debe reconciliar mediante estado observado. Cierre/fallo observado limpia recibos.
- Ediciones disponibles durante Loading/Cancelling/Ready/error de preparación;
  lifecycle, propiedad de banco, cancelación y consumo se conservan.

API y políticas: [contrato de aplicación](../docs/launcher-application.md).

## Verificación

**14 casos nuevos únicos:** cuatro internos de selección/recibos/errores/timeout,
uno de preparación sincronizada y nueve públicos (uno condicionado a native-audio
desactivado). La selección focal también incluye el caso previo de dirty tracking.

- WAV reales: banco instalado después de borrar sus archivos de entrada;
  últimos modo/tempo/mezcla seleccionados y sesión vacía/detenida sin auto escena.
- Copias de documento, payload aceptado y clip activo independientes; relanzamiento
  explícito. Guardar/exportar/reabrir mantiene escena inicial y descarta historial.
- FIFO llena de 128: rechazos sin ID consumido ni cambio de documento/recibo,
  drenado 32/64/96/128 y próxima aceptación 129. Edición solo documental sigue
  disponible sin atribuirle aplicación de audio.
- Escena parcial, Keep/supersesión, fila/StopAll, cambios de cuantización y BPM,
  Pause/Play/Stop y resultados manuales separados de notas programadas.
- Clasificación de errores originales, modo incompatible, Busy antes de consumir
  banco, cancelación/fallo/stale ID, gate nativo y ausencia de fallback.
- Timeout después de encolar realmente una edición compuesta o tempo: selección
  antigua, motor puede aplicar, observación inconclusa y ninguna compensación.
- IDs exactos frente a reporte nuevo/contenido idéntico; Unknown no inventa
  resultado anterior. Ediciones durante trabajador bloqueado y Cancelling.

Verificación final correcta:

- `cargo test -p daw-app --locked --lib launcher --test launcher_application_contract`:
  seis internos (cinco nuevos y dirty tracking previo) + nueve públicos correctos,
  tanto debug como release. Con `native-audio,rt-diagnostics`: seis + ocho,
  también debug/release. Las variantes se solapan; la unión son 14 casos nuevos.
- Clippy de daw-app con `--all-targets --locked -- -D warnings`, tanto headless
  como con `native-audio,rt-diagnostics`: correcto.
- `npm run check`: código 0, 2026-11-14 04:48:20–04:50:59 UTC (13 de noviembre
  en México). Bindings, tipos, lint, 63 frontend, formato, build web, nueve
  escenarios CLI, Rust headless/workspace, audio instrumentado, diagnósticos,
  memoria y Clippy. No se suman las suites solapadas como casos únicos.

Logs en `target/stage13-3c-20261113/`: `check.log`, `release-headless.log` y
`release-native-feature.log`. El [manifiesto](evidencia-aplicacion-clips-13.3c.json)
registra siete archivos de este paso y 40 fuentes/configuraciones acumuladas
respecto a base. De las 34 anteriores, 33 conservan sus hashes y lib.rs de la
aplicación se amplía deliberadamente. Lockfiles, dependencias y bindings intactos;
enlaces locales de los catorce documentos revisados correctos. El check compila
pruebas desktop y build web; no equivale a un bundle completo de escritorio.

## Continuidad y límites

Siguiente: **13.3.d, DTOs/bindings/IPC/ACL y UI de clips/escenas**. Exponer edición
y modo, inicialización por modo y lanzamientos/recibos/estado observado, conservando
modo navegador, orden/pérdida/errores y separación de escena inicial e interpretación.
Después: aceptación local integrada 13.3.e y diagnósticos/escucha/entrega 13.4.

El editor/DTO actual todavía muestra el patrón; proyectos en modo Launcher ya
rechazan edición/inicialización de patrón incompatible. Las nuevas APIs Rust
coordinan launcher y CLI sigue exportando escenas; exposición visual es el paso d.
Motor, servicio de audio, formato, callbacks, dependencias y lockfiles conservan
sus fuentes anteriores. Sin apertura de hardware, nueva escucha ni medición física.
Sin commit/push ni CI nueva; los manifiestos previos son capturas históricas.
