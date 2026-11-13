# Plan de ejecución — Cierre del sampler y etapas 10–13

Fecha: 2026-11-11. Base local revisada: `4dc2c6c2e0882b8f131b68088feffa8def84ba83`.
Al revisar, el árbol estaba limpio y `main` coincidía con la referencia local
`origin/main`. La planificación inicial no consultó el remoto ni su CI.
La ejecución posterior del 2026-11-11 confirmó `main` remoto y ambos jobs correctos
de `4dc2c6c` en run 42896571891, y auditó huella/artefactos; ver
[auditoría 9.5.c](auditoria-cierre-9.5c.md). Después de repetir la prueba con
audífonos, el usuario confirmó «Si se escucha»: [etapa 9 cerrada](cierre-etapa-9.md).
El [contrato 10.1](../docs/mixer-contract.md) y el
[núcleo 10.2](etapa-10-mezclador-motor.md) están implementados localmente.
[Servicio 10.3.a](etapa-10-sesiones-mezclador.md) y
[aplicación 10.3.b](etapa-10-aplicacion-mezclador.md) completados localmente.
[Desktop 10.3.c](etapa-10-desktop-mezclador.md) también completado localmente.
[Aceptación 10.3.d](cierre-integracion-mezclador-10.3d.md) completada. [Diagnósticos 10.4.a](etapa-10-diagnosticos-mezclador.md) listos. [Matriz nativa 10.4.b medida](etapa-10-medicion-nativa-mezclador.md); etapa 10 cerrada con escucha y CI en [cierre](cierre-etapa-10.md).

Este documento concreta las próximas entregas del
[plan general](plan-etapas-7-13.md), sin cambiar su orden ni dar por realizadas
las tareas propuestas. Las etapas 1–10 están completadas dentro de su alcance;
núcleo, servicio, aplicación y desktop hasta 10.3.c están implementados;
aceptación integrada, verificación nativa y cierre 10.4 completados dentro del alcance registrado. Los reportes anteriores conservan su evidencia fechada.

## Cierre 9.5.b–c — Completado el 2026-11-11

Objetivo: cerrar el sampler integrado con evidencia de la revisión entregada
antes de introducir el mezclador. Las seis mediciones Realtek/WASAPI ya están
[registradas](etapa-9-medicion-nativa.md). Auditoría/CI y confirmación de audibilidad
ya completadas; el desglose siguiente documenta el trabajo realizado.

1. **Auditar la revisión actual y su CI.** Identificar el SHA remoto y los jobs
   `headless-linux` y `desktop-windows` que corresponden a esa revisión. Registrar
   enlaces y resultados; corregir fallos si aparecen. La CI de un commit anterior
   no valida los cambios posteriores. Comprobar qué verificaciones ya cubren
   exactamente el código actual antes de decidir cuáles hay que repetir.
2. **Completar la observación de escucha.** Coordinar una sesión de sampler con
   quien pueda escuchar la salida física. Registrar fecha, dispositivo, entradas,
   controles ejercitados y lo que esa persona observe, incluidos cortes o errores.
   Si hay que repetir la sesión para obtener esa observación, seguir el
   [protocolo nativo](../docs/sampler-native-verification.md). No atribuir escucha
   a los contadores ni a los probes; sin confirmación este punto sigue pendiente.
3. **Consolidar la evidencia.** Contrastar criterios con logs, manifiesto,
   resultados por ID y lifecycle. Mantener separados resultados de procesamiento,
   escucha y continuidad acústica. Conservar como límites explícitos loopback,
   desconexión física, estrés prolongado, otros dispositivos/formatos y plataformas
   nativas no verificadas. Repetir las pruebas afectadas si se corrige código y
   completar check/build sobre la revisión final según el protocolo.
4. **Actualizar el estado y cerrar.** Alinear README, checklist, auditoría de
   estado y plan general con Git: `4dc2c6c` ya contiene las trazas y evidencia
   antes descritas como cambios locales. Crear el reporte de cierre 9.5.c con
   aceptación, SHA, comandos/resultados y CI inspeccionada. Distinguir revisiones
   de implementación y documentación si son distintas.

**Criterio de salida:** escucha registrada, verificaciones correspondientes
correctas, ambos jobs del SHA entregado inspeccionados, evidencia trazable y
checklist coherente. Un fallo o dato pendiente se registra antes de declarar
cerrada la etapa. Este plan no ejecuta commit, push ni publicación.

**Etapa 10 cerrada dentro de su alcance:** usuario confirmó «Se escucha» tras la repetición del mezclador.
[Cierre](cierre-etapa-10.md) y [evidencia](evidencia-cierre-mezclador-10.json).
**11.1 implementado localmente:** [contrato y modelo del patrón](etapa-11-modelo-patron.md).
**11.2 implementado localmente:** [planificador y mezcla secuenciada](etapa-11-planificador.md).
**11.3.a implementado localmente:** [servicio secuenciado](etapa-11-sesiones-secuenciador.md).
**11.3.b implementado localmente:** [aplicación](etapa-11-aplicacion-secuenciador.md).
**11.3.c implementado localmente:** [desktop](etapa-11-desktop-secuenciador.md).
**11.3.d aceptada localmente:** [beat de referencia e integración](cierre-integracion-secuenciador-11.3d.md).
**11.4.a implementada localmente:** [diagnósticos y protocolo](etapa-11-diagnosticos-secuenciador.md).
**11.4.b medida:** [seis casos físicos y reaperturas](etapa-11-medicion-nativa-secuenciador.md), escucha pendiente.
**Etapa 12 implementada localmente:** [persistencia/exportación](etapa-12-persistencia-exportacion.md).
En curso: auditoría/entrega/CI y cierre; [continuidad](ejecucion-11.4b-12.md).

El modelo 11.1 y el planificador con offsets de bloque 11.2 están implementados.
11.3.a ya conecta sesiones, FIFO y estado observado; 11.3.b coordina el documento
y la instalación; 11.3.c expone IPC/cuadrícula. 11.3.d completa la aceptación
local con WAV reales y referencias independientes. Sigue cierre nativo/entrega 11.4.
La etapa 11 completa sigue abierta.

## Etapa 10 — Mezclador de cuatro canales

Objetivo: controlar cuatro canales musicales y obtener una salida mezclada
verificable. Los 16 slots y las 16 voces del sampler no representan estos canales.

| Paso                | Entrega                                                                                              | Aceptación                                                                                                                                                            |
| ------------------- | ---------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 10.1 — especificado | Contrato de mezcla: enrutamiento por slot, pool global, gain/pan/mute, headroom, rampas y medidores. | Contrato y fixtures definidos; núcleo 10.2 y desktop 10.3.c locales, aceptación integrada 10.3.d local; etapa 10 cerrada con mediciones, CI y audibilidad confirmada. |
| 10.2 — local        | Modelo de cuatro canales, rutas capturadas, gain/pan/mute, rampas y medidores por render.            | Fixtures independientes, particiones y memoria verificadas; ver etapa-10-mezclador-motor.md.                                                                          |
| 10.3                | Comandos de aplicación/servicio, DTO/IPC y cuatro tiras de mezcla en UI.                             | Controles aplicados y observados de forma coherente; medidores calculados sobre señal real y telemetría acotada.                                                      |
| 10.4                | Verificación integrada y cierre.                                                                     | Pruebas de mezcla y regresiones de sampler/transporte, check/build, comprobación nativa correspondiente y CI de la revisión entregada.                                |

10.1 fija rutas iniciales slot % 4 y captura de destino al disparar, conservando
el pool global de 16 voces. 10.2 ya contrasta muestras, estado y memoria; el siguiente trabajo conecta
el render con nuevos comandos y UI.

Desglose del próximo incremento:

1. **10.3.a — Servicio de audio, completado localmente:** sesiones mezcladas con ajustes Copy,
   encolar controles/rutas por la FIFO existente y agrupar medidores sobre todos
   los chunks de cada callback. Verificar pérdidas/overflow, reinicio y propiedad PCM.
2. **10.3.b — Aplicación, completado localmente:** documento seleccionado,
   aceptación de cola y estado aplicado diferenciados,
   inicializar la sesión con ajustes explícitos y propagar errores sin confundir
   aceptación de cola con aplicación efectiva.
3. **10.3.c — Desktop, completado localmente:** ampliar DTO/IPC/ACL y bindings; exponer cuatro tiras con
   gain/pan/mute, rutas y niveles, conservando modo navegador y lifecycle.
4. **10.3.d — Aceptación local, completada:** fixtures de sesión/aplicación, pruebas mock/UI,
   check/build y reporte. Medición/escucha nativa y CI exacta cierran etapa 10 en 10.4.

10.4 se desglosa en: **a, implementado localmente**, herramientas/protocolo y
benchmarks offline; **b, matriz física medida y escucha confirmada**, seis casos/reaperturas; **c, auditoría/entrega/CI de implementación completadas**,
con [descripción revisable](entrega-mezclador-etapa-10.md) antes del cierre. No se atribuyen
mediciones del sampler original al camino mezclado.

## Etapa 11 — Secuenciador de 16 pasos

Objetivo: reproducir un patrón de cuatro canales a tempo controlado.

**11.1 completado localmente el 2026-11-12**, con
[contrato](../docs/sequencer-contract.md), [reporte](etapa-11-modelo-patron.md) y
[evidencia](evidencia-modelo-patron-11.1.json). **11.2 implementado localmente**,
con [reporte](etapa-11-planificador.md) y [evidencia](evidencia-planificador-11.2.json).
Servicio 11.3.a implementado localmente: [reporte](etapa-11-sesiones-secuenciador.md)
y [evidencia](evidencia-sesiones-secuenciador-11.3a.json). Aplicación 11.3.b implementada:
[reporte](etapa-11-aplicacion-secuenciador.md), [evidencia](evidencia-aplicacion-secuenciador-11.3b.json).
IPC/UI 11.3.c implementados localmente: [reporte](etapa-11-desktop-secuenciador.md)
y [evidencia](evidencia-desktop-secuenciador-11.3c.json).
Aceptación integrada 11.3.d [completada localmente](cierre-integracion-secuenciador-11.3d.md).

| Paso | Entrega                                                                                | Aceptación                                                                                                                                  |
| ---- | -------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------- |
| 11.1 | Contrato y modelo de patrón: pasos, resolución musical, eventos, repetición y edición. | Semántica definida para Play/Pause/Stop, cambios de BPM y aplicación de ediciones durante reproducción.                                     |
| 11.2 | Planificación de eventos con offsets dentro del bloque.                                | Sin eventos perdidos/duplicados en límites; política de redondeo y deriva probada con tempos fraccionales y distintas particiones.          |
| 11.3 | Aplicación/IPC y cuadrícula de cuatro filas por 16 pasos.                              | Editar, reproducir y observar posición; los eventos musicales se programan en el motor y la UI refleja su estado.                           |
| 11.4 | Patrón completo de referencia y cierre.                                                | Audio y eventos esperados en varios BPM/bloques; integración con mixer, regresiones, check/build, verificación nativa y CI correspondiente. |

**Hito de producto:** al cerrar 11 se puede crear y reproducir un beat en memoria.
Guardar ese beat y exportarlo pertenecen a 12.

Desglose de 11.3: **11.3.a/b/c completados localmente; siguiente 11.3.d**.

1. **11.3.a — Servicio de audio, implementado localmente:** sesiones secuenciadas manuales/CPAL, patrón instalado
   explícitamente, controles Copy por la FIFO existente y snapshot observado.
   Aplicar comandos una vez al inicio del callback; comprobar chunks, pérdida,
   overflow y propiedad del PCM al cerrar/reabrir.
2. **11.3.b — Aplicación, implementada localmente:** Project seleccionado, cola aceptada y estado aplicado
   diferenciados; preparación/cancelación y sesiones con patrón/tempo/mezcla
   actuales. Errores/cola llena sin mutar el documento.
3. **11.3.c — Desktop, implementado localmente:** DTOs/bindings/IPC/ACL y cuadrícula 4 × 16 con slots por fila y
   playhead observado; ninguna nota programada desde un temporizador de UI.
4. **11.3.d — Aceptación local, completada:** dos casos nuevos con 42 combinaciones
   de WAV/PCM/documento/sesión, auditoría de lifecycle/errores/IPC/UI, check,
   release y build. [Reporte](cierre-integracion-secuenciador-11.3d.md).

El cierre 11.4 se ejecutará en este orden:

1. **11.4.a — Diagnósticos y protocolo, completados localmente:** comandos explícitos,
   referencia PCM/eventos, 72 mediciones offline release y memoria verificada;
   runner nativo probado con backend manual y generador de cuatro WAV con hashes.
   [Reporte](etapa-11-diagnosticos-secuenciador.md) y protocolo listos para 11.4.b.
   Los benchmarks de sampler/mezclador no inicializan
   el secuenciador y no se reutilizan como medición de este camino.
2. **11.4.b — Matriz física y escucha:** medir dispositivo/configuración reales,
   controles, errores/reaperturas y audición identificada. Conservar los límites
   de loopback, desconexión física, duración y otras plataformas.
3. **11.4.c — Auditoría y entrega:** consolidar evidencia de la revisión final,
   verificar check/build y CI del SHA entregado, y cerrar solo al cumplir criterios.

## Etapa 12 — Guardar, abrir y exportar beats

| Paso | Entrega                                                                       | Aceptación                                                                                                                              |
| ---- | ----------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------- |
| 12.1 | Esquema versionado del proyecto con patrones, mezcla y referencias a samples. | Reglas de rutas relativas, compatibilidad y migración explícitas; documento separado del estado transitorio de reproducción.            |
| 12.2 | Guardar/abrir mediante adaptadores fuera del callback.                        | Round-trip conserva el proyecto; errores claros de versión, archivo inválido, escritura y assets ausentes.                              |
| 12.3 | Render offline del proyecto y exportación WAV del beat.                       | Reutiliza sampler, mezcla y secuenciador; duración, repeticiones y colas de samples definidas; muestras consistentes entre particiones. |
| 12.4 | Flujo integrado y cierre.                                                     | Crear beat, guardar, cerrar, reabrir y exportar con fixtures reproducibles, check/build y CI de la entrega.                             |

**Hito de producto:** sesión musical recuperable y beat exportable.
12.1–12.3 y el flujo local de 12.4 ya están implementados y probados; ver
[reporte](etapa-12-persistencia-exportacion.md). Entrega/CI y cierre auditivo de
etapa 11 se registran en [continuidad](ejecucion-11.4b-12.md).

## Etapa 13 — Clips y escenas

| Paso | Entrega                                                                             | Aceptación                                                                                             |
| ---- | ----------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------ |
| 13.1 | Modelo de clips/escenas y reglas de lanzamiento, parada y cuantización.             | Relación explícita con patrones, transporte y formato persistido.                                      |
| 13.2 | Transiciones programadas dentro del motor.                                          | Cambios deterministas entre tempos y tamaños de bloque; sin disparos perdidos/duplicados en fronteras. |
| 13.3 | Edición/lanzamiento desde aplicación/UI e integración con persistencia/exportación. | Estado solicitado y observado coherentes; semántica de guardar y exportar definida para clips/escenas. |
| 13.4 | Verificación de sesiones y cierre.                                                  | Secuencias de lanzamiento/transición reproducibles, regresiones, check/build y CI correspondiente.     |

Timeline, piano roll y automatización se planifican después. Grabación, MIDI
externo, plugins y time-stretching mantienen su alcance posterior.

## Forma de ejecución

Entregar incrementos revisables: contrato, implementación del motor,
integración y cierre. Cada etapa empieza tras cerrar su dependencia y conserva
su propio reporte de aceptación. Las comprobaciones se eligen según lo que
cambia; los resultados locales y remotos siempre identifican su revisión.
No se estiman fechas antes de resolver las decisiones de contrato de cada etapa.
