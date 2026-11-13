# Etapa 12 — Persistencia y exportación del beat

**Actualización de cierre:** entrega inicial `6c8c52e`; precisión JSON
corregida en `4f126dc`, con check completo, release (9 casos), build,
WebView2 y CI Linux/Windows correctos. [Cierre definitivo](cierre-etapa-12.md)
y [corrección](correccion-12-precision-json.md). Las cifras/huellas de la
entrega inicial siguientes se conservan como historial fechado.

**Entregada y cerrada dentro de su alcance:** `6c8c52e`, con CI Linux/Windows
correcta. [Cierre](cierre-etapa-12.md) y [publicación](publicacion-secuenciador-proyectos-11-12.md).
La captura local siguiente conserva su contexto anterior a la publicación.

Implementación local sobre `2d4cc28` y las entregas locales del secuenciador.
Forma parte de la [ejecución autorizada 11.4.b–12](ejecucion-11.4b-12.md).
El cierre de escucha de etapa 11 se conserva pendiente hasta la respuesta del
usuario; la persistencia y exportación se desarrollan y comprueban por separado.

## Entrega 12.1–12.3

- Proyecto UTF-8 JSON v1 con ID decimal exacto, título, BPM, patrón fijo 4×16,
  cuatro canales gain/pan/mute, 16 rutas y referencias opcionales a samples.
  La capa de dominio conserva las referencias; no posee PCM ni estado vivo.
- Guardado con copia de assets a una carpeta nueva junto al proyecto y rutas
  relativas portables. Confirmación del JSON desde archivo temporal completado;
  destino nuevo exclusivo y reemplazo explícito. Los errores antes de confirmar
  conservan el archivo anterior y la selección. No se borran carpetas antiguas.
- Apertura limitada a audio cerrado y sin banco en preparación/listo. Valida
  tamaño de 1 MiB, versión, campos/dimensiones, valores, referencias y metadatos
  de archivos antes de reemplazar. Sin migración implícita de versiones futuras.
- Preparación separada de assets guardados; abrir no decodifica, inicializa ni
  reproduce audio. Se mantiene selección frente a preparación/banco aplicado.
- `BeatRender` independiente usa el mismo sampler/mezclador/secuenciador, con
  duración exacta para BPM validados, repeticiones y bloques acotados. `cut`
  termina en los compases; `finish` apaga notas futuras y completa voces activas
  sin reiniciar, disparar otro paso 0 ni aplicar la envolvente de Stop.
- Exportación WAV float32, exclusiva, tamaños RIFF/byte-rate comprobados y
  errores de salida propagados. La aplicación y CLI comparten el escritor;
  hound 3.5.1 pasa a ser dependencia también de daw-app, fuera del callback.
- CLI `project-check` y `export-beat`, sin servicio vivo ni dispositivo.

## Integración desktop 12.4

Cinco comandos nuevos (30 totales), DTOs/bindings generados y ACL de ventana
main: consulta, guardar con título, abrir, preparar assets y exportar. Frames,
IDs y contadores mantienen strings exactos. El panel conserva borradores al
consultar y opciones de exportación al guardar; abrir actualiza patrón/mezcla
atómicamente e invalida respuestas iniciales/consultas anteriores.

[Contrato](../docs/project-format.md) y [guía](../howToRunThis/09-proyectos.md).
La UI de navegador no ejecuta acciones nativas. Guardar y exportar mantienen
el motor vivo; abrir exige cerrarlo. El master transitorio, dispositivo, fase,
voces, medidores y recibos no forman parte del archivo.

## Verificación

20 casos nuevos únicos: seis de aplicación, dos CLI, tres IPC, ocho UI y uno
del cliente desktop. Frontend: 63 casos totales; IPC: diez. Las suites de perfiles
y features repiten casos y no se suman como pruebas únicas.

Las pruebas verifican mover/abrir/preparar tras eliminar originales, round-trip,
ID u64 máximo, versión/JSON/campos/duplicados inválidos, assets ausentes, errores
de guardado y protección de archivo/selección, estados Busy, duración racional,
PCM y colas equivalentes entre bloques 1/127/256/4096, WAV reabierto, no overwrite,
errores del sink y exportación independiente de una sesión manual en Play.
IPC usa archivos reales y prueba permisos con argumentos válidos. UI comprueba
errores sin inventar salida, precisión, preparación explícita, respuestas tardías,
borradores y opciones conservadas, y modo navegador.

La prueba real Tauri/WebView2 guardó un patrón de 18 notas con cuatro assets
relativos, cambió la selección, abrió el guardado y exportó dos compases a
120 BPM: 192000 frames musicales y 5520 de cola, 197520 totales y 36 notas.
El audio permaneció `uninitialized`, sin motor de servicio ni callbacks.
Capturas normal/estrecha y hashes quedan en
[metadatos visuales](../docs/screenshots/stage12-project.json). Se inspeccionan
visualmente; una captura estrecha inicial quedó recortada y se corrige la captura
sin atribuir ese defecto a la UI.

Verificación final correcta: `npm run check`, 01:40:52–01:44:08 UTC del
2026-11-13 (2026-11-12 en México), con 63 casos frontend y suites Rust/Clippy;
release de proyectos (seis app y dos CLI), código 0; build desktop final sin
bundle, código 0. Las capturas finales corresponden exactamente a las 164
fuentes verificadas; no hubo cambios de producción después de capturarlas.

Comandos, resultados y hashes se consolidan en
[evidencia](evidencia-persistencia-exportacion-12.json). Logs/binarios/demos
locales: `target/stage12-20261112/`. Se repitió el check después de ajustar la
presentación de rutas Windows y conservación de formularios. La CI de entrega
posterior está registrada en el cierre y el manifiesto de publicación.

## Cierre y límites

Entrega/CI del SHA exacto completadas; escucha de etapa 11 sigue siendo una
observación externa pendiente. Se conservan
manifiestos anteriores con su estado histórico. La revisión de estado y los
enlaces de publicación se actualizarán al terminar esos pasos.

El formato v1 no incluye clips/escenas ni automatización; corresponden a 13 y
etapas posteriores. No se garantiza recuperación ante corte eléctrico ni se
eliminan assets anteriores automáticamente. El destino debe admitir la operación
de commit requerida; si falla se propaga el error. Los errores durante el WAV
pueden dejar una salida parcial identificada. No se remuestrea ni se atribuye
escucha o continuidad acústica a una exportación offline.

La compilación nativa optimizada actual fue correcta, pero la repetición
adicional de hardware no inició ningún caso: el entorno devolvió EPERM al leer
el ejecutable y la copia archivada dejó de estar disponible. Consulta de
detecciones Windows sin coincidencias; causa no confirmada, sin cambios de
seguridad. La incidencia local se conserva con la evidencia final. La matriz
física anterior sigue ligada a sus 156 fuentes; engine/audio no cambian, pero
no se atribuyen sus tiempos a la revisión nueva de 164 fuentes.
