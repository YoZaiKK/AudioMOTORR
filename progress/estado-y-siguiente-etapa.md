# Estado verificado, objetivos y siguiente etapa

Revisión: 2026-11-11. **Núcleo de mezcla 10.2 implementado localmente**, con
[reporte](etapa-10-mezclador-motor.md) y [huella propia](evidencia-mezclador-10.2.json).
**Servicio 10.3.a implementado localmente**, con [reporte](etapa-10-sesiones-mezclador.md)
y [huella propia](evidencia-sesiones-mezclador-10.3a.json). **Aplicación 10.3.b implementada localmente**, con
[reporte](etapa-10-aplicacion-mezclador.md) y [huella propia](evidencia-aplicacion-mezclador-10.3b.json).
**Desktop 10.3.c implementado localmente**, con [reporte](etapa-10-desktop-mezclador.md)
y [huella propia](evidencia-desktop-mezclador-10.3c.json). [Aceptación 10.3.d completada localmente](cierre-integracion-mezclador-10.3d.md),
con [huella propia](evidencia-integracion-mezclador-10.3d.json). [Diagnósticos 10.4.a implementados localmente](etapa-10-diagnosticos-mezclador.md),
con [huella propia](evidencia-diagnosticos-mezclador-10.4a.json). [Matriz nativa 10.4.b medida](etapa-10-medicion-nativa-mezclador.md): seis casos y seis reaperturas Speakers Realtek;
[evidencia](evidencia-nativa-mezclador-10.4b.json). [Auditoría local 10.4.c completada](auditoria-cierre-mezclador-10.4c.md), con
[manifiesto](evidencia-auditoria-mezclador-10.4c.json) y [entrega preparada](entrega-mezclador-etapa-10.md).
[Publicación 10.4.c](publicacion-mezclador-10.4c.md): implementación `e7f543f` entregada en main y CI Linux/Windows correcta.
**Etapa 10 cerrada dentro de su alcance:** usuario confirmó «Se escucha» tras la repetición del mezclador.
[Cierre](cierre-etapa-10.md) y [evidencia](evidencia-cierre-mezclador-10.json).
**Siguiente: 11.1**, contrato y modelo del patrón/secuenciador de 16 pasos. La implementación de sampler publicada en `4dc2c6c` incluye trazas del
runner y generador de fixture 9.5.b. En la auditoría histórica, la API de GitHub confirmó ese SHA en main y ambos
jobs correctos en [run 42896571891](https://github.com/YoZaiKK/AudioMOTORR/actions/runs/42896571891).
La [auditoría 9.5.c](auditoria-cierre-9.5c.md) verificó al cerrar el sampler los 120 archivos medidos y
los artefactos guardados. El usuario confirmó audibilidad tras la última repetición
con audífonos; [etapa 9 cerrada](cierre-etapa-9.md) dentro del alcance registrado. La implementación 10.2–10.4.a ya está entregada en e7f543f con CI propia;
la huella de etapa 9 se conserva como histórica y ya no describe el motor nuevo.
El kernel previo 7acc1db conserva su [CI Linux/Windows correcta](https://github.com/YoZaiKK/AudioMOTORR/actions/runs/42530699397),
que no se atribuye a la revisión nueva. Ver [auditoría de configuración](revision-readme-configuracion.md).
Se contrastan código, pruebas y reportes existentes junto a seis mediciones
nativas Realtek nuevas; no se atribuye la CI de un SHA a otro. El orden original de
los hitos se conserva en el [documento inicial](rust-daw-bootstrap-prompt.md).

## Objetivos y estado real

| Objetivo                               | Qué está hecho                                                                                                   | Qué falta                                                                                                                           |
| -------------------------------------- | ---------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------- |
| Fundamento de escritorio/audio         | Etapas 1–6: workspace, motor separado, backend manual/CPAL, aplicación, IPC/DTO, UI, cierre, documentación y CI. | Validaciones físicas/plataformas adicionales, registradas como límites.                                                             |
| Generar una señal verificable          | Etapas 7–8: senoide offline/WAV, salida nativa explícita, volumen y rampas.                                      | Loopback acústico, unplug físico, estrés prolongado y otras configuraciones físicas.                                                |
| Reproducir samples en motor            | PCM/banco/preparación, AudioEngine, sesiones manual/CPAL, aplicación/IPC/UI y diagnósticos.                      | Etapa 9 cerrada: mediciones, auditoría, CI de 4dc2c6c y audibilidad confirmada; límites físicos conservados.                        |
| Groovebox de cuatro canales y 16 pasos | Base y kernel disponibles.                                                                                       | Núcleo/servicio/aplicación/desktop hasta 10.3.d aceptados localmente; etapa 10 cerrada con escucha y CI; siguiente secuenciador 11. |
| Guardar, reabrir y exportar un beat    | La CLI exporta una senoide WAV.                                                                                  | Etapa 12: documento versionado, assets relativos y exportación del beat completo.                                                   |
| Clips y escenas                        | Alcance documentado.                                                                                             | Etapa 13: modelo, lanzamiento cuantizado, transiciones y UI. Timeline/piano roll/automatización siguen después.                     |

El objetivo de groovebox capaz de ejecutar un patrón se alcanza al cerrar 11; guardar,
reabrir y exportar ese beat corresponde a 12. Son límites de producto, no
funciones actuales ni fechas de entrega.

Las 16 voces son capacidad de reproducción simultánea. No equivalen a cuatro
canales musicales de mezcla ni a 16 pasos de un patrón; esas funciones se
implementan en 10 y 11. Mono/estéreo describe los canales PCM de entrada/salida.

## Evidencia contrastada

| Área                 | Evidencia                                                                                                                              | Resultado                                                                                                                                                  |
| -------------------- | -------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------- |
| PCM y voces          | [sample.rs](../crates/daw-engine/src/sample.rs), [sampler.rs](../crates/daw-engine/src/sampler.rs), suites de recursos/kernel/memoria. | Kernel completado: muestras desde memoria, sumas deterministas y recursos retenidos.                                                                       |
| AudioEngine/servicio | [audio](../crates/daw-audio/src/service.rs) y [reporte 9.4.b](etapa-9-sesiones-sampler.md).                                            | Sesiones manual/CPAL y FIFO/outcomes implementados con pruebas de adaptador sin hardware. UI/DTOs exponen sampler desde 9.4.c, con pruebas mock.           |
| IPC/UI               | [comandos](../src-tauri/src/ipc/command_names.rs) y [panel](../src/features/audio/AudioPanel.tsx).                                     | Veinte comandos; sampler y mezcla explícitos, cuatro tiras, rutas y niveles; selección/recibo/estado aplicado diferenciados.                               |
| Documento            | [daw-domain](../crates/daw-domain/src/lib.rs).                                                                                         | Project contiene ID, título, BPM y ajustes validados de cuatro canales/rutas; sin patrones, assets persistidos o clips.                                    |
| WAV                  | [export.rs](../crates/daw-cli/src/export.rs) y OfflineRender.                                                                          | Exporta senoide, no el proyecto/sampler ni un beat.                                                                                                        |
| Carga WAV y banco    | [sample_loading.rs](../crates/daw-app/src/sample_loading.rs) y [cierre 9.3](cierre-preparacion-9.3.md).                                | Preparación cerrada localmente, con evidencia/manifiesto. Entrega a holder; Application 9.4.c mueve luego el banco a la sesión, sin I/O en controlador/RT. |
| Verificación previa  | [Reporte 9.2](etapa-9-voces.md), código 7acc1db y jobs enlazados.                                                                      | 59 tests headless, 64 workspace, 15 audio instrumentados, 13 frontend y nueve escenarios CLI; suites solapadas. Build y focales release registrados.       |

El reporte de 9.3.c conserva su check de 77 pruebas headless/82 workspace y build.
9.3.d añade cuatro pruebas internas y tres instrumentadas de fallos/cancelación
y cierre entre hilos. Su check correcto registra 81 headless, 86 workspace, ocho de
memoria decoder/banco, 15 audio instrumentadas, 13 frontend y nueve escenarios
CLI, con build Windows correcto; su reporte registra alcance y resultados.
9.3.e añade cuatro pruebas de carga/reproducción completas y una matriz técnica
de aceptación. Su check correcto registra 85 headless, 90 workspace, ocho de memoria
decoder/banco, 15 audio instrumentadas, 13 frontend y nueve escenarios CLI;
build Windows correcto. Su reporte registra alcance y resultados.
9.4.a integra AudioEngine offline. Su check correcto registra 94 headless, 99
workspace, 16 audio instrumentadas, ocho memoria decoder/banco, 13 frontend y
nueve escenarios CLI; build Windows correcto. La huella nueva registra 111 archivos.
9.4.b conecta sesiones/outcomes y cierre de PCM en propietario. Su check registra:
101 headless, 108 workspace, 26 audio instrumentadas, ocho memoria decoder/banco,
13 frontend y nueve CLI; build Windows correcto. Manifiesto de 112 archivos.
Las suites se solapan. La CI enlazada arriba corresponde al kernel publicado,
no a estos cambios locales.

9.4.c conecta Application/DTO/IPC/UI con worker de carga/cancelación y resultados
por ID exacto. Check/build Windows correctos: 106 headless, 112 workspace,
26 audio instrumentadas, ocho memoria decoder/banco, 21 frontend y nueve CLI.
El [manifiesto actual](evidencia-desktop-9.4c.json) registra 116 archivos; ver
[reporte](etapa-9-aplicacion-sampler.md). 9.4.d audita/cierra localmente y añade tres regresiones de UI; ver
[cierre de integración](cierre-integracion-9.4.md). Mediciones 9.5.b ya registradas;
quedan escucha humana y consolidación 9.5.c.

La ejecución nueva de 9.4.d pasa check/build: 106 headless, 112 workspace,
26 audio, ocho memoria, 24 frontend y nueve CLI. El
[manifiesto actual](evidencia-integracion-9.4d.json) identifica 116 archivos;
los anteriores conservan sus snapshots, sin atribuirles esta nueva ejecución.

9.5.a entrega diagnósticos y benchmark offline. Check/build Windows correctos:
108 headless/114 workspace/29 audio/nueve diagnósticos CLI/ocho memoria/24 frontend/
nueve escenarios. Release y 24 casos de benchmark sin hardware correctos. El
[manifiesto actual 9.5.a](evidencia-diagnosticos-9.5a.json) registra 119 archivos;
[reporte](etapa-9-diagnosticos-sampler.md). Seis mediciones nativas 9.5.b ya correctas; ver [reporte](etapa-9-medicion-nativa.md). Escucha/9.5.c pendientes.

No se detecta una etapa musical marcada como completada sin implementación.
Debe conservarse la distinción entre **kernel de sampler completado** y
**sampler de app implementado con mocks, aún sin audición física verificada**. Linux/macOS nativos y las
mediciones físicas no realizadas siguen como limitaciones explícitas.

## Plan por etapas

| Etapa   | Entrega                                              | Estado / aceptación                                                                                                                                                                                                                                                              |
| ------- | ---------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1–6     | Fundamento, audio, UI, pruebas, documentación y CI.  | Completadas dentro de su alcance registrado.                                                                                                                                                                                                                                     |
| 7       | Oscilador offline y WAV.                             | Completada; señal, frames/metadatos y errores verificados.                                                                                                                                                                                                                       |
| 8       | Oscilador nativo, controles y rampas.                | Completada; evidencia Windows y límites físicos registrados.                                                                                                                                                                                                                     |
| 9.1–9.2 | Recursos PCM, banco y voces.                         | Completadas; código 7acc1db con CI correcta.                                                                                                                                                                                                                                     |
| 9.3     | Preparar un banco desde WAV fuera del procesamiento. | Cerrada localmente en 9.3.f; implementación incluida en 4dc2c6c con CI correcta. Manifiesto histórico de 110 archivos conservado.                                                                                                                                                |
| 9.4     | Sampler en AudioEngine/CPAL y en la UI.              | 9.4.a–d implementados/auditados localmente con pruebas offline/adaptador/mock. Evidencia física queda en 9.5.                                                                                                                                                                    |
| 9.5     | Verificar y cerrar el sampler completo.              | Completada: mediciones, check/build, auditoría y CI de 4dc2c6c correctos; audibilidad confirmada por el usuario.                                                                                                                                                                 |
| 10      | Mezclador de cuatro canales.                         | Núcleo, servicio, aplicación y desktop implementados localmente; integración aceptada en 10.3.d; diagnósticos 10.4.a listos; matriz nativa 10.4.b correcta; implementación e7f543f entregada y CI correcta; escucha confirmada y etapa 10 cerrada dentro del alcance registrado. |
| 11      | Secuenciador de 16 pasos.                            | Pendiente; eventos dentro del bloque, BPM fraccional, transporte y control de deriva.                                                                                                                                                                                            |
| 12      | Guardar/abrir y exportar el beat.                    | Pendiente; versionado, assets relativos, round-trip y WAV consistente.                                                                                                                                                                                                           |
| 13      | Clips y escenas.                                     | Pendiente; lanzamiento cuantizado y transiciones deterministas.                                                                                                                                                                                                                  |

No se empieza mezclador/secuenciador antes de cerrar sampler. Las mediciones
de senoide y las pruebas con mock no sustituyen las pruebas nativas del sampler.

Las referencias inferiores a publicación/CI pendiente describen el estado local
de cada incremento fechado. Su implementación ya está incluida en `4dc2c6c`, cuya
CI se inspeccionó en esta auditoría; no se reescriben los manifiestos históricos.

## Historial de preparación e integración — Estado anterior al cierre de etapa 9

El estado actual y la confirmación auditiva se registran en el
[cierre de etapa 9](cierre-etapa-9.md). El desglose siguiente conserva sus hitos locales.

**Resultado:** suministrar WAV admitidos y obtener un SampleBank validado,
listo para integración posterior. 9.4.c incorpora los controles sampler a la UI;
guardar proyectos sigue en 12.

| Paso  | Trabajo                                                        | Aceptación                                                                                                                                        |
| ----- | -------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------- |
| 9.3.a | Contrato especificado el 2026-11-09.                           | Formatos y presupuestos fijados; límites individuales aplicados en 9.3.b y agregados en 9.3.c, con memoria debug/release.                         |
| 9.3.b | Lector/decoder individual implementado localmente.             | Formatos/límites, cancelación, reserva exacta y propiedad verificados; 9.3.c añade coordinación del banco.                                        |
| 9.3.c | Coordinador implementado localmente.                           | Validación previa a I/O, carga ordenada, permiso de preparación/resultado, aritmética agregada y coexistencia real de bancos máximos verificadas. |
| 9.3.d | Fallos/cancelación/cierre verificados localmente.              | 17 casos en segundo asset, entrega/Read sincronizados, cierre de lectores y memoria en trabajador/receptor correctos debug/release.               |
| 9.3.e | Fixtures de extremo a extremo y matriz verificadas localmente. | 18 combinaciones × cuatro particiones, todos los códigos PCM16, bancos y límites reales; cuatro pruebas debug/release correctas.                  |
| 9.3.f | Auditoría y cierre local completados.                          | Criterios contrastados; check/build 9.3.e y manifiesto de código/configuración. Publicación/CI y etapa 9 completa siguen pendientes.              |

9.3.a está especificada en el [contrato de carga](../docs/sample-loading-contract.md)
y [reporte](etapa-9-contrato-carga.md): PCM16/float32 clásico/extensible, 16 MiB por
archivo, 4096 chunks, 64 KiB de buffers y 8 MiB extra para finalización. Preparación
máxima de datos: 136 MiB + 64 KiB; con banco activo: 264 MiB + 64 KiB. 9.3.c
aplica los límites agregados y prueba coexistencia de bancos máximos con el
contador de asignaciones. usage reporta techo conservador, no pico medido ni RSS.
9.3.b verifica la parte de archivo/recurso individual.
Se conservan los límites PCM ya implementados (1048576 frames/recurso y hasta
128 MiB/banco). PCM24/32, comprimidos y conversión de rate quedan fuera del
contrato inicial; cancelación cooperativa cada 256 frames y antes de entregar.

daw-app coordina política/preparación; un adaptador concreto lee/decodifica;
daw-engine recibe únicamente PCM validado. El decoder 9.3.b usa la biblioteca
estándar, sin dependencias nuevas o acceso a archivos dentro del motor.
9.3.b implementa el adaptador individual: formatos/límites por archivo, reserva
exacta y cancelación, con pruebas debug/release y round-trip del exportador real.
9.3.c añade coordinador de banco, permiso compartido hasta entrega/descarte y
presupuesto agregado. 9.3.d verifica fallos/cancelación, cierre y transferencia
entre hilos. 9.3.e verifica fixtures WAV → banco → Sampler offline,
oráculos de muestras/particiones y matriz de formatos/límites. 9.3.f cierra
preparación localmente con evidencia y limitaciones explícitas; ver
[cierre de preparación](cierre-preparacion-9.3.md). 9.4.a integra AudioEngine con
headroom 1/16, envolvente/transporte y resultados Copy, con pruebas offline;
9.4.b conecta sesiones manual/CPAL, propiedad/cierre del banco y outcomes en cola.
9.4.c añade worker de preparación/cancelación, aplicación/DTO/IPC/ACL y UI con
mocks, IDs/counters string y resultado solo para su command ID. Ver
[reporte de aplicación](etapa-9-aplicacion-sampler.md). 9.4.d cierra la
[aceptación integrada](../docs/sampler-integration-acceptance.md) localmente y
corrige enumeración fallida/polls solapados. 9.5.a implementa benchmark offline,
runner sampler opt-in y métricas de liberaciones/todos los canales/histograma. Ver
[reporte actual](etapa-9-diagnosticos-sampler.md). 9.5.b registra seis casos nativos y reaperturas en Realtek;
confirmación auditiva y consolidación/CI se completaron en el [cierre](cierre-etapa-9.md). El
[protocolo nativo](../docs/sampler-native-verification.md) define 9.5.b mediciones
y 9.5.c cierre/CI. Etapa 9 completada dentro de su alcance; mezcla cerrada con mediciones, escucha confirmada y CI; siguiente 11.1.
