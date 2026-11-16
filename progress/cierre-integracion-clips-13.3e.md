# Etapa 13.3.e — Aceptación integrada local de clips y escenas

Estado: **aceptada localmente**, 2026-11-16 (México).
Entrada: [desktop 13.3.d](etapa-13-desktop-clips.md).
13.1–13.3 quedan implementadas y verificadas dentro de su alcance local;
**la etapa 13 sigue abierta** para diagnósticos, medición nativa y entrega 13.4.

## Aceptación

Dos contratos Rust atraviesan los comandos registrados de Tauri, la aplicación,
preparación de WAV, servicio manual, motor, archivos y exportación. El único acceso
directo de la prueba al backend es instalar y avanzar la sesión manual; no se
expone ningún comando de diagnóstico nuevo a la ventana de producción.

- Doce sesiones: mono/estéreo × 95.5/137.25 BPM × bloques de 1/127/4096 frames.
  Cuatro WAV reales PCM16/float32, mono/estéreo; originales eliminados después
  del guardado, banco preparado todavía utilizable y copias portátiles disponibles.
- Escena inicial de cuatro filas, transición parcial al pulso, Keep/Stop,
  supersesión de una fila y edición después de aceptación. Las copias activas
  conservan el contenido aceptado; orígenes independientes en pasos globales 0/4.
- Referencia independiente frame a frame con límites racionales, pool de voces,
  muestras, rampa de 5 ms, gain/pan/mute y suma. Compara eventos/notas y voz exacta,
  límites, contadores, voces activas, silencio y medidores de cada bloque.
  El API manual no devuelve PCM: esta verificación no afirma comparar directamente
  toda su salida. El PCM completo se verifica en las exportaciones.
- Exportación durante reproducción usa la escena inicial guardada, aunque se
  haya lanzado otra escena. PCM, cabecera RIFF/float32 o extensible, duración y
  colas se comprueban con un lector y referencia independientes; tolerancia 1e-7.
  Audio, documento y recibo no cambian por exportar. Disparos manuales y notas
  programadas conservan contadores separados.
- Pause estabiliza el reloj y produce silencio; Stop reinicia posición y cancela
  pendientes. Cierre, preparación explícita, inicialización y dos reaperturas
  mantienen sesión vacía/detenida y no reproducen la escena inicial ni el historial.
- Proyecto v2 trasladado con sus assets relativos y originales eliminados:
  conserva patrón, mezcla, clips, escenas, modo y título. Exportación cut/finish
  en tres tamaños de bloque, sin inicializar audio. Archivo inválido y destino
  de exportación existente fallan preservando documento y bytes anteriores.
- Abrir con banco retenido devuelve Busy; inicializar secuenciador en modo Clips
  falla antes de consumirlo. Inicializar proyecto en modo Clips y pulsar Play sin
  lanzar explícitamente sigue dejando las filas vacías.

Cuatro casos nuevos frontend completan el recorrido abrir → preparar → inicializar
→ lanzar → guardar → exportar → cerrar/reabrir; conservan borradores y escena inicial
separada de la escena interpretada. Un timeout se reconcilia solo al consultar,
sin volver a encolar ni pulsar Play automáticamente. Dos variantes controlan una
consulta tardía después de un resultado exacto Processed o Rejected.

## Correcciones encontradas

1. Una consulta de estado iniciada antes de procesar el lanzamiento podía terminar
   después de observar su resultado y otro reporte más reciente. La interfaz
   sustituía el resultado confirmado por Unknown. Ahora conserva ese resultado
   para el mismo ID de servicio y sesión; un recibo distinto o el cierre no lo hereda.
2. Bajo carga del hilo de UI, el efecto de preparación podía programar otra
   consulta antes de que React completara su limpieza, aunque el último snapshot
   ya fuera final. Comprueba el estado observado síncrono antes de invocar o
   programar otra consulta. Pasan los casos existentes de cancelación y sustitución
   con una consulta anterior todavía en curso.

No cambia la semántica de audio, archivos, cola o callback. Las modificaciones
de producto están limitadas al hook desktop y al texto de exportación del panel,
que ahora menciona el modo/contenido seleccionado; no hay compensación ni reintento
automático de un lanzamiento.

## Verificación

- **Seis casos nuevos únicos:** dos contratos IPC y cuatro frontend. Los doce
  escenarios dentro del primer contrato son variantes, no doce tests adicionales.
  Total frontend: **72** (antes 68). Las dos regresiones de polling ya existían.
- `cargo test -p rust-daw-desktop --locked --test launcher_acceptance_contract`:
  dos correctos. Misma selección en `--release`: dos correctos.
- `npm run check`: código 0; bindings, tipos, lint, 72 frontend, formato, build web,
  nueve escenarios CLI, Rust headless/workspace, audio instrumentado, diagnósticos,
  memoria y Clippy con `-D warnings`. Los avisos previos de ts-rs sobre atributos
  Serde no alteran la validación Rust ni los bindings.
- `npm run desktop:build -- --debug --no-bundle`: código 0, después del check para
  conservar el ejecutable con assets embebidos. Sin instalador.
- WebView2 real: dos WAV reales cargados desde el editor, BPM offline, dos clips,
  escena inicial 7, guardado v2, descarte de preparación, traslado y reapertura
  sin originales, exportaciones cut/finish verificadas muestra por muestra,
  rechazo de sobrescritura y preparación de las copias trasladadas. Audio
  Uninitialized, cero callbacks y ninguna inicialización/escucha de hardware.
- Capturas revisadas en 1000/360 CSS px, sin desbordamiento de página ni excepciones
  WebView; aplicación cerrada normalmente.

Capturas [ancha](../docs/screenshots/stage13-integration-1000.png),
[estrecha](../docs/screenshots/stage13-integration-360.png) y
[metadatos de archivos/PCM](../docs/screenshots/stage13-integration.json).
Logs en `target/stage13-3e-20261116/`: `check.log`, `release-ipc.log`,
`desktop-build.log`, `preview.log` y `preview.json`.
[Manifiesto de evidencia](evidencia-integracion-clips-13.3e.json).

El manifiesto registra fuentes acumuladas y sus hashes crudos: 59 de las 62
heredadas intactas; App.test, el hook y el texto de ProjectPanel se amplían deliberadamente, junto con dos
archivos nuevos de aceptación/referencia. Bindings, lockfiles y dependencias sin
cambios. Los manifiestos anteriores son históricos. No hay commit/push ni CI nueva.

## Siguiente paso

**13.4.a: diagnósticos reproducibles de clips y escenas.** Preparar el benchmark y
protocolo de sesión nativa con WAV, transiciones, fases, tiempos y propiedad/memoria
identificados, sin confundir ensayos manuales con medición física.
Después: 13.4.b, matriz nativa release y escucha; 13.4.c, entrega/CI y cierre.
Timeline, piano roll y automatización continúan fuera de este incremento.
