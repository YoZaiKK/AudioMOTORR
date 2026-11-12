# Etapa 10.3.b — Coordinación de aplicación y mezcla

Fecha: 2026-11-11, hora de México. Base Git `4dc2c6c` con 10.2 y 10.3.a locales
preexistentes. **Coordinación de aplicación implementada localmente; etapa 10
todavía abierta.** Siguiente: 10.3.c, DTO/IPC/ACL y controles de interfaz.

## Entrega

- [Application](../crates/daw-app/src/mixer.rs) expone selección de ajustes/rutas
  y devuelve `MixerEdit { settings, accepted }` sin simular estado aplicado.
- Audio sin inicializar: edición validada solo en `Project::mixer`, con
  `accepted: None`, incluso con preparación en curso o banco listo.
- Sesión mezclada activa: el documento cambia después de aceptación por la FIFO.
  `Some(CommandAccepted)` confirma la cola; targets, rampas y medidores se consultan
  por separado en `ServiceSnapshot.mixer`. Render manual vacío no aplica controles.
- Índices/valores inválidos, cola llena, fuente legacy y errores del servicio
  conservan la selección anterior. Los errores originales no se convierten en
  ediciones offline exitosas.
- [Instalación](../crates/daw-app/src/sampler_session.rs) manual/nativa consume
  el banco preparado con sus gates existentes y copia los ajustes más recientes
  del proyecto junto con el tempo. No hay captura prematura al iniciar la carga,
  relectura de WAV, apertura implícita o Play automático.
- Cierre conserva la selección aceptada, aunque sus comandos no hayan llegado al
  callback. La reapertura instala esos ajustes explícitamente, sin replay de
  comandos ni confirmaciones fabricadas.

El [contrato de aplicación](../docs/mixer-application.md) fija resultados según
estado, preparación y semántica del recibo. Domain/engine/audio, decoder,
presupuestos, DTO/IPC/ACL/UI y dependencias permanecen sin cambios frente a 10.3.a.

## Pruebas y aceptación

Se añaden **nueve pruebas únicas**: ocho en
[mixer_application_contract](../crates/daw-app/tests/mixer_application_contract.rs)
y una de política de errores. El caso native-disabled se compila solo sin esa
feature; debug/release y suites se solapan. La prueba existente del worker
bloqueado añade edición offline, instalación rechazada durante carga y retención
de la selección después de cancelar.

| Área                                | Resultado verificado                                                                                                                           |
| ----------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------- |
| Preparación real                    | WAV float32 decodificado; cambios antes/después de preparar; archivo eliminado antes de instalar; PCM y metadatos conservados.                 |
| Selección / aceptación / aplicación | Sin ID offline; aceptación real en vivo; targets antes de rampas efectivas; bloque vacío no drena comandos.                                    |
| Orden y errores                     | Rutas capturadas por FIFO, voces pausadas preservadas, cola 128 y presupuesto 32; rechazo no cambia documento ni consume IDs.                  |
| Ciclo de vida                       | Selección/tempo aceptados sobreviven a cierre y reapertura, sin replay ni auto Play; ID de preparación y secuencia de comandos independientes. |
| Fallos                              | Índices, NaN/infinito, modos legacy, Busy, ID obsoleto, cancelación, carga fallida y native-disabled sin fallback.                             |
| Propagación                         | Política de errores conserva códigos/detalles; servicio fallido o cerrándose no se presenta como edición offline correcta.                     |

## Verificación local

| Comando                                                                                                                                                                  | Resultado                                                                                   |
| ------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------- |
| `npm run check`                                                                                                                                                          | Correcto: bindings, frontend, CLI, Rust headless/workspace, memoria instrumentada y Clippy. |
| `cargo test -p daw-app --release --no-default-features --locked --lib --test mixer_application_contract --test application_contract --test sampler_application_contract` | Correcto, incluidas regresiones de tempo/sampler.                                           |
| `npm run desktop:build -- --debug --no-bundle`                                                                                                                           | Correcto, después de las suites; ejecutable debug sin instalador.                           |

Logs locales en `target/stage10-3b-20261111/`; el
[manifiesto](evidencia-aplicacion-mezclador-10.3b.json) identifica fuentes y
comandos/hashes. La nueva evidencia conserva los manifiestos anteriores como
snapshots fechados. Sin commit/push ni CI remota nuevos.

La huella registra 131 archivos de código/configuración: cuatro difieren respecto
de 10.3.a, incluidos dos archivos nuevos. Se contrastan también todos los hashes
de audio/domain/engine, locks, bindings y workflow, sin diferencias.

Revisión final: formato y `git diff --check` correctos; 471 destinos locales de
enlaces en 23 documentos, sin ausentes. Los 131 hashes coinciden con las fuentes
finales de la verificación.

## Siguiente incremento y límites

**10.3.c:** exponer ajustes seleccionados, recibos y observaciones aplicadas mediante
DTO/IPC/ACL y TypeScript generado; integrar cuatro tiras con gain/pan/mute,
enrutamiento y niveles. Pruebas de orden de observaciones/respuestas, estado
offline/en vivo y restricciones de navegador antes de 10.3.d.

No se abrió hardware ni se realizó una nueva audición física. Persistencia de
proyectos sigue en 12; aceptación integrada, mediciones nativas y CI exacta del
mezclador permanecen en 10.4. No se atribuye cero asignaciones a toda la aplicación:
la restricción de realtime se verifica sobre el motor/servicio sin cambios.
