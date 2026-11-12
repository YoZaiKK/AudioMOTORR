# Plan de progreso — Etapas 7 a 13

Actualizado: 2026-11-11. Etapa 7 completada, publicada en c6cf627 y verificada
en Linux headless/Windows desktop por CI. **Etapa 8 completada**, publicada en
1e6bca8 con CI correcta en run 42524752652: señal nativa explícita, rampas y
mediciones release. Ver [reporte actual](etapa-8.md)
para comprobaciones y estado de publicación/CI. Desktop conserva silencio inicial
y permite seleccionar senoide; los smoke ordinarios siguen en silencio.
Los reportes de motor/WAV conservan sus resultados previos a la publicación.

**Etapa 9 completada dentro de su alcance**; ver [cierre](cierre-etapa-9.md).
Implementación publicada en 4dc2c6c, CI Linux/Windows correcta, matriz Realtek y
audibilidad confirmada por el usuario tras repetición con audífonos.
**10.1 especificado** en el [contrato de mezcla](../docs/mixer-contract.md);
**10.2 implementado localmente**; ver [reporte](etapa-10-mezclador-motor.md).
**10.3.a completado localmente**; ver [sesiones mezcladas](etapa-10-sesiones-mezclador.md).
**10.3.b completado localmente**; ver [coordinación de aplicación](etapa-10-aplicacion-mezclador.md).
**10.3.c completado localmente**; ver [desktop](etapa-10-desktop-mezclador.md).
**10.3.d completado localmente**; ver [aceptación integrada](cierre-integracion-mezclador-10.3d.md).
**10.4.a implementado localmente**; ver [diagnósticos de mezcla](etapa-10-diagnosticos-mezclador.md).
**Matriz 10.4.b medida**; ver [resultados nativos](etapa-10-medicion-nativa-mezclador.md).
[Auditoría local 10.4.c completada](auditoria-cierre-mezclador-10.4c.md).
[Publicación 10.4.c](publicacion-mezclador-10.4c.md): implementación `e7f543f` entregada en main y CI Linux/Windows correcta.
**Etapa 10 cerrada dentro de su alcance:** usuario confirmó «Se escucha» tras la repetición del mezclador.
[Cierre](cierre-etapa-10.md) y [evidencia](evidencia-cierre-mezclador-10.json).
**Siguiente: 11.1**, contrato y modelo del patrón/secuenciador de 16 pasos.
9.3.a está completada como [contrato de carga](../docs/sample-loading-contract.md),
9.3.b implementa lector/decoder de recursos individuales y 9.3.c añade
coordinación de banco/permisos/presupuesto agregado. 9.3.d verifica fallos,
cancelación/entrega y cierre entre hilos. 9.3.e verifica fixtures de extremo a
extremo y matriz de aceptación; 9.3.f cierra preparación localmente.
9.4.a integra AudioEngine y 9.4.b conecta sesiones/outcomes manual/CPAL, con
pruebas sin hardware. 9.4.c añade preparación en worker, DTO/IPC/UI y observación
por ID. 9.4.d audita/cierra integración local, corrige recuperación/polls y
define protocolo nativo. 9.5.a entrega sampler-bench/native-check y métricas
acotadas con tests sin dispositivo. 9.5.b registra seis mediciones Realtek/WASAPI
correctas; escucha humana y cierre 9.5.c completados posteriormente. Ver
[reporte actual](etapa-9-medicion-nativa.md).
El [contrato](../docs/sampler-contract.md) y [reporte actual](etapa-9-voces.md)
separan implementación, comprobaciones y límites.

La [auditoría de estado, objetivos y siguiente etapa](estado-y-siguiente-etapa.md)
contrasta el plan con la implementación publicada en `4dc2c6c`, que incorpora
el kernel 7acc1db, preparación, integración, diagnósticos y evidencia nativa.
La [auditoría 9.5.c](auditoria-cierre-9.5c.md) verifica huella/artefactos y ambos
jobs correctos del SHA exacto en run 42896571891. La confirmación de audibilidad
completa el cierre de etapa 9.
Los reportes de incrementos conservan sus estados históricos de publicación/CI.
La groovebox utilizable
requiere completar 10–11; guardar/reabrir/exportar el beat llega en 12.
La UI de sampler está implementada con mocks; audibilidad física confirmada
mediante el runner nativo, sin atribuirle cobertura de todos los flujos de UI.
El WAV de senoide no equivale a exportación de beat.

El [checklist](checklist.md) registra el estado; el [roadmap técnico](../docs/roadmap.md)
define los hitos y el [contrato](../docs/offline-oscillator.md) concreta
la etapa 7. Las etapas 7–13 corresponden, en el mismo orden, a los siete hitos
posteriores del roadmap anterior. Las etapas 1–6 no se renumeran ni se reabren.

## Orden y dependencias

| Etapa | Resultado                                                                      | Depende de                 | Estado                                                                                                                                                                                            |
| ----- | ------------------------------------------------------------------------------ | -------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 7     | Oscilador offline determinista y exportación WAV desde CLI.                    | Base 1–6 y revisión de CI. | Completada; c6cf627, CI correcta.                                                                                                                                                                 |
| 8     | Mismo oscilador por CPAL, rampas y mediciones en release.                      | 7 cerrada.                 | Completada; 1e6bca8, CI correcta.                                                                                                                                                                 |
| 9     | Sampler con pool fijo de voces y recursos preparados fuera del callback.       | 8 cerrada.                 | Completada: implementación, mediciones, CI y escucha registrada.                                                                                                                                  |
| 10    | Mezclador de cuatro canales con gain/pan/mute y medidores.                     | 9 cerrada.                 | Núcleo/servicio/aplicación/desktop hasta 10.3.d aceptados localmente; diagnósticos 10.4.a listos; matriz nativa 10.4.b correcta; CI y escucha confirmadas; etapa 10 cerrada dentro de su alcance. |
| 11    | Secuenciador de 16 pasos con eventos dentro de cada bloque.                    | 10 cerrada.                | Pendiente.                                                                                                                                                                                        |
| 12    | Documento versionado, guardar/abrir, recursos relativos y exportación de beat. | 11 cerrada.                | Pendiente.                                                                                                                                                                                        |
| 13    | Clips y escenas con reglas explícitas de lanzamiento.                          | 12 cerrada.                | Pendiente.                                                                                                                                                                                        |

El orden es de entrega, no una petición de construir todas las etapas de una vez.
Cada una termina con pruebas, documentación y evidencia propia antes de iniciar
la siguiente. Timeline, piano roll y automatización se planifican tras clips/escenas.

## Etapa 7 — Oscilador offline y WAV

Objetivo: obtener una señal verificable en un archivo sin abrir hardware y sin
cambiar el comportamiento actual de desktop. La primera implementación debe ser
pequeña y conservar los límites de paquetes existentes.

### 7.0 — Confirmar la base

- Revisar en Actions los jobs headless-linux y desktop-windows del commit
  publicado; registrar enlace, SHA, resultado y fallos si los hay.
- Resolver cualquier bloqueo de esa base antes de añadir señal. Si una ejecución
  no está disponible, registrar el bloqueo y distinguir las pruebas locales.
- Registrar git status y conservar trabajo ajeno; tomar hashes de lockfiles/tipos.

Aceptación: resultado de CI conocido o limitación concreta documentada; no
presentar como verde una ejecución no inspeccionada.

Completado el 2026-11-07: [run 42314334447](https://github.com/YoZaiKK/AudioMOTORR/actions/runs/42314334447)
de `1ef9bd0afb35bf380004f87c65ca7ccace132357`, ambos jobs correctos. Se confirma
núcleo Ubuntu y desktop Windows de esa revisión; no cambios locales posteriores.

### 7.1 — Fijar el contrato del incremento

- Usar el [contrato propuesto en inglés](../docs/offline-oscillator.md): frecuencia
  y amplitud validadas, fase, transporte, señal explícita y WAV float32.
- Mantener silencio por defecto. Solo la operación offline activa el oscilador.
- Determinar la API mínima concreta al implementar, sin modificar DTO/UI ni crear
  traits genéricos para instrumentos futuros. En 7.3 se eligió hound 3.5.1,
  fijado solo en CLI, con actualización deliberada de Cargo.lock.

Aceptación: políticas decididas antes del código de señal y pruebas que las
comprueben. Completado para este incremento: constructor with_sine y errores
tipados en motor; políticas de WAV fijadas y escritor seleccionado durante 7.3.
El contrato implementado incorpora señal explícita y exportación.

### 7.2 — Implementar y verificar el oscilador en daw-engine

- Añadir un módulo interno con fase por frame, sin dependencias externas.
- Renderizar en el buffer interleaved existente; conservar la fase entre bloques
  y duplicar la misma muestra en los canales, sin avanzar fase por canal.
- Play reanuda; Pause silencia y conserva fase; Stop reinicia fase/posición.
  Los cambios de BPM no cambian la frecuencia en Hz.
- Mantener estado/fase tras error, buffers vacíos y contadores comprobados.
- Verificar señal de referencia, parámetros inválidos, mono/estéreo, transporte
  y equivalencia completa de muestras con bloques 1/127/256/4096 y bloque final parcial.

Aceptación: tests de señal y regresiones de silencio correctos; render sin
asignaciones dinámicas, locks, I/O o serialización por diseño e inspección.
Esto no sustituye la instrumentación del allocator prevista en etapa 8.

Implementado localmente: módulo interno oscillator.rs, constructor opt-in y seis
pruebas nuevas, además del overflow que ahora comprueba fase. Referencia a tres
sample rates y 1/2/6 canales, particiones exactas y transporte. Ver reporte para
comandos y resultados finales; default de audio nativo/CLI sigue silencioso.

### 7.3 — Coordinar render offline y escribir WAV

- daw-app valida y coordina un motor independiente del servicio vivo/documento;
  reutiliza un buffer acotado y entrega bloques a un destino fuera del render.
- El render_manual actual devuelve solo un reporte de frames/silencio, no las
  muestras. No tratarlo como API de exportación ni copiar el DSP a la CLI.
- daw-cli valida ruta, frames exactos y bloque; escribe/finaliza WAV float32 con
  metadatos correctos. El comando implementado es export-wav.
- Rechazar destino existente por defecto, duración no representable y parámetros
  inválidos. Propagar errores de creación/escritura/finalización, cerrar recursos
  y documentar qué ocurre con un archivo parcial.

Implementado: OfflineRender en daw-app y export-wav con hound 3.5.1 en CLI.
Aceptación: exportación sin CPAL/Tauri ni hardware, memoria acotada por bloque,
archivo reabierto con muestras y metadatos correctos, duración cero y bloque final
parcial cubiertos; errores verificables y código de salida no cero cuando corresponda.

### 7.4 — Integrar comprobaciones y documentación

- Conservar los nueve escenarios smoke de silencio y añadir escenarios de señal/
  WAV que usen directorios temporales, sin archivos de salida versionados.
- Integrar la nueva cobertura en scripts/check.mjs y CI sin abrir hardware ni
  instalar dependencias nativas en el job Linux.
- Actualizar contrato implementado, README, guía CLI, pruebas y seguimiento con
  comandos reales. Si aparece una dependencia WAV, registrar el cambio deliberado
  de lockfile; las comprobaciones posteriores no deben modificarlo.
- Ejecutar tests/Clippy headless, check completo y después build Windows debug.
  Comprobar bindings intactos si no cambian DTO y registrar CI del nuevo SHA.

Aceptación: checks afectados correctos, WAV contrastado con el motor, limitaciones
explícitas y resultados remotos separados de los locales. No actualizar capturas
de una UI que no cambia. No afirmar continuidad nativa o ausencia de dropouts.

Completada: nuevas pruebas descubiertas por los comandos existentes, check y build
correctos, documentación actualizada y run 42419963899 correcto para c6cf627.

### 7.5 — Cerrar la etapa

Completado el 2026-11-07. Ver [matriz de aceptación y CI](cierre-etapa-7.md).

Entregables: módulo de oscilador real, coordinación offline, comando WAV, pruebas,
guías actualizadas y reporte fechado con comandos/resultados. Marcar la etapa
solo al cumplir la aceptación anterior. Cualquier validación no disponible queda
identificada; no se sustituye por afirmaciones de éxito.

## Etapas posteriores: responsabilidad y criterio de cierre

La fila 8 registra el criterio acordado; su implementación y mediciones actuales
se detallan en el reporte y en las subetapas siguientes.

| Etapa | Responsabilidades                                                                                                                                             | Criterio de cierre                                                                                                                                                                                                                          |
| ----- | ------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 8     | daw-engine implementa rampas y transiciones; daw-audio conecta el mismo oscilador a CPAL; daw-app/IPC/React exponen el control explícito necesario.           | Señal nativa consistente con offline, rampas probadas, lifecycle y errores conservados. Medir carga, tiempos de callback, asignaciones y continuidad en release sobre hardware identificado; declarar pendientes de plataforma/dispositivo. |
| 9     | daw-engine reproduce samples con pool de voces fijo; aplicación prepara/carga/decodifica recursos; UI solicita acciones mediante casos de uso.                | Fixtures deterministas, política de agotamiento de voces, propiedad de recursos y cierre seguros; sin carga/decodificación/asignación en callback.                                                                                          |
| 10    | Documento guarda estado de cuatro canales; motor mezcla gain/pan/mute y acumula medidores; audio publica telemetría acotada; UI controla mediante aplicación. | Ganancia, ley de pan, mute, suma y política de nivel de salida definidos y probados con fixtures; medidores corresponden a muestras reales.                                                                                                 |
| 11    | Dominio guarda patrones; motor agenda eventos con offsets dentro del bloque; aplicación edita y UI presenta 16 pasos.                                         | Eventos equivalentes entre particiones, tempos con frames fraccionales por paso, cambios de BPM y Play/Pause/Stop sin eventos duplicados/perdidos en límites; redondeo/drift especificados.                                                 |
| 12    | Dominio define documento versionado; aplicación migra/resuelve recursos; adaptadores guardan/abren/exportan fuera del callback.                               | Round-trip de documentos, versiones inválidas y assets ausentes con errores claros; export de beat con duración exacta y muestras consistentes con render offline.                                                                          |
| 13    | Dominio representa clips/escenas; motor agenda transiciones; aplicación/UI editan y lanzan.                                                                   | Cuantización, lanzamiento y transiciones definidos y probados entre tamaños de bloque/tempos, manteniendo documento separado de reproducción y colas acotadas.                                                                              |

No forman parte de este plan inmediato grabación, MIDI externo, time-stretching,
hosting VST/CLAP, routing complejo, compensación de latencia ni colaboración.

## Siguiente acción concreta

Implementar **11.1**, contrato y modelo del patrón/secuenciador: cuatro filas,
16 pasos, resolución musical, semántica de edición y transporte/BPM.
El planificador con offsets de bloque será 11.2.
[Etapa 10 cerrada](cierre-etapa-10.md); persistencia/exportación de beat y clips
siguen en 12–13.

## Etapa 9 — Desglose y aceptación (historial previo al cierre)

### Etapa 9.3 e integración 9.4 — Cerradas localmente; diagnósticos 9.5.a implementados

Resultado: transformar WAV admitidos en un banco completo validado fuera del
procesamiento, conservando recursos existentes ante fallo/cancelación.

| Paso  | Entrega                                                        | Criterio de cierre                                                                                                                                         |
| ----- | -------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 9.3.a | Contrato/presupuestos especificados el 2026-11-09.             | Formatos y límites individuales aplicados en 9.3.b; presupuestos agregados de preparación/coexistencia y permiso implementados en 9.3.c.                   |
| 9.3.b | Lector/decoder individual implementado localmente.             | Preflight, formatos/límites, reserva exacta, cancelación/cierre, round-trip export y pruebas de memoria debug/release correctos.                           |
| 9.3.c | Coordinador de banco implementado localmente.                  | Slots validados antes de I/O, carga ordenada, permiso hasta entrega/descarte, límites agregados y memoria de dos bancos máximos verificados debug/release. |
| 9.3.d | Fallos/cancelación/cierre verificados localmente.              | Matriz de 17 casos, entrega/Read sincronizados, cierre de lectores y memoria en trabajador/receptor correctos debug/release.                               |
| 9.3.e | Fixtures de extremo a extremo y matriz verificadas localmente. | Cuatro pruebas debug/release: 18 combinaciones × cuatro particiones, todos los códigos PCM16, bancos y límites reales exactos/un exceso.                   |
| 9.3.f | Auditoría y cierre local completados.                          | Criterios contrastados, check/build 9.3.e y huella de 110 archivos; publicación/CI/plataformas pendientes explícitos. Etapa 9 completa sigue abierta.      |

El [contrato 9.3.a](../docs/sample-loading-contract.md) especifica 16 MiB/archivo,
4096 chunks, 64 KiB de buffers, 8 MiB extra de finalización y una sola preparación.
Presupuesto de datos: 136 MiB + 64 KiB de preparación, o 264 MiB + 64 KiB con un
banco activo. Se aplican en 9.3.c con medición de heap de bancos máximos; no es
RSS de la app ni una medición de callback nativo.
Los límites PCM ya existentes se conservan. La [auditoría](estado-y-siguiente-etapa.md)
y el [reporte](etapa-9-contrato-carga.md) distinguen decisiones de código operativo.

### Estado de los incrementos de sampler

- **9.1, contrato y recursos:** PCM f32 finito normalizado, mono/estéreo,
  máximo 1048576 frames/recurso y rate igual al de salida. SampleBuffer posee
  memoria inmutable; preparación/destrucción fuera del procesamiento. Fixtures
  y cuatro tests de validación/compatibilidad implementados. Política fijada:
  banco de 16 slots y 16 voces, elegir la primera libre y rechazar el nuevo
  disparo al agotar el pool. El reporte de 9.1 conserva la evidencia de ese alcance.
- **9.2, motor:** banco/kernel de one-shots implementados, suma determinista,
  fin exacto, reuse/agotamiento y oráculos del contrato. Ocho tests, overflow
  interno y prueba instrumentada verifican muestras/estado y vida útil sin
  asignaciones/liberaciones en procesamiento. Sin transporte del AudioEngine aún.
- **9.3, preparación:** carga/decodificación y presupuesto de memoria en
  aplicación fuera del callback; errores de archivo/formato/rate/canales/longitud
  y cierre seguro. 9.3.b–c implementan decoder/coordinador; matriz ampliada de
  fallos/cancelación/cierre verificadas en 9.3.d. Fixtures de extremo a extremo
  verificados en 9.3.e; auditoría/cierre local completado en 9.3.f. Evidencia
  identificada por manifiesto, publicación/CI nueva pendientes.
- **9.4, integración:** modo sampler con headroom 1/16 y transporte/rampas,
  comandos y outcomes Copy, controles explícitos e IPC/frontend con mock.
  Aceptar en cola no confirma disponibilidad de voces ni disparo exitoso.
- **9.5, cierre:** check/build, mediciones del sampler nativo con allocator y
  deallocaciones, tiempos/carga/continuidad y CI del SHA nuevo. La evidencia de
  senoide en etapa 8 no valida el futuro sampler.

## Etapa 8 — Desglose ejecutado

- **8.1, motor:** Gain validado 0–1; constructor with_ramped_sine; rampas lineales
  de ceil(sample_rate/200) frames, retarget desde nivel actual y transporte
  solicitado distinto del observado durante release. Pause conserva fase tras la
  cola y Stop reinicia al llegar a cero. Offline/WAV inmediato permanece igual.
- **8.2, servicio/aplicación:** inicialización de senoide opcional y volumen en
  FIFO existente; amplitud de voz fija 0.1; reinicio detenido con nivel efectivo
  cero. Comparación de salida nativa por chunks con motor offline y presupuesto
  de 32 comandos una vez por callback.
- **8.3, desktop:** initialize_audio admite frecuencia opcional; undécimo comando
  signal_set_gain, ACL, DTO regenerado y controles explícitos de fuente/frecuencia/
  volumen. Pruebas IPC con mock y frontend sin hardware.
- **8.4, mediciones:** feature rt-diagnostics opcional con allocation-counter
  0.8.1; CLI engine-bench y tone-check solo release. Medición local Realtek y
  microbench por cuatro particiones. No se infiere continuidad acústica ni
  ausencia de dropouts del procesamiento medido.
- **8.5, cierre:** check completo antes de build Windows y smoke WebView2;
  enlaces/formato/documentación, hashes y reporte de CI del SHA nuevo por separado.
  El [reporte](etapa-8.md) identifica resultados obtenidos y pendientes.
