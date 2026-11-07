# Plan de progreso — Etapas 7 a 13

Fecha: 2026-11-07. Base publicada: `1ef9bd0` en main; etapas 1–6 completadas con
evidencia local. Estado: **etapa 7 en curso; oscilador, render acotado y WAV
implementados localmente (7.2–7.3)**. Desktop/smoke siguen en silencio; export-wav
genera señal offline. Evidencia: [reporte WAV](etapa-7-exportacion.md) y
[reporte de motor](etapa-7-oscilador.md). Cierre y CI nueva pendientes.

El [checklist](checklist.md) registra el estado; el [roadmap técnico](../docs/roadmap.md)
define los hitos y el [contrato propuesto](../docs/offline-oscillator.md) concreta
la etapa 7. Las etapas 7–13 corresponden, en el mismo orden, a los siete hitos
posteriores del roadmap anterior. Las etapas 1–6 no se renumeran ni se reabren.

## Orden y dependencias

| Etapa | Resultado                                                                      | Depende de                 | Estado                             |
| ----- | ------------------------------------------------------------------------------ | -------------------------- | ---------------------------------- |
| 7     | Oscilador offline determinista y exportación WAV desde CLI.                    | Base 1–6 y revisión de CI. | En curso; exportación local lista. |
| 8     | Mismo oscilador por CPAL, rampas y mediciones en release.                      | 7 cerrada.                 | Pendiente.                         |
| 9     | Sampler con pool fijo de voces y recursos preparados fuera del callback.       | 8 cerrada.                 | Pendiente.                         |
| 10    | Mezclador de cuatro canales con gain/pan/mute y medidores.                     | 9 cerrada.                 | Pendiente.                         |
| 11    | Secuenciador de 16 pasos con eventos dentro de cada bloque.                    | 10 cerrada.                | Pendiente.                         |
| 12    | Documento versionado, guardar/abrir, recursos relativos y exportación de beat. | 11 cerrada.                | Pendiente.                         |
| 13    | Clips y escenas con reglas explícitas de lanzamiento.                          | 12 cerrada.                | Pendiente.                         |

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

Parte local completada: nuevas pruebas descubiertas por los comandos existentes,
check y build correctos, documentación actualizada. La confirmación remota de la
revisión nueva queda pendiente junto con el cierre 7.5; ver reporte de exportación.

### 7.5 — Cerrar la etapa

Entregables: módulo de oscilador real, coordinación offline, comando WAV, pruebas,
guías actualizadas y reporte fechado con comandos/resultados. Marcar la etapa
solo al cumplir la aceptación anterior. Cualquier validación no disponible queda
identificada; no se sustituye por afirmaciones de éxito.

## Etapas posteriores: responsabilidad y criterio de cierre

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

Concluir **7.4/7.5: validar/documentar el conjunto y obtener CI del estado nuevo**.
7.3 ya tiene render/exportación y pruebas. La etapa 7 permanece abierta hasta
registrar su cierre; la siguiente etapa musical será 8 (CPAL/rampas/mediciones),
sin anticipar sampler, mezclador ni secuenciador.
