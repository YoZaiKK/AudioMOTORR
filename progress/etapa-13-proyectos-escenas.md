# Etapa 13.3.b — Proyectos v2, migración y exportación de escenas

Fecha: 2026-11-13 (México). **Implementado localmente** sobre `ac37226` y los
incrementos 13.1–13.3.a conservados en el árbol. Etapa 13 abierta.

## Entrega

- `Project` posee launcher, modo Pattern/Launcher, cuantización y escena inicial
  validada. Copias independientes, patrón heredado separado y dirty tracking de
  todos los campos nuevos. Editar documento no aplica controles a audio vivo.
- JSON v2 con todos los campos anteriores más modo, cuantización, escena inicial,
  cuatro filas × 16 clips opcionales y 16 escenas × cuatro acciones. Objetos,
  dimensiones, slots y grafo completos validados; campos nuevos obligatorios.
- Lector v1/v2 con migración explícita: identidad, título, BPM/gain/pan exactos,
  patrón, mezcla, rutas y assets conservados; escena 0 copia el patrón, modo
  Pattern, Bar y escena inicial 0. Leer no reescribe el original; guardar escribe
  siempre v2, sin down-save silencioso ni datos de reproducción transitorios.
- Copia portable de assets, límite 1 MiB, guardado exclusivo/reemplazo explícito,
  conservación de archivo/documento anterior ante errores y limpieza de recursos
  nuevos. Se conservan las garantías del adaptador previo.
- Export independiente según modo/escena inicial o selección explícita mediante
  `BeatRender::from_scene`/`open_scene` y CLI `export-beat --scene 0..15`.
  Usa el mismo motor de clips desde silencio: Keep/Stop inactivos, sin historial.
- Duración musical exacta, `cut` y `finish` naturales. Al iniciar colas se paran
  todas las filas en el próximo límite Step no procesado, impidiendo notas
  adicionales incluso si varias fronteras coinciden en el primer frame de cola.
- `project-check` indica la versión real del archivo original, incluida v1
  migrada en memoria. Opciones inválidas no crean archivos y no se sobrescribe WAV.

Contrato actual: [formato y exportación](../docs/project-format.md).

## Verificación

**14 casos nuevos:** dos de dominio, uno interno de dirty tracking, cinco de
persistencia v2, cuatro de exportación y dos de CLI real.

- Migración v1 con ID máximo, BPM/gain/pan exactos, patrón, modo y snapshots;
  archivo original intacto y guardado posterior v2. Round-trip de los 64 slots,
  16 escenas, Keep/Stop/Launch, tres cuantizaciones y configuración inicial.
- Matriz de 26 grafos/campos/duplicados/versiones inválidos, selección intacta;
  límite 1 MiB exacto y un byte de exceso tanto v1 como v2. Relocación después de
  borrar originales y reemplazo fallido sin perder documento, archivo o limpieza.
- PCM de referencia independiente mono/estéreo: cuatro BPM, uno/dos compases,
  dos políticas de cola y bloques 1/127/256/4096 (128 combinaciones). Incluye
  cuatro WAV de entrada, gain/pan/mute, selección de escena y filas con slot compartido.
- WAV real y metadatos; referencias exactas de frames/colas/notas, invariancia
  de partición, pool lleno y límites coincidentes a tasas 1/100 Hz. `finish`
  termina voces y no genera otra ocurrencia al comenzar la cola.
- Escena explícita frente a modo guardado, Keep/Stop desde silencio, assets
  usados ausentes, scene fuera de rango, fallo de sink y preservación de sesión viva.
- CLI sin hardware: v1/v2, traslado, escena predeterminada/expresa, bloque,
  errores de opciones/grafo y archivo existente sin sobrescribir.

Una prueba de validación detectó que variantes unitarias de Keep/Stop podían
aceptar campos sobrantes. Las variantes de esquema se expresan como objetos
vacíos estrictos; la regresión ahora rechaza ambos casos y conserva los mismos
bytes JSON válidos. Las siete pruebas anteriores de proyectos/precisión pasan.

Verificación final correcta:

- Dominio: dos casos nuevos correctos en debug/release.
- Aplicación: 13 unitarios (uno nuevo), cinco v2, cuatro de escenas y siete de
  persistencia/precisión anterior correctos en release; selección focal debug
  correcta y el resto de regresiones incluido en check completo.
- CLI: dos casos nuevos y dos anteriores correctos en release; casos nuevos
  también correctos en debug. Clippy de dominio/app/CLI con `-D warnings` correcto.
- `npm run check`: código 0, 2026-11-14 03:49:22–03:52:23 UTC (13 de noviembre
  en México). Bindings, tipos, lint, 63 frontend, formato, build web, nueve escenarios
  CLI, Rust headless/workspace, audio instrumentado, diagnósticos, memoria y Clippy.
  Las suites se solapan; no se suman ejecuciones como pruebas únicas.

Logs en `target/stage13-3b-20261113/`: `check.log`, `release-domain.log`,
`release-project.log` y `release-cli.log`. El
[manifiesto](evidencia-proyectos-escenas-13.3b.json) registra base Git, quince
archivos de fuente/configuración de este paso y 34 acumulados respecto a base.
De las veinte fuentes anteriores, diecinueve conservan sus hashes y Project
se amplía deliberadamente en `daw-domain/src/lib.rs`. Bindings intactos y enlaces
locales de los doce documentos revisados correctos. El check compila pruebas
desktop y build web; no equivale a un bundle completo de escritorio.

## Dependencias y continuidad

CLI añade dev-dependencies de `daw-domain` y `serde_json` ya presentes en el
workspace, para fixtures y pruebas reales. Cargo.lock cambia solo esas dos
aristas de daw-cli; no cambian versiones ni se descargan paquetes. Actualización
resuelta offline; verificaciones posteriores usan `--locked`.

Siguiente: **13.3.c, coordinación de aplicación**. Conectar selección de modo/
clips/escenas, preparación e instalación, solicitudes/recibos y estado observado;
conservar rollback ante cola llena, timeout/pérdida inconclusos y lifecycle.
Después: DTO/IPC/UI (d), aceptación local (e) y diagnósticos/cierre 13.4.

El DTO/editor actual conserva los campos v2 en el backend al abrir/guardar, pero
todavía presenta el patrón heredado; la reproducción launcher desde Application
se integra en c y su exposición visual en d. CLI/Rust ya pueden exportar escenas.
No hay nueva apertura de hardware, escucha, medición física ni CI publicada.
Sin commit/push. Los manifiestos previos conservan sus capturas históricas.
