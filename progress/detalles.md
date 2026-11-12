# Detalles de implementación

## Auditoría local de entrega 10.4.c — 2026-11-11 (México)

136 hashes/fingerprint coincidentes con check/build y matriz nativa. 21 artefactos
verificados, siete fases por cada uno de seis casos, doce sesiones/histogramas,
rutas/coeficientes/mute/reapertura y 24 benchmarks con memoria/teardown correctos.
Git: 88 fuentes idénticas, cuatro solo CRLF/LF, 28 modificadas y 16 nuevas frente
a HEAD 4dc2c6c. API oficial GitHub confirma main y CI anterior de ese SHA, que no
contiene mezcla local. [Auditoría](auditoria-cierre-mezclador-10.4c.md), manifiesto
y descripción/patch revisable de entrega preparados. Sin cambios de implementación,
tests/build/hardware repetidos o commit/push. Escucha real y entrega/CI propias
siguen pendientes; etapa 10 abierta.

## Medición nativa de mezcla 10.4.b — 2026-11-11 (México)

Seis casos y seis reaperturas mixed correctos en Speakers Realtek/WASAPI,
48000 Hz estéreo f32, entradas nuevas mono/estéreo y 0/1/16 voces iniciales.
Doce sesiones con cero asignaciones/frees/nonfinite/deadline misses medidos;
máximo 206300 ns, ratio 0.02063, callbacks 480–1056 frames. Trazas/inputs/binario
y fuentes conservados con hashes en [reporte](etapa-10-medicion-nativa-mezclador.md)
y manifiesto nuevo. Sin cambios de código frente a 10.4.a. La enumeración actual
solo mostró Speakers; Headphones previo ausente. Se preguntó por audibilidad,
sin respuesta registrada; no se atribuye la confirmación de etapa 9 a esta mezcla.
Etapa 10 abierta; pendientes escucha y auditoría/CI de revisión entregada 10.4.c.

## Diagnósticos de mezcla 10.4.a — 2026-11-11 (México)

Nuevos comandos mixer-bench/mixer-native-check, con fuente mezclada explícita,
cuatro assets/rutas/gain/pan/mute y controles aplicados observados. Matriz offline
24 casos a 2000 iteraciones, oráculo por muestra/niveles y control de cuatro frees
PCM fuera del render. Lifecycle con WAV real/manual, exact IDs, mute/pausa/rutas,
final natural y reapertura; compilación/test release offline y con soporte nativo.
[Reporte](etapa-10-diagnosticos-mezclador.md), manifiesto nuevo y protocolo nativo.
Sin cambios de DSP/servicio/app/desktop/dependencias. Sin hardware, escucha,
commit/push o CI nuevos. Siguiente: medición/escucha mezclada 10.4.b, luego cierre/CI
10.4.c; etapa 10 sigue abierta.

## Aceptación integrada 10.3.d — 2026-11-11 (México)

Tres pruebas nuevas de aplicación con cuatro WAV decodificados/eliminados antes
de instalar, oráculos racionales por canal/master, particiones 1/127/256/4096,
transporte/rutas/mute/reapertura, pool global y cancelación mono/estéreo.
Tres regresiones de UI; dos reproducen fallos previos: consulta inicial lenta
sobrescribe selección nueva y recibo offline afirma estado no inicializado después
de abrir. Correcciones acotadas al hook y texto. Ver
[aceptación local](cierre-integracion-mezclador-10.3d.md) y su manifiesto.
Sin cambios de producción Rust, dependencias, IPC/ACL/bindings, CLI o estilos.
No hay hardware/escucha/commit/push/CI nuevos. Siguiente: diagnósticos explícitos
de mezcla y verificación nativa 10.4; etapa 10 sigue abierta.

## Desktop de mezcla 10.3.c — 2026-11-11 (México)

Cuatro comandos nuevos y capability main acotada; DTOs/tipos generados conservan
IDs/frames/secuencias grandes como cadenas y right mono null. Cuatro tiras con
borradores gain/pan/mute, selección y observación separadas, rutas de 16 slots,
niveles de canal/master por ventana medida e inicialización mezclada explícita.
Pruebas de orden/errores/sesión perdida/navegador y backend manual por IPC;
contrato/guía/reporte con manifiesto propio. Ver
[reporte 10.3.c](etapa-10-desktop-mezclador.md).
Núcleo/servicio/app y dependencias sin cambios frente a 10.3.b. Sin escucha,
commit/push o CI hospedada nuevos. Siguiente 10.3.d; cierre 10 en 10.4.

## Coordinación de aplicación 10.3.b — 2026-11-11 (México)

Application devuelve MixerEdit con ajustes seleccionados y aceptación opcional:
offline cambia solo Project, en vivo confirma primero la cola y después el
documento; estado aplicado/rampas se consultan aparte. Errores/QueueFull no
mutan selección. Instalación mixed copia ajustes y tempo al consumir el banco
preparado, sin auto Play o relectura. Cierre conserva selección aceptada sin replay.

Nueve pruebas nuevas (ocho integración y una política), más extensión de worker
bloqueado para editar sin esperar a decodificación. Check completo y release
correctos; build y huella en [reporte](etapa-10-aplicacion-mezclador.md) y
[manifiesto](evidencia-aplicacion-mezclador-10.3b.json). Audio/domain/engine, locks,
DTO/IPC/ACL/TS y dependencias sin cambios respecto de 10.3.a. Sin hardware,
commit/push o CI hospedada nuevos. Siguiente 10.3.c; etapa 10 aún abierta.

Verificación final 10.3.b: check completo, aplicación release y build desktop
debug correctos. Huella de 131 archivos con cuatro cambios respecto de 10.3.a;
motor/dominio/audio y locks/bindings/workflow intactos. Formato y espacios
correctos; 471 destinos locales en 23 documentos sin ausentes.

## Servicio de mezcla 10.3.a — 2026-11-11

AudioHandle añade sesiones manual/native mezcladas y controles de canal/ruta
por la FIFO existente. ServiceSnapshot.mixer observa ajustes aplicados, rampas
y picos de todo el callback, agrupando chunks sin perder picos tempranos.
Capacidades 128/64 y presupuesto 32 preservados; solo Copy cruza las colas RT.
Prevalidación completa de salida/scratch y secuencias; ventanas vacías preservadas,
overflow/fallo señalizado y observaciones de mezcla limpiadas en fallo/cierre.

Se añaden 16 pruebas condicionales (seis servicio, cuatro realtime, tres adaptador,
tres lifecycle) y se amplía teardown legacy/mixed a propietario/trabajador.
Check completo y audio release correctos; build y huella propios en el
[reporte](etapa-10-sesiones-mezclador.md) y [manifiesto](evidencia-sesiones-mezclador-10.3a.json).
El fixture IPC solo agrega mixer: None al literal Rust; DTO/ACL/TS de producción
sin cambios. Sin pruebas físicas o CI hospedada nuevas. Siguiente 10.3.b;
etapa 10 continúa abierta. Las evidencias previas permanecen fechadas.

Verificación final 10.3.a: check completo, audio release con native/rt-diagnostics
y build desktop debug correctos. La prueba física sigue ignorada. Huella de
129 archivos con nueve cambios respecto de 10.2; locks/bindings/workflow intactos.
Formato y espacios correctos; 447 destinos locales en 21 documentos sin ausentes.

## Núcleo de mezclador 10.2 — 2026-11-11

Dominio añade MixerChannel/MixerSettings y ajustes propios de Project; el motor
instala la ruta nueva únicamente mediante with_mixed_sampler. Pool global de
16 voces, rutas capturadas al disparar, balance lineal sin boost, gain/mute y
pan con rampas, suma determinista, headroom 1/16 y picos de contribuciones reales.
Sampler raw comparte avance de cursores sin cambiar su salida; modos existentes
conservados. Sin cambios IPC/UI, dependencias, locks o workflow.

Se añaden 16 pruebas (dos dominio, doce integración, una interna, una instrumentada)
y se amplían regresiones de overflow existentes. Check completo correcto;
release y build identificados en el [reporte](etapa-10-mezclador-motor.md) y
[manifiesto](evidencia-mezclador-10.2.json). La memoria instrumentada verifica
100 ciclos sin asignación/liberación y destrucción de cuatro PCM fuera de render.
La evidencia de etapa 9 conserva su huella histórica. Sin CI hospedada o medición
nativa del mezclador nuevo. Siguiente 10.3; etapa 10 aún abierta.

Verificación final 10.2: check completo, suites dominio/motor release, memoria
instrumentada release y build desktop debug correctos, en ese orden. Huella de
125 archivos con nueve cambios respecto del snapshot 9.5.b; locks/bindings/workflow
intactos. Formato y espacios correctos; 418 destinos locales en 19 documentos
sin ausentes. No se atribuye a estos cambios la CI histórica de 4dc2c6c.

## Cierre de etapa 9 y contrato 10.1 — 2026-11-11

Tras corregir la conexión de audífonos, el usuario solicita otra repetición:
20:59:47–21:00:19 UTC, Headphones Realtek explícito, mono/estéreo, 16 voces y
ventanas de cinco segundos; ambas código 0, aserciones y memoria/tiempos correctos.
Confirma «Si se escucha». Se conserva el intento anterior no oído con su aclaración
de conexión y se vincula la confirmación exclusivamente al último par. No se
atribuyen continuidad acústica ni ausencia general de artefactos a esa frase.
[Cierre de etapa 9](cierre-etapa-9.md), manifiesto y seguimiento actualizados.

Se especifica [contrato 10.1](../docs/mixer-contract.md): cuatro canales sobre el
pool global de 16 voces, ruta por slot capturada al disparar, gain/pan/mute,
balance lineal sin boost, headroom 1/16, rampas y medidores sobre señal real.
Diseño documentado, sin DSP/IPC/UI de mezcla nuevos. Siguiente implementación
10.2; no se crean commits/push ni se atribuye CI propia a estos documentos.
Los registros siguientes conservan su estado histórico previo a la confirmación.

Verificación final del cierre/contrato: formato y `git diff --check` correctos,
380 destinos locales de enlaces en 17 documentos sin ausentes. Cambios de
producto inexistentes: se reutiliza CI/check/build de la implementación publicada;
las nuevas ejecuciones físicas y la observación humana están identificadas aparte.

## Repetición de escucha sampler — 2026-11-11

A petición del usuario, dos ejecuciones release consecutivas entre 20:54:01 y
20:54:34 UTC en Speakers (2- Realtek(R) Audio): entrada mono y estéreo, 16 voces,
ventana de cinco segundos, salida 48 kHz/estéreo y lifecycle/reapertura. Ambas
terminan con código 0 y aserciones correctas. Binario e inputs coinciden con los
hashes medidos; nuevos logs preservados en
`target/stage9-listening-2026-11-11T20-54-01-577Z/`. Evidencia añadida al
[manifiesto de auditoría](evidencia-cierre-9.5c.json) y al
[reporte](auditoria-cierre-9.5c.md). Confirmación de escucha humana solicitada,
pendiente; etapa 9 aún abierta. Sin cambios de producto ni nueva publicación.

## Auditoría de cierre sampler 9.5.c — 2026-11-11

API oficial de GitHub consultada a las 20:07:32 UTC: `main` remoto y HEAD en
`4dc2c6c`, run 42896571891 completed/success; headless-linux y desktop-windows
correctos, incluidos check/build y locks/bindings. Contrastados 120 archivos con
el manifiesto 9.5.b, huella agregada, seis logs nativos, dos WAV, cuatro logs de
check/build/release, piloto y binario medido. Cuatro blobs Git difieren solo en
finales de línea; no se modifican fuentes ni manifiestos históricos. Ver
[auditoría](auditoria-cierre-9.5c.md) y [evidencia](evidencia-cierre-9.5c.json).
Se actualiza estado/plan/README al SHA publicado. Confirmación de escucha
solicitada y pendiente; etapa 9 sigue abierta y etapa 10 aún no empieza.
Sin nuevas mediciones físicas, cambios de producto, commit o push.

Verificación documental: Prettier y `git diff --check` correctos; 282 destinos
locales de enlaces en diez documentos revisados, sin ausentes. Se revalidan
seis casos/doce sesiones, resultados por ID, sumas de histogramas y contadores
cero del manifiesto; reproducción estéreo y log de sobrescritura coincidentes.
Se reutilizan check/build locales y CI del código entregado; no se repiten suites
de producto por esta actualización documental.

## Mediciones nativas sampler 9.5.b — 2026-11-10

Seis casos release Realtek/WASAPI, 48 kHz/estéreo/f32, entradas mono/estéreo y
ventanas 0/1/16: correctos, incluidos rechazos por ID, gain/transporte, finalización
y reapertura. Cero asignaciones/liberaciones/non-finite/overruns medidos; mayor
tiempo 432100 ns (ratio 0.04321). Piloto inicial conservado. Runner añade trazas
de IDs/métricas de reapertura fuera de callback; generador estéreo reproduce bytes
y rechaza overwrite. Ver [reporte](etapa-9-medicion-nativa.md). Escucha humana
solicitada sin confirmación registrada; loopback/unplug/estrés/plataformas y CI
siguen pendientes. No hay commit/push ni cierre final de etapa 9.

## Revisión de documentación/configuración — 2026-11-10

README actualizado al commit 0c346d5 y siguientes etapas, sin atribuirle CI no
inspeccionada. Git ignore protege env locales, coverage y caché TS, conserva
plantillas y archivos necesarios; lint/formatter excluyen coverage. Coherencia de
toolchains/lockfiles, workspace, dev URL, CSP/ACL y CI revisada; 27 casos de ignore
correctos. Ver [auditoría](revision-readme-configuracion.md). Sigue 9.5.b; no cambia
código musical ni se abre hardware. Sin commit/push de esta revisión.

## Diagnósticos sampler 9.5.a — 2026-11-10

CLI añade sampler-bench y sampler-native-check, con offline-diagnostics sin CPAL
y release obligatorio. Banco/buffers/timing preasignados, oráculo estéreo distinto,
actividad 0/1/16 y control positivo de liberación PCM fuera de procesamiento.
Instrumentación opcional suma liberaciones, todos los canales e histograma fijo.
Runner nativo usa Application y dispositivo/WAV explícitos; su lifecycle se
verifica con WAV real y servicio manual, sin hardware. Benchmark release de 24
casos mono/estéreo correcto. Ver [reporte](etapa-9-diagnosticos-sampler.md).
Siguiente 9.5.b: mediciones/audición físicas. No hay commit/push o CI nuevos.

## Cierre local de integración 9.4.d — 2026-11-09

Auditoría de dieciséis criterios contra código/pruebas, con límites por capa.
Se corrigen dos defectos del hook: fallo de enumeración impedía consultar banco,
y cambios de efecto permitían solapar polls pendientes. Tres casos reproducen
fallo antes y pasan después; 24 pruebas frontend. No cambia audio/DTO/dependencias.
Se documenta protocolo 9.5: el runner sampler todavía falta y la telemetría actual
no expone liberaciones. Siguiente 9.5.a: benchmark offline y runner native opt-in,
con buffers acotados y tests sin hardware. Ver [cierre](cierre-integracion-9.4.md).
No hay hardware, capturas, commit/push o CI nuevos. Historial inferior conservado.

## Sampler en Application/IPC/UI 9.4.c — 2026-11-09

Application añade worker de carga/cancelación y entrega por ID a sesión; el
decoder no usa el mutex de aplicación ni controlador/RT. Cerrar audio precede
a cargar/instalar; fallo de apertura consume el banco. Cinco comandos nuevos,
ACL local main y DTOs/TS generados exponen preparación y telemetría sampler con
IDs/counters string y rechazo tipado. UI añade rutas WAV, carga/descarte, salida,
inicialización y disparo explícitos, master gain y resultado exacto/desconocido.
Ver [reporte actual](etapa-9-aplicacion-sampler.md). Siguiente: auditoría 9.4.d;
mediciones nativas/cierre siguen en 9.5. Los apartados inferiores son históricos.

## Sesiones/outcomes de sampler 9.4.b — 2026-11-09

AudioHandle añade sesiones manual/CPAL con banco propietario y trigger en FIFO
Copy compartida. ServiceSnapshot Rust observa último ID/result exacto y contadores
saturantes; DTO/IPC/UI siguen en 9.4.c. Native valida configuración exacta y todo
el bloque antes de chunking. Un guard cierra el stream y recupera Box<Realtime>
por buzón exclusivo de teardown para destruir PCM en propietario. Once pruebas
nuevas cubren sesión, límites, pérdidas de telemetría, fallo/reinicio, banco
expirado y retorno desde worker; release/Clippy correctos. Ver
[reporte actual](etapa-9-sesiones-sampler.md). Siguiente 9.4.c: Application, DTO,
bindings, IPC/ACL y UI con mocks, manteniendo IDs/counters como strings.

## Sampler en AudioEngine 9.4.a — 2026-11-09

Motor con banco propietario, headroom 1/16 y GainRamp master de 5 ms, transporte
y trigger_sample con resultado Copy tipado. Pause congela después de release;
Stop limpia voces sin liberar PCM. Errores conservan voces/envolvente/contadores.
Siete pruebas de integración, overflow raw interno, WAV → AudioEngine y medición
integrada sin asignaciones/liberaciones pasan debug/release; regresiones correctas.
Ver [reporte actual](etapa-9-integracion-motor.md). No cambia EngineCommand,
EngineSnapshot, DTO, bindings, dependencias o UI. Siguiente 9.4.b: sesiones
manual/CPAL y outcomes en cola. El manifiesto 9.3 se conserva como histórico.

## Cierre local de preparación 9.3.f — 2026-11-09

Auditoría de 9.3.a–e contra implementación, suites y matriz: se cierra
preparación 9.3 localmente. Se distingue entrega a un holder de banco de la
instalación en AudioEngine/servicio, pendiente de 9.4. Ver
[cierre y criterios](cierre-preparacion-9.3.md) y
[manifiesto de 110 archivos](evidencia-preparacion-9.3.json). La huella identifica
bytes locales, no un commit nuevo o CI remota.

Se reutilizan check/build correctos de 9.3.e (85 headless/90 workspace/ocho memoria,
15 audio instrumentadas/13 frontend/nueve escenarios CLI); no cambia código o
configuración. Se verifican documentación y hashes durante el cierre. Siguiente
9.4.a: modo sampler en AudioEngine con headroom 1/16, rampas/transporte y outcomes
tipados, primero sin dispositivo/IPC/UI. Etapa 9 completa sigue abierta.

## Fixtures de carga/reproducción 9.3.e — 2026-11-09

Cuatro pruebas nuevas recorren archivo WAV → banco → kernel Sampler: 18
combinaciones de encoding/header/canales sobre cuatro particiones, 65536 códigos
PCM16 a 44.1 kHz, banco vacío/completo y límites reales exactos/un exceso.
Los archivos se borran antes del render; oráculos enteros y muestras exactas
comprueban propiedad y reproducción raw. Debug/release y Clippy correctos;
ver [reporte actual](etapa-9-fixtures-integracion.md) para resultados completos
y [matriz de aceptación](../docs/sample-loading-acceptance.md) para criterios,
evidencia y límites. Solo pruebas/docs, sin cambios de producción o UI.
Siguiente 9.3.f: auditoría/cierre de preparación; nativo/servicio/IPC/UI en 9.4.

## Fallos, cancelación y cierre 9.3.d — 2026-11-09

Cuatro pruebas internas amplían cobertura con 17 fallos/cancelaciones del segundo
asset, sincronización antes/después del retorno y Read bloqueado, y prioridad de
errores por checkpoint. Se observan cierre en trabajador, Busy durante cierre,
banco activo conservado y permiso reutilizable. Tres pruebas instrumentadas nuevas
comprueban saldo de memoria en trabajador y receptor tras fallo/cancelación o
entrega/descarte. Focales debug/release y Clippy correctos; ver
[reporte actual](etapa-9-fallos-cancelacion.md) para resultados completos y límites.
Solo se amplía una costura privada del coordinador; API, motor, audio e IPC/UI
conservan su comportamiento. Siguiente 9.3.e: fixtures WAV → banco → Sampler offline.

## Preparación de banco 9.3.c — 2026-11-09

Coordinador síncrono en daw-app: valida slots antes de I/O y carga secuencialmente
un banco completo. Permiso compartido entre clones, mantenido por el resultado
hasta entrega/descarte, y presupuesto agregado comprobado antes de cada reserva.
Seis pruebas públicas, dos internas y dos instrumentadas nuevas pasan en debug
y release, incluida coexistencia de bancos estéreo de 128 MiB cada uno y limpieza
tras fallo en el octavo asset. Ver [reporte actual](etapa-9-preparacion-banco.md)
para check/build y alcance exacto de medición. No hay dependencias, DTO o UI nuevos.
Siguiente 9.3.d: matriz ampliada de fallo/cancelación/cierre; sampler nativo en 9.4.
Los registros de 9.3.b y anteriores se conservan como evidencia de cada incremento.

## Lector WAV 9.3.b — 2026-11-09

Adaptador estándar Rust en daw-app para preflight/decodificación individual;
sin dependencias ni cambios del motor/UI. Ocho pruebas, reserva fallida inyectada,
round-trip del exportador real y tres pruebas instrumentadas debug/release
verifican límites, cancelación y propiedad. check añade la suite de memoria del
CLI; ver [reporte actual](etapa-9-lectura-wav.md) para resultados finales y límites.
Se conservan los cambios locales de 9.3.a y auditoría. Siguiente 9.3.c: banco y
permiso/presupuesto agregado, sin atribuir a este código la CI histórica de 9.2.

## Contrato de carga 9.3.a — 2026-11-09

Se completa la especificación previa al loader: PCM16/float32 clásico/extensible,
16 MiB por archivo, hasta 4096 chunks, 64 KiB de buffers y presupuesto de una
preparación más banco activo. Se definen preflight, errores, cancelación cada
256 frames y entrega atómica del banco. Ver [reporte](etapa-9-contrato-carga.md)
y [contrato en inglés](../docs/sample-loading-contract.md). Se conservan las
ediciones locales de planificación anteriores. Solo documentación: no decoder,
garantía de memoria medida, nuevas dependencias ni cambios de UI. Siguiente 9.3.b.

## Revisión del plan y objetivos — 2026-11-08

Se contrasta 1002efa con código/evidencia de 7acc1db: etapas 1–8 y 9.1–9.2
completadas; carga/integración/cierre del sampler pendientes. Se define el
objetivo groovebox en 9–11, guardar/reabrir/exportar en 12 y clips en 13.
Se detallan 9.3.a–f, su aceptación y el presupuesto adicional aún por fijar,
separándolo del límite PCM implementado. Ver
[auditoría y siguiente etapa](estado-y-siguiente-etapa.md). Esta revisión actualiza
documentación y no implementa loader ni modifica el motor/UI.

## Estado actual — Etapa 9.2, 2026-11-08

Implementación publicada en 7acc1db88915e5fe1b0c17db4837fc3b197d35ad; ambos jobs
correctos en [run 42530699397](https://github.com/YoZaiKK/AudioMOTORR/actions/runs/42530699397),
inspeccionado el 2026-11-08 a las 22:40:03 UTC. El reporte registra enlaces de
jobs, 59 tests headless/64 workspace/15 audio instrumentados/13 frontend y nueve
escenarios CLI, build Windows y focales release. La actualización de cierre
documental posterior no cambia código, lockfiles, bindings ni capturas.

Banco inmutable de 16 slots y kernel de 16 voces implementados en daw-engine.
Reproduce PCM one-shot mono/estéreo, suma f32 por índice de voz, rechaza el nuevo
disparo al agotarse y conserva recursos al terminar/vaciar voces. Nueve pruebas
de comportamiento/overflow y una instrumentada cubren particiones, memoria y
destrucción. No se añade loader ni integración con AudioEngine/servicio/IPC/UI.
Ver [reporte actual](etapa-9-voces.md). Siguiente cambio: 9.3, preparación/carga y
decodificación acotadas fuera del procesamiento. Los registros inferiores son históricos.

## Etapa 9.1, 2026-11-08 — Registro histórico

Se inicia sampler con SampleBuffer inmutable dentro de daw-engine: PCM f32
normalizado, mono/estéreo, límite por frames y comprobación de rate/mapeo de salida.
Se añaden fixtures en memoria, cuatro tests y un contrato que fija banco/pool
de 16, rechazo del nuevo disparo al agotarse y propiedad de recursos.
No se añade reproducción, decodificador, UI/IPC ni dependencias. Ver
[reporte de etapa 9](etapa-9-sampler.md) y [contrato](../docs/sampler-contract.md).
El siguiente cambio es 9.2: banco fijo y kernel de voces con oráculos exactos.

La sección de etapa 8 siguiente conserva su estado al cierre, anterior a 9.1.

## Etapa 8, 2026-11-08 — Cierre histórico

Etapa 8 cerrada y publicada en 1e6bca8, con headless-linux y desktop-windows
correctos en run 42524752652 (SHA exacto inspeccionado el 2026-11-08).
Senoide nativa explícita, rampas de 5 ms y volumen integrados de motor a React;
silencio por defecto y WAV offline conservados. Ver [reporte completo](etapa-8.md)
para implementación, mediciones release, resultados de check/build/smoke, hashes,
límites y situación de publicación/CI del incremento. Se actualizan arquitectura,
contrato realtime, contrato nativo nuevo, README, guías y roadmap.

La instrumentación allocation-counter 0.8.1 es opcional; Cargo.lock y DTO/TS
cambian deliberadamente. Los snapshots SPSC siguen siendo Copy; los snapshots
de respuesta y el procesador manual se reservan al inicializar fuera del callback
para mantener acotado el tamaño de los enums, sin silenciar Clippy.
check prueba la variante instrumentada sin abrir hardware. El siguiente cambio
después del cierre es 9.1: contrato del sampler y fixtures en memoria.

Las secciones siguientes son el registro histórico de sus fechas; afirmaciones
anteriores de silencio o mediciones pendientes no describen la app de etapa 8.

Este archivo conserva el registro granular de cada etapa. El estado resumido se
mantiene en [checklist.md](checklist.md). Los requisitos originales permanecen en
[rust-daw-bootstrap-prompt.md](rust-daw-bootstrap-prompt.md).

Las secciones de etapas anteriores conservan el estado de Git y CI al tomar
cada evidencia. La revisión para publicación de las etapas 4–6 está al final;
el relevo vigente es [continuidad-etapa-6.md](continuidad-etapa-6.md).

## Etapa 1 — Preparar el proyecto

Estado: completada. Inicio: 2026-11-04. Cierre: 2026-11-05.

### Inspección y alcance

- El repositorio contenía únicamente `README.md`, `.git/` y el documento en
  `prompt/`; no existía una aplicación que conservar ni configuraciones de paquetes.
- No se encontraron `AGENTS.md` aplicables en el repositorio ni en sus directorios
  ascendentes inspeccionados.
- Se renombró `prompt/` a `progress/` conservando el documento original.
- Esta etapa prepara una aplicación compilable. El motor, dispositivos, transporte
  y contratos IPC completos corresponden a las siguientes etapas.
- El workspace empieza con `daw-app` y `rust-daw-desktop`, que tienen uso real.
  `daw-domain`, `daw-engine`, `daw-audio` y `daw-cli` se incorporarán cuando se
  implemente su responsabilidad en las etapas 2 y 3, sin paquetes vacíos.

### Decisiones iniciales

- Proyecto JavaScript en la raíz y adaptador desktop en `src-tauri/`.
- Workspace Cargo virtual; metadatos, dependencias y perfiles compartidos.
- Rust edition 2024, resolver 3 y versión Rust fijada.
- npm como único gestor JavaScript, versiones directas exactas y Node fijado.
- TypeScript 5.9.3 se selecciona por compatibilidad con `typescript-eslint`:
  la versión consultada del linter admite TypeScript menor que 6.1, mientras
  `typescript@latest` consultado devuelve 7.0.2.
- La pantalla inicial muestra el estado de preparación; no simula audio.

### Prerrequisitos observados

- Rust y Cargo instalados: 1.90.0, host `x86_64-pc-windows-msvc`.
- Node 22.23.4 y npm 10.9.9.
- Visual Studio 2022 Build Tools detectado mediante `vswhere` con el componente
  `Microsoft.VisualStudio.Component.VC.Tools.x86.x64`.
- WebView2 Runtime 156.0.4314.58 detectado en el registro.
- La primera consulta npm falló por permisos sobre su caché fuera del workspace;
  se reintentó mediante el mecanismo de aprobación del entorno.

### Referencias consultadas

- [Prerrequisitos de Tauri](https://v2.tauri.app/start/prerequisites/).
- [Configuración Tauri + Vite](https://v2.tauri.app/start/frontend/vite/).
- [Inicio de Vite](https://vite.dev/guide/).
- [Workspace Cargo](https://doc.rust-lang.org/cargo/reference/workspaces.html).
- [Tauri Rust](https://docs.rs/tauri/latest/tauri/).
- [Tauri build](https://docs.rs/tauri-build/latest/tauri_build/).
- Metadatos oficiales de paquetes obtenidos mediante `npm view`.

### Versiones instaladas

| Herramienta o paquete                     | Versión          |
| ----------------------------------------- | ---------------- |
| Rust / Cargo                              | 1.90.0           |
| Node / npm                                | 22.23.4 / 10.9.9 |
| React / React DOM                         | 19.3.1           |
| TypeScript                                | 5.9.3            |
| Vite / plugin React                       | 8.3.5 / 6.2.1    |
| Tauri Rust / tauri-build                  | 2.12.5 / 2.7.4   |
| Tauri API JavaScript / CLI                | 2.12.4 / 2.12.4  |
| ESLint / @eslint/js                       | 10.14.0 / 10.0.1 |
| typescript-eslint                         | 8.72.1           |
| eslint-plugin-react-hooks / react-refresh | 7.1.1 / 0.5.7    |
| Prettier                                  | 3.9.10           |
| @types/react / @types/react-dom           | 19.3.0 / 19.3.0  |
| @types/node                               | 22.20.5          |

Las versiones directas JavaScript son exactas en `package.json`. Las dependencias
Rust externas actuales están fijadas en el workspace; las transitivas quedan
registradas en el único `Cargo.lock`. No se necesita que las versiones de los
paquetes Rust y JavaScript de Tauri tengan el mismo número de parche.

### Cambios concretos

- React montado con StrictMode, estilos oscuros y pantalla de estado inicial.
- Vite en `127.0.0.1:1420`, puerto estricto y exclusión de `target/`, `crates/` y
  `src-tauri/` de sus watchers.
- Tauri inicializado con el CLI oficial, conservando `build.rs` e iconos.
- Se sustituyeron el identificador genérico y el código inicial del generador.
  No se mantuvo el plugin de logs porque todavía no tiene un uso necesario.
- `daw-app` proporciona información de aplicación sin depender de Tauri;
  desktop la utiliza para componer el título de ventana.
- CSP explícita para producción; CSP de desarrollo permite el servidor y HMR
  locales y los estilos inyectados de Vite. Capabilities básicas solo para `main`.
- `scripts/check.mjs` orquesta comprobaciones con Node y detiene la secuencia ante
  el primer error; `doctor.ps1` inspecciona sin instalar herramientas.
- `doctor` usa `rustup run` sin `--install` para evitar que la consulta de versiones
  instale una toolchain de forma implícita.
- No se crearon carpetas vacías, controles de transporte sin implementación,
  mocks de dispositivos, tests triviales ni generadores de bindings sin DTO reales.

### Verificaciones ejecutadas

| Comando                                                                                    | Resultado                                                                                                       |
| ------------------------------------------------------------------------------------------ | --------------------------------------------------------------------------------------------------------------- |
| `rustup toolchain install 1.90.0 --profile minimal --component rustfmt --component clippy` | Toolchain fijada instalada con autorización.                                                                    |
| `npm install`                                                                              | Dependencias y lockfile generados.                                                                              |
| `npm ci`                                                                                   | Instalación reproducible completada.                                                                            |
| `npm ls --depth=0`                                                                         | Dependencias directas instaladas sin conflictos.                                                                |
| `npm run doctor`                                                                           | Rust, Cargo, Node, npm, MSVC, SDK y WebView2 detectados.                                                        |
| `npm run typecheck`                                                                        | Correcto.                                                                                                       |
| `npm run lint`                                                                             | Correcto tras importar explícitamente `node:process` en el orquestador.                                         |
| `npm run build`                                                                            | Correcto: TypeScript y bundle de producción de Vite.                                                            |
| `cargo generate-lockfile`                                                                  | Correcto: resolución de 406 paquetes compatibles con Rust 1.90.                                                 |
| `npm audit --omit=dev`                                                                     | Sin vulnerabilidades reportadas en dependencias de producción.                                                  |
| `npm run desktop:build -- --debug --no-bundle`                                             | Correcto: ejecutable en `target/debug/rust-daw-desktop.exe`, sin instalador.                                    |
| `npm run dev` y petición HTTP a `http://127.0.0.1:1420`                                    | HTTP 200 con el contenedor React y la entrada `src/main.tsx`; servidor detenido después. No es un smoke visual. |

La secuencia `npm run check` terminó correctamente, incluido formato frontend y
Rust, typecheck, lint, build frontend y
`cargo clippy --workspace --all-targets --locked -- -D warnings`.

### Incidencias y límites

- Cargo no pudo conectar a crates.io dentro del entorno restringido por un error
  TLS de Schannel (`SEC_E_NO_CREDENTIALS`). Con autorización fuera del sandbox se
  generó el lockfile y comenzó la descarga/compilación nativa.
- Se reemplazó ESLint 9 por ESLint 10 después de que npm señalara que la versión
  inicial ya no tenía soporte.
- La primera compilación desktop detectó una colisión de nombres de archivos PDB
  entre el binario y la biblioteca en Windows. Se dio a la biblioteca el nombre
  `rust_daw_desktop_lib` para mantener distintos sus artefactos.
- `npm audit` completo reporta 10 avisos moderados en herramientas de desarrollo,
  principalmente su árbol de `minimatch`. Las correcciones automáticas propuestas
  incluyen retrocesos de versiones; no se aplicó `audit fix --force` ni un override
  de versión mayor sin verificar compatibilidad. Queda registrado para revisión
  en la etapa de comprobaciones. El árbol de producción no presenta avisos.
- La presencia de WebView2 en el registro no prueba que una ventana se haya
  renderizado. El smoke visual y de audio se registra por separado en la etapa 5.
- Linux/macOS, audio real, motor, transporte, pruebas de comportamiento y contratos
  IPC todavía no están implementados ni verificados.

## Etapa 2 — Construir el motor

Estado: completada. Inicio y cierre: 2026-11-05.

### Cambios y decisiones

- Se incorporaron `daw-domain` y `daw-engine` al workspace y a sus miembros por
  defecto. El motor depende únicamente del dominio; no se añadieron dependencias
  externas. Cargo.lock registra los dos paquetes locales nuevos.
- `ProjectId` encapsula un `u64` no nulo proporcionado por el llamador, sin promesa
  de unicidad global. `Project` valida y recorta el título. El documento inicial
  usa ID 1, «Proyecto sin título» y 120 BPM.
- `Tempo` solo permite valores finitos de 20 a 300 BPM inclusive. Sus campos y
  los de configuración permanecen privados para conservar los invariantes.
- `EngineConfig` acepta frecuencia `u32` y canales `u16` positivos. El render de
  silencio admite esos valores; la compatibilidad con hardware se comprobará
  mediante el backend en la siguiente etapa.
- `AudioEngine` comienza detenido, recibe configuración y tempo explícitos y
  conserva su estado separado del documento editable. `EngineSnapshot` es Copy.
- `render_interleaved` escribe cero en todo el buffer del llamador y acepta
  tamaños variables alineados a frames, incluido el buffer vacío. No asigna,
  bloquea, hace I/O ni formatea errores. Un error de alineación o desbordamiento
  silencia el buffer sin modificar el estado; los incrementos son comprobados.
- El contador del dispositivo avanza siempre que se procesa un bloque válido;
  la posición del transporte solo avanza en reproducción. Pause conserva la
  posición, Play reanuda y Stop reinicia únicamente la posición del transporte.
- Los comandos Copy se aplican explícitamente entre renders. Cambiar el tempo
  conserva los contadores. Las colas, Accepted/Applied y el backend corresponden
  a la etapa 3; las notas futuras requerirán offsets dentro del bloque.
- `npm run check` incorpora las pruebas de dominio y motor. README documenta
  el contrato y los comandos para verificar únicamente estos paquetes.
- Se actualizó el texto de la pantalla inicial a etapa 2 y al siguiente paso:
  conectar el servicio de audio. No se añadieron controles nativos todavía.

### Verificaciones ejecutadas

- `cargo test -p daw-domain -p daw-engine --offline`: correcto; ocho pruebas
  cubren documento, BPM, silencio, configuración, particiones, transporte,
  separación documento/motor, buffers inválidos y desbordamiento de contadores.
- `cargo clippy -p daw-domain -p daw-engine --all-targets --locked --offline -- -D warnings`:
  correcto, sin warnings.
- `npm run check` (ejecutado con `npm.cmd` desde PowerShell): correcto; typecheck,
  lint, formato, build frontend, formato Rust, ocho pruebas del núcleo y Clippy de
  todo el workspace con `--locked` y warnings como errores. La primera ejecución
  detectó formato pendiente en el texto actualizado de App.tsx; se aplicó Prettier
  y la secuencia completa posterior terminó con código cero.
- `git diff --check`: correcto para los archivos ya rastreados; el formato de los
  archivos nuevos queda cubierto por Prettier y rustfmt en la secuencia anterior.

### Límites

- El motor se verifica sin dispositivo, ventana ni Tauri. No hay audio nativo ni
  controles de transporte conectados a React todavía.
- La ausencia de asignaciones se establece por inspección del camino de render;
  no se ejecutó instrumentación del allocator ni medición de plazos de tiempo real.
- No se verificó compilación en Linux/macOS ni funcionamiento con hardware.

## Etapa 3 — Conectar el audio

Estado: completada. Inicio y cierre: 2026-11-06.

### Implementación y decisiones

- Nuevos paquetes `daw-audio` y `daw-cli`, incorporados al workspace y sus miembros
  por defecto. `daw-app` coordina proyecto y audio sin depender de Tauri. Se
  reexporta `Tempo` desde el motor para sus consumidores; no cambia su validación.
- Dependencias fijadas: CPAL 0.18.2 y rtrb 0.4.1, consultadas en sus APIs oficiales.
  CPAL requiere Rust 1.85 y es compatible con la toolchain 1.90 del proyecto.
  `native-audio` es opcional y se propaga explícitamente por audio, app y CLI;
  desktop la habilita. No se añadieron Serde ni contratos IPC al motor.
- Hilo de control propietario del stream y de las operaciones CPAL, incluida su
  destrucción. Handles clonables admiten llamadas concurrentes mediante un canal
  externo acotado a 32 solicitudes. Cola llena devuelve Busy; respuesta con plazo
  de 10 segundos. Se rechazan solicitudes que vencieron antes de ejecutarse.
- Si una operación nativa ya está ejecutándose al vencer el plazo, puede terminar
  después; se debe consultar estado antes de reintentar. El cierre no espera un
  acknowledgement del callback, pero depende del retorno de las llamadas del driver.
- Cola SPSC de comandos con 128 entradas Copy; FIFO y máximo de 32 comandos por
  callback completo. QueueFull rechaza explícitamente sin consumir commandId.
- Segunda cola SPSC con 64 snapshots Copy. El callback intenta publicar una vez
  por bloque; si no cabe, el siguiente callback vuelve a publicar el estado más
  reciente, con acknowledgement acumulativo. El control drena hasta 64 snapshots
  por iteración y consulta su canal cada 5 ms.
- La aceptación devuelve un ID monotónico. El snapshot confirma el último ID
  aplicado, callbacks, configuración, contadores y estado observado. Los snapshots
  del servicio tienen secuencia monotónica; los nativos incluyen dispositivo y
  formato reales. Los enteros siguen siendo u64: su conversión JSON a strings
  corresponde a la etapa 4.
- Backend manual sin timer ni dispositivo: procesa exactamente 0–4096 frames
  por solicitud, usando el mismo procesador de comandos y motor. Cero frames es
  una operación vacía; no aplica comandos ni incrementa callbacks. El llamador
  divide duraciones largas en bloques. El reporte comprueba el silencio del buffer
  real, inicialmente rellenado con NaN para detectar muestras no sobrescritas.
- Backend nativo enumera IDs, salidas predeterminadas y configuraciones soportadas.
  Inicializa la salida predeterminada o el ID exacto solicitado, con su configuración
  predeterminada soportada. Cambiar salida requiere shutdown e initialize; no hay
  fallback, hot-swap, ASIO ni selección manual de sample rate/buffer en esta etapa.
- Ambos backends admiten 1–32 canales. Conversión nativa de f32, f64 y enteros con
  y sin signo de 8/16/32/64 bits. Formatos restantes, incluidos los de 24 bits,
  devuelven UnsupportedSampleFormat. El silencio unsigned usa su punto medio.
- Scratch nativo preasignado de 256 frames por canal; callbacks mayores se procesan
  por fragmentos alineados sin ampliar memoria. Los comandos se procesan una sola
  vez por callback. El callback de error solo señala un flag atómico; el control
  forma el error y cierra el stream. No se conserva el mensaje original de CPAL.
- Shutdown es idempotente y descarta comandos pendientes explícitamente al cerrar
  la sesión. Reiniciar crea motor detenido en cero con el tempo del documento;
  IDs de comandos y secuencia de observación siguen creciendo dentro del servicio.
  El Drop del propietario cierra el servicio y une el hilo incluso si hay handles.
- `Application` expone resumen del proyecto, dispositivos, inicialización/cierre,
  snapshots y transporte. Tempo inválido se rechaza antes de enviar; el documento
  cambia tras aceptación, y el playback cambia tras aplicación.
- CLI: smoke offline con frames o segundos enteros, bloque de 1–4096 frames y
  comprobación de silencio, contadores y Play aplicado. También devices y live-smoke
  opcionales: este último verifica callbacks y Play/Pause/Stop aplicados, con cierre.
- README conservado en inglés, nuevo `docs/audio-service.md` con ownership,
  límites y políticas; pantalla inicial actualizada sin conectar controles todavía.
- `npm run check` amplía pruebas y Clippy headless a los cinco paquetes, y añade
  pruebas de todo el workspace. Las pruebas automáticas no abren dispositivos.

### Verificaciones ejecutadas

- `cargo fetch`: el intento dentro del sandbox falló por TLS Schannel
  SEC_E_NO_CREDENTIALS. La descarga autorizada fuera del sandbox completó el
  lockfile y las dependencias; las compilaciones posteriores usan --locked/offline.
- `cargo test -p daw-domain -p daw-engine -p daw-audio -p daw-app -p daw-cli --no-default-features --locked --offline`:
  correcto, 22 pruebas. Cubren además colas, telemetría llena, aceptación/aplicación,
  concurrencia, cierre/reinicio, handles tras Drop, error de stream simulado,
  solicitudes vencidas, CLI inválida y particiones 1/127/256/4096.
- Clippy headless y con native-audio de los paquetes audio/app/CLI: correctos con
  warnings tratados como errores. Se corrigieron diferencias de API de rtrb y un
  import no usado durante las primeras compilaciones.
- `devices` con native-audio enumeró Speakers (2- Realtek(R) Audio), WASAPI,
  salida predeterminada estéreo, con rangos y formatos reportados por el driver.
- `live-smoke --seconds 2`: correcto; 200 callbacks y 96576 frames al observar
  Play; Pause y Stop confirmados antes del cierre. Configuración 48000 Hz, 2 canales.
- `live-smoke --seconds 1 --device <ID enumerado>`: correcto; 101 callbacks y
  49056 frames. La salida volvió a abrir y cerró correctamente por ID explícito.
- Tras añadir metadatos de salida al snapshot, live-smoke confirmó el formato f32,
  100 callbacks, 48576 frames y Pause/Stop aplicados. Los contadores de hardware
  dependen de los callbacks realmente recibidos; no son duraciones exactas offline.
- Prueba nativa `native_failure_recovery_and_restart_on_same_service`, ejecutada
  explícitamente con --ignored: correcta. Verifica error DeviceAbsent con ID
  inexistente, recuperación, Play/Pause/Stop, cierre, reapertura por ID en el mismo
  servicio, IDs crecientes y Drop mientras reproduce. Permanece ignorada en CI y
  en pruebas normales porque abre hardware real.
- `npm run check` (mediante npm.cmd): correcto; TypeScript, ESLint, Prettier,
  build Vite, rustfmt, 22 pruebas headless, Clippy headless, pruebas del workspace
  con audio nativo (22 correctas y una de hardware ignorada por defecto) y Clippy
  del workspace con --locked y -D warnings.
- `npm run desktop:build -- --debug --no-bundle`: correcto; ejecutable generado en
  `target/debug/rust-daw-desktop.exe` con la feature nativa habilitada, sin instalador.
- `git diff --check`: correcto. El formato de archivos nuevos se verifica mediante
  rustfmt y Prettier. No se realizaron commits ni publicación.

### Límites y validaciones pendientes

- No se abrió una ventana para verificar controles: React sigue sin IPC ni audio
  conectado, que corresponden a la etapa 4. No se adelantaron sus DTO ni eventos.
- Solo se verificó hardware WASAPI estéreo f32 a 48 kHz. Otros formatos tienen
  conversión implementada y prueba de silencio unsigned, sin verificación física.
- Desconexión física del dispositivo, Linux/macOS, instrumentación del allocator,
  carga de CPU y plazos de tiempo real permanecen sin verificar. Procesar silencio
  no demuestra ausencia de dropouts ni continuidad audible.

## Etapa 4 — Conectar la interfaz

Estado actual: completada y verificada.
Fecha: 2026-11-06. El registro de pausa siguiente es histórico; los resultados
actuales están en «Implementación después de reanudar».

### Publicación de la etapa anterior

- Se creó el commit `69467ef97698b291fa764acee3d7cfd52d5a412e`,
  `feat(audio): add manual and native audio service with smoke CLI`.
- Push completado a `origin/main` en
  `https://github.com/YoZaiKK/AudioMOTORR.git`.
- El sandbox denegó escritura del índice Git y la conexión a GitHub falló por TLS
  Schannel. Commit y push se completaron mediante las autorizaciones del entorno.
- La nota de la etapa 3 sobre ausencia de commits describe su cierre inicial de
  verificación; la publicación ocurrió después, al comenzar esta etapa.

### Cambios locales al pausar

- `Cargo.toml`: declaraciones compartidas de Serde 1.0.229, serde_json 1.0.154 y
  ts-rs 12.0.1. Las dos primeras versiones ya existían transitivamente en el lock.
- `src-tauri/Cargo.toml`: dependencias de DTO/bindings y feature `test` de Tauri
  como dependencia de desarrollo. Falta resolver ts-rs en Cargo.lock y verificar
  su compatibilidad real con la toolchain.
- `crates/daw-app/src/lib.rs`: reexport de `TransportState`, para que los DTO usen
  el contrato público de aplicación sin añadir dependencia directa del adaptador
  al motor.
- Nuevos `src-tauri/src/ipc/{mod,commands,dto,state}.rs`: borradores de los diez
  comandos, errores estructurados, tipos IPC, conversión de u64 a strings y estado
  compartido. Las operaciones de aplicación se plantean con spawn_blocking.
- Nuevo `src-tauri/src/bin/bindings.rs`: borrador de generador y comprobador de
  declaraciones en `src/shared/api/generated/api.ts`. Ese archivo aún no existe.
- `package.json` y `package-lock.json`: instaladas Vitest 5.1.0,
  @testing-library/react 16.3.3 y jsdom 31.0.0 con versiones directas exactas.
  Se inspeccionaron sus package.json en node_modules para confirmar instalación.
  Todavía no hay configuración, tests ni script `test:frontend`.

### Punto exacto de interrupción y verificación

- Se intentó `cargo fetch` tras declarar ts-rs; falló dentro del sandbox por
  `SEC_E_NO_CREDENTIALS`. El turno se interrumpió durante el reintento con
  autorización. Al inspeccionar el estado tras la interrupción, Cargo.lock no
  contiene ts-rs ni ts-rs-macros y no tiene cambios respecto al commit publicado.
  No se afirma que esa descarga haya terminado.
- `src-tauri/src/lib.rs` conserva el arranque de la etapa 3: no declara `pub mod ipc`
  ni usa `ipc::configure`, por lo que los borradores no están integrados.
- El nuevo binario de bindings referencia precisamente ese módulo no expuesto;
  debe corregirse la integración antes de intentar afirmar que desktop compila.
- No se ejecutaron compilación, tests, generación de bindings, formato completo ni
  integración IPC para esta etapa. Las verificaciones satisfactorias de la etapa 3
  siguen siendo la última evidencia de funcionamiento completo.
- React, CSS, CSP, capabilities, build.rs y scripts de verificación siguen sin
  implementar la etapa 4. Tampoco existe publicador de eventos engine-snapshot.
- No se observaron procesos Cargo ni procesos npm de esta tarea todavía ejecutándose
  al revisar la pausa; no se detuvieron procesos ajenos.

### Punto de reanudación registrado al pausar (histórico)

Al pausar se dejó el inventario y orden de trabajo en retomar-etapa-4.md, con los
cambios sin publicar y las casillas abiertas. El usuario solicitó después continuar.

### Implementación después de reanudar

- Descarga de ts-rs 12.0.1 completada mediante cargo fetch autorizado; Cargo.lock
  actualizado. Compilación correcta con Rust 1.90 y DTO Serde/ts-rs fuera del motor.
- Los diez comandos quedaron conectados al Builder real mediante ipc::configure.
  DesktopState conserva Application y serializa su acceso; las llamadas que pueden
  esperar al servicio se ejecutan con spawn_blocking, fuera del hilo de interfaz.
- DTO camelCase, errores code/message/details y strings decimales para IDs,
  secuencias, frames y callbacks. El motor permanece sin Serde ni IPC.
- Binario bindings y scripts bindings:generate/check: salida única en
  src/shared/api/generated/api.ts, comparación exacta sin reescribir. Se excluye
  esa salida de Prettier para conservar generación determinista; ESLint y TypeScript
  siguen comprobándola. default-run fija el binario principal al añadir el generador.
- Publicador nativo de engine-snapshot hacia main: espera al menos 67 ms entre
  observaciones, omite snapshots repetidos y ciclos con estado ocupado, y termina
  si main desaparece. Cerrar main o salir señaliza el publicador, une su hilo y
  destruye Application/servicio. No hay llamadas IPC desde el callback de audio.
- React separa DTO observados, datos de proyecto y estado visual mediante
  useDawSession. desktop-client centraliza invoke/listen. Paneles de dispositivos,
  apertura/cierre, transporte, BPM y estado técnico; errores visibles y controles
  deshabilitados sin servicio. La inicialización solo ocurre tras clic explícito.
- La suscripción se registra antes de consultar el estado inicial. Comparación
  mediante BigInt de secuencias e IDs, descarte de eventos viejos y limpieza incluso
  si el registro asíncrono termina después del desmontaje. Se contempla StrictMode.
- Aceptar un comando muestra pendiente hasta su aplicación. También se contemplan
  snapshots aplicados o de error que llegan antes de la respuesta de aceptación.
- En navegador se muestra «Vista de interfaz: requiere Tauri para audio» sin
  llamadas nativas ni dispositivos ficticios. Nota visible de salida de silencio.
- ACL de aplicación generado por build.rs a partir de una lista de comandos
  compartida; permisos limitados a main local y listen/unlisten. Los artefactos
  de permisos están ignorados en Git y formato. CSP explícita conservada; el smoke
  real confirma que permite la aplicación empaquetada sin deshabilitar seguridad.
- Vitest/jsdom configurados, tests React y cliente añadidos a npm run check junto
  con bindings:check. El peer @testing-library/dom está instalado transitivamente.
- Nuevo smoke opcional desktop-smoke.ps1/mjs: abre el ejecutable, controla WebView2
  vía CDP local 9223, verifica la UI real con hardware, toma captura y cierra la
  ventana. No se ejecuta en la suite automática ni modifica configuración global.

### Incidencias resueltas al verificar

- El primer ejecutable de tests que ejercita Tauri falló con STATUS_ENTRYPOINT_NOT_FOUND.
  Se identificó la importación de TaskDialogIndirect desde comctl32, que requiere
  Common Controls v6. Se pasó a tests de integración y se añadió manifiesto MSVC
  solo a esos ejecutables; Tauri sigue generando los manifiestos de los binarios.
  La primera prueba con flags globales provocó un recurso duplicado y fue corregida.
- El mock runtime necesita acceso a rutas de datos de la aplicación fuera del
  workspace. Dentro del sandbox falló con Access is denied; al ejecutar mediante
  autorización del entorno pasaron las pruebas. No se sustituyeron por mocks de IPC
  que eludan el handler o la evaluación de permisos.
- Se corrigieron imports explícitos de globals Node en el smoke para pasar ESLint.

### Verificaciones actuales

- typecheck: correcto. Once pruebas frontend correctas: modo navegador, errores de
  apertura, estado aceptado/aplicado, carreras de aceptación, secuencias mayores a
  Number.MAX_SAFE_INTEGER, StrictMode, limpieza real y contrato del cliente IPC.
- cargo test -p rust-daw-desktop --locked --offline, autorizado: cuatro pruebas
  de integración correctas. Handler real, argumentos y errores; precisión JSON;
  rechazo desde otra ventana; tasa de publicación y parada del worker.
- cargo clippy -p rust-daw-desktop --all-targets --locked --offline -- -D warnings:
  correcto en la comprobación individual anterior al cierre general.
- bindings:check: correcto; generación original completada con el binario bindings.
- desktop:build -- --debug --no-bundle: correcto con la interfaz nueva empaquetada.
- desktop-smoke.ps1 autorizado: correcto con WebView2 real y Realtek WASAPI f32,
  48 kHz y 2 canales. Inicializar, Play, 137.5 BPM, Pause, Stop (posición cero),
  shutdown, reinicializar conservando tempo y salida normal de la ventana. Primera
  corrida: 40 callbacks observados y último comando aplicado 4. Captura revisada
  visualmente en target/desktop-stage4.png. El smoke se amplió para esperar también
  callbacks observados en la UI tras reiniciar antes de tomar la captura.
- npm run check completo, ejecutado con autorización del entorno: correcto.
  Bindings, TypeScript, lint, once tests frontend, formato, build Vite, rustfmt,
  22 tests headless, Clippy headless, 26 tests Rust del workspace y Clippy de todo
  el workspace con -D warnings. La prueba nativa de hardware de etapa 3 permanece
  ignorada en la suite automática como estaba previsto.
- Se regeneraron los bindings y se comparó su SHA-256 antes/después: idéntico.
- Tras el check se volvió a empaquetar desktop debug sin instalador. Cargo test
  puede reconstruir el binario sin custom-protocol, que usa devUrl; un intento de
  smoke intermedio no encontró el destino empaquetado. Recompilar con Tauri CLI
  antes de ejecutar el smoke resolvió el problema. Usar el orden check, build
  desktop y smoke, sin comandos Cargo intermedios que sustituyan ese ejecutable.
- Smoke final ampliado: correcto; error DeviceAbsent por ID inexistente a través
  del IPC real, error visible en React y recuperación; luego Play/Pause/Stop,
  BPM 137.5, shutdown/restart con callbacks observados en UI y cierre normal de
  ventana. 44 callbacks y último ID aplicado 4 en la observación previa al cierre.
  Captura actualizada en target/desktop-stage4.png.

### Límites que permanecen

- Salida de silencio; no se afirma continuidad audible ni ausencia de dropouts.
- Linux/macOS, desconexión física de hardware y medición de deadlines sin verificar.
- La suite IPC usa mock runtime; el smoke real se registra aparte. No hay CI aún.
- No se ejecutó todavía instalador ni release. Esta reanudación no crea commit/push;
  los cambios quedan para revisión local, sobre el commit publicado de etapa 3.

## Etapa 5 — Verificación y guías prácticas

Estado: completada en Windows, 2026-11-06.

- Motor: nueva prueba de particiones 1/127/256/4096 con Play/Pause/Stop y cambio
  de tempo en límites equivalentes: 3228 frames de dispositivo y posición final 257.
- Servicio: FIFO con comandos mezclados al cruzar el límite de 32 por callback;
  Pause conserva posición hasta aplicar Stop en el callback siguiente.
- Aplicación: cierre con comandos aceptados pendientes; se descartan al cerrar,
  pero el documento conserva 137.5 BPM para el reinicio detenido, con IDs crecientes.
- React: apertura fallida seguida de recuperación y transporte habilitado sin Play
  automático. Se conserva toda la cobertura anterior de carreras y StrictMode.
- howToRunThis: índice, cinco guías españolas, fixtures JSON y ejecutor Node sin
  dependencias nuevas. Nueve casos CLI incluidos en npm run check. Los archivos
  son escenarios, no proyectos musicales importables. README principal en inglés.
- Smoke real con -UpdateDocs: siete PNG versionables y metadata.json con fecha,
  plataforma y snapshots. El modo normal no los modifica. Se comprueba el puerto
  9223 antes de abrir la ventana y se mantiene el cierre/restauración del entorno.

### Verificaciones

- npm ci autorizado: correcto; 218 paquetes instalados y diez avisos moderados.
  No se aplicó audit fix ni se realizó una auditoría detallada adicional.
- npm run check autorizado: correcto tras corregir el import URL del ejecutor.
  Doce tests frontend, 25 headless y 29 Rust del workspace, nueve fixtures CLI,
  bindings, TypeScript, lint, formato, build y Clippy con -D warnings. Las suites
  comparten pruebas; la prueba nativa ignorada sigue fuera del check normal.
- Hashes SHA-256 de Cargo.lock, package-lock.json y api.ts antes de npm ci y después
  de regeneración/check/build: sin cambios. Valores en howToRunThis/resultados.md.
- Build desktop debug sin instalador: correcto después del check.
- Smoke -UpdateDocs autorizado: correcto con WebView2 real y Realtek WASAPI f32,
  48000 Hz, dos canales. Error DeviceAbsent y recuperación, Play/137.5 BPM/Pause/Stop,
  cierre/reapertura con callbacks y salida normal de ventana. Se observaron 67
  callbacks y último ID aplicado 4 antes de reiniciar.
- Siete capturas inspeccionadas visualmente: sin inicializar, error, reproduciendo,
  pausado, detenido, reiniciado y cerrado. La etiqueta Etapa 4 de la UI identifica
  el hito de interfaz validado; esta etapa no cambia funciones de producción.

### Continuidad y límites

Sin pruebas Linux/macOS, desconexión física, otros formatos de hardware, medición
del allocator/deadlines o continuidad audible. No se añadió CI ni instalador ni se
crearon commits/push adicionales. Se conserva el trabajo previo de etapa 4.
Continuar con etapa 6. Guías y reporte: ../howToRunThis/README.md.

## Etapa 6 — Documentación técnica y CI

Estado: completada localmente en Windows, 2026-11-06. Evidencia y auditoría final
en [cierre-etapa-6.md](cierre-etapa-6.md); siguiente trabajo en
[continuidad-etapa-6.md](continuidad-etapa-6.md).

### Archivos y decisiones

- docs/architecture.md: dependencias y secuencia React/IPC/aplicación/servicio/
  callback en Mermaid; documento editable, reproducción, observación y cierre.
- docs/realtime-contract.md consolida audio-service.md; el anterior queda como
  enlace compatible. Se actualizan las referencias obsoletas de etapa 3 y se
  incorporan strings JSON, snapshots/errores desktop, publicación y cierre.
- docs/adr/0001-core-and-desktop-boundaries.md registra motor independiente,
  backend manual, CPAL opcional, owner del stream, SPSC, strings y DTO generados.
- docs/development.md distingue Windows verificado, mock runtime, build y hardware;
  Linux/macOS previstos sin verificación local. docs/roadmap.md ordena los siete
  hitos con responsabilidades y aceptación. README y guías enlazan estos documentos.
- .github/workflows/ci.yml: push a main, PR y manual, contents: read y cancelación
  por workflow/ref. Ubuntu 24.04 usa paquetes explícitos sin CPAL/Tauri; Windows
  Server 2022 usa npm ci, regeneración/diff de bindings, check y build debug sin
  instalador. Diffs finales protegen lockfiles/tipos incluso tras fallo.
- checkout v4.2.2 y setup-node v4.4.0 fijados por SHA completo; verificados mediante
  git ls-remote sobre repositorios oficiales. No hay acciones de terceros, caché
  Cargo, smoke de hardware, modificación de capturas, firma, artefactos o releases.

### Comandos y resultados

- Rust 1.90.0, Node 22.23.4 y npm 10.9.9 ya instalados; no se cambiaron versiones.
  npm run doctor confirma MSVC 14.44.35207, SDK 10.0.26100.0 y WebView2 156.0.4314.58.
- npm ci: correcto, 218 paquetes; informa 13 avisos moderados en esta ejecución
  (etapa 5 registró diez). No se ejecutó audit fix ni se modificaron dependencias.
- actionlint 1.7.7: workflow válido, sin diagnósticos de sintaxis o expresiones.
  Descarga oficial comprobada contra checksum SHA-256 publicado, alojada solo en
  target/stage6-tools. La consulta git y descarga requirieron autorización de red
  del entorno tras fallar dentro del sandbox; no se cambió configuración global.
- npm run bindings:generate: correcto. Hash de api.ts idéntico al inicial.
- npm run check: correcto dentro del entorno disponible; 12 pruebas frontend,
  25 headless y 29 Rust del workspace, nueve escenarios CLI; bindings, typecheck,
  lint, formato, build Vite, rustfmt y Clippy headless/workspace con -D warnings.
  La prueba nativa de hardware sigue ignorada por defecto. Las suites se solapan.
- npm run desktop:build -- --debug --no-bundle: correcto después del check.
  Ejecutable target/debug/rust-daw-desktop.exe, sin instalador. Sin smoke real nuevo.
- Comprobación de destinos locales Markdown/PNG: correcta; ver cantidad final
  en el reporte de cierre. Prettier y actionlint se repiten tras actualizar el registro.
- SHA-256 de Cargo.lock, package-lock.json y api.ts: idénticos antes/después de
  npm ci, regeneración, check y build. Valores completos en cierre-etapa-6.md.

### Cierre y límites

Las seis etapas quedan cerradas en el checklist; esta etapa documenta y automatiza
la arquitectura existente sin añadir código musical ni alterar evidencia previa.
No se crearon commits/push ni se ejecutó GitHub Actions con estos cambios.
Workflow creado/validado localmente no equivale a ejecución remota satisfactoria.
Linux/macOS, otros formatos/dispositivos, desconexión física, allocator, deadlines
y continuidad/carga en release siguen pendientes. No se atribuyen al commit
publicado los cambios locales de etapas 4–6.

Siguiente cambio concreto: oscilador senoidal offline de baja amplitud dentro de
daw-engine, fase continua y pruebas por particiones; después WAV fuera del callback.
No implementado durante esta etapa.

## Revisión de documentación y entrega de etapas 4–6

Fecha: 2026-11-06. Solicitud posterior del usuario: comprobar toda la documentación
y hacer commit/push del último estado completo de la aplicación.

- Revisados README, documentos técnicos, guías howToRunThis y seguimiento contra
  scripts, configuración y contratos implementados. Se conserva el prompt original.
- README, desarrollo, índice de guías, pruebas, checklist y continuidad dejan de
  presentar las etapas 4–6 como trabajo pendiente de publicación. El historial Git
  identifica la revisión entregada y Actions informa el resultado remoto.
- Reportes de etapas 4/5/6 marcados como históricos cuando describen estado previo
  de Git/CI. Se conservan las pruebas, hashes y capturas fechadas.
- Comandos de check descritos en el orden real, incluidos los nueve escenarios CLI;
  conteos actuales separados de los históricos y suites solapadas identificadas.
- Se mantienen pendientes oscilador, mediciones de tiempo real, desconexión física
  y plataformas sin verificación. Ninguna comprobación local acredita éxito de CI remota.

La entrega incluye la UI/IPC, pruebas, bindings, guías/capturas, documentación
técnica y workflow existentes; no añade funciones musicales. El asunto del commit
es `feat(desktop): complete UI, verification guides, and CI`. Consultar el historial
Git y los [logs de Actions](https://github.com/YoZaiKK/AudioMOTORR/actions/workflows/ci.yml)
para identificar la revisión publicada y su validación hospedada.

Verificación repetida antes del commit:

- npm run check: correcto; 12 tests frontend, 25 headless, 29 Rust workspace y
  nueve escenarios CLI, además de bindings, TypeScript, ESLint, Prettier, build,
  rustfmt y Clippy. Test de hardware ignorado por defecto; sin smoke/capturas nuevos.
- npm run desktop:build -- --debug --no-bundle: correcto después del check.
- 108 destinos locales de enlaces Markdown/imágenes correctos y actionlint 1.7.7
  sin diagnósticos. git diff --check sin errores de espacios.
- Hashes de ambos lockfiles y api.ts idénticos a los del cierre de etapa 6.
- git fetch origin completado con autorización del entorno para escribir en .git;
  HEAD y origin/main coincidían antes de crear esta entrega. No se fuerza el push.

## Planificación posterior — Etapas 7–13

Fecha: 2026-11-07. Base: commit 1ef9bd0, publicado en main. Se modifica solo
documentación; oscilador, WAV y cambios de UI/audio siguen sin implementar.

- Nuevo progress/plan-etapas-7-13.md: etapas numeradas sin renumerar la base 1–6,
  dependencias y aceptación; etapa 7 desglosada en 7.0–7.5.
- Nuevo docs/offline-oscillator.md: contrato propuesto, separado del implementado.
  Señal opt-in con fase continua, 440 Hz/0.1/48 kHz estéreo por defecto de export,
  Pause conserva fase y Stop reinicia; silencio por defecto permanece intacto.
- Exportación prevista: daw-app coordina un motor offline con memoria acotada;
  CLI escribe WAV float32. No se confunde render_manual (reporte) con entrega de
  muestras, ni se duplica DSP en un exportador.
- Roadmap inglés actualizado con etapas 7–13 y criterios; checklist español
  distingue planificación terminada de implementación pendiente. README, continuidad,
  arquitectura, contrato actual, desarrollo y guía práctica enlazan el nuevo plan.
- Secuencia: revisar CI de la base; contrato/API mínima; oscilador y pruebas;
  coordinación/WAV; integración de checks/docs; cierre con evidencia. Después
  nativo/rampas, sampler, mezcla, pasos, persistencia/export y clips/escenas.
- La planificación no inspecciona ni afirma resultado remoto de Actions, no abre
  hardware y no repite mediciones. Conserva reportes, hashes y capturas históricas.

Siguiente paso: ejecutar 7.0 y, con la base confirmada, empezar 7.2 conforme al
contrato de 7.1. No iniciar implementación musical durante esta petición de plan.

Verificación de esta actualización documental:

- npm run format:check: correcto tras formatear los documentos modificados.
- Comprobador Node de enlaces: 130 destinos locales Markdown/imágenes existentes.
- git diff --check: correcto; cambios limitados a documentación y seguimiento.
- No se ejecutan suites de producto ni build nuevos por este cambio exclusivo
  de Markdown. La evidencia de pruebas de la base permanece en los reportes fechados.

## Etapa 7 — Base CI y oscilador offline (7.0–7.2)

Fecha: 2026-11-07. [Reporte del incremento](etapa-7-oscilador.md).

- Se consultó la API oficial de GitHub: run 42314334447, SHA completo 1ef9bd0,
  conclusión success en ambos jobs (headless-linux y desktop-windows). La
  ejecución fue del 2026-11-06; su inspección es posterior. No se atribuye ese
  resultado a las modificaciones locales nuevas.
- Implementado módulo interno oscillator.rs y constructor AudioEngine::with_sine
  con errores tipados. Fase continua por frame, validación, Pause/Stop y BPM
  independiente. El constructor original y todas las rutas app/CLI siguen en silencio.
- Seis tests nuevos y ampliación del overflow con comprobación de fase. Tests
  del motor y Clippy pasan; se conserva ausencia de dependencias externas en core.
- Contratos, arquitectura, roadmap, README, guías, checklist y continuidad
  actualizados para distinguir motor implementado y exportación pendiente.
- Elección de biblioteca WAV diferida a 7.3; no se adelantan I/O, UI, DTO,
  comandos CLI ni reproducción nativa. Etapa 7 no cerrada; siguiente trabajo 7.3.

Verificación final: npm run check correcto, con 12 tests frontend, 31 Rust headless,
35 Rust workspace y nueve escenarios CLI; Clippy sin warnings. Build desktop
debug sin instalador correcto después del check. El reporte distingue suites
solapadas, hardware ignorado y validaciones pendientes. Lockfiles y api.ts conservan
sus hashes. Formato/enlaces documentales y git diff --check correctos. Se mantiene
la evidencia histórica de etapas previas, incluidas capturas y hashes.

## Etapa 7 — Exportación offline WAV (7.3)

Fecha: 2026-11-07. [Reporte de exportación](etapa-7-exportacion.md).

- Implementada API daw-app::offline::OfflineRender: motor propio, validación,
  buffer reutilizado máximo 32 KiB y entrega a destino falible tras cada render.
  No crea Application/servicio, no modifica el documento ni el transporte vivo.
- Nuevo comando CLI export-wav con ruta/frames obligatorios y opciones para
  sample rate, canales, frecuencia, amplitud y bloque. Mono/estéreo float32,
  bloque final exacto y cero frames. La rama corre antes de Application::new.
- hound 3.5.1 fijado solo en CLI, un paquete añadido a Cargo.lock con su checksum.
  APIs/límites revisados contra documentación/código de la versión fijada.
  Descarga autorizada tras fallo Schannel dentro del sandbox. El cambio de hash
  de Cargo.lock es deliberado; package-lock y bindings no cambian.
- Protección create_new, validación RIFF/byte rate previa, finalize explícito y
  cierre del escritor con errores propagados. Un fallo tras crear archivo puede
  dejarlo parcial; se informa ruta, sin sobrescribir/eliminar/reintentar.
- Diez tests nuevos: cuatro de aplicación, cuatro CLI de round-trip/errores y dos
  unitarios del adaptador de archivos. Comparan datos con el motor, inspeccionan
  metadatos/bytes RIFF y fallos de escritura/seek/flush con cierre. Check y ambos
  jobs descubren automáticamente estos tests; nueve escenarios Node conservados.
- Contrato, arquitectura, README, guías y seguimiento actualizados con API/comando
  reales y límites. Reporte 7.2 conservado como histórico. Sin cambios de UI,
  servicio nativo, DTO, hardware o capturas. Etapa 7.5 y CI nueva pendientes.

Verificación final: check correcto (12 frontend, 41 headless y 45 workspace,
nueve escenarios CLI; suites Rust solapadas) y build desktop debug sin instalador
correcto después del check. WAV de ejemplo de 48001 frames reabierto y contrastado
independientemente con Node, metadatos/muestras correctos. Formato/enlaces y
git diff --check correctos. Nuevo hash Cargo.lock fijado y estable durante las
comprobaciones; package-lock/api.ts intactos. Valores y límites en el reporte.

## Cierre y publicación de etapa 7 (7.5)

Fecha: 2026-11-07. [Auditoría final](cierre-etapa-7.md).

- Revisión final sin cambios funcionales adicionales. Se repitió check completo:
  12 frontend, 41 Rust headless y 45 Rust workspace, más nueve escenarios CLI;
  suites Rust solapadas, hardware ignorado por defecto. Build Windows debug
  sin instalador correcto después del check.
- Commit c6cf62703bcd8207030d6f87d38e51d0804d18e1 publicado en origin/main:
  feat(engine): add offline sine rendering and WAV export. Veintinueve archivos
  incluyen implementación, pruebas, contrato, guías y progreso de la etapa.
- Run 42419963899, SHA exacto c6cf627: completed/success en headless-linux y
  desktop-windows, comprobado en API oficial a las 22:27:29 UTC. El 504 temporal
  del observador se resolvió reintentando; no fue fallo de CI.
- Con esa evidencia se cierran 7.4/7.5, se actualizan README/roadmap/contratos y
  continuidad. El commit documental posterior registra el cierre; la CI de esa
  revisión final se comprueba aparte. No se inicia código de etapa 8 aquí.
- Permanecen límites de Linux nativo/macOS, desconexión física, formatos de
  hardware adicionales, allocator/deadlines y continuidad en release. Desktop
  y smoke conservan silencio; export-wav es la ruta de señal offline.

Siguiente cambio: etapa 8, señal por CPAL, rampas y controles explícitos,
con comparación offline y mediciones de hardware documentadas.
