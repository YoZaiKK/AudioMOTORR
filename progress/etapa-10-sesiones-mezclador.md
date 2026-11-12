# Etapa 10.3.a — Sesiones mezcladas y medidores por callback

Actualización posterior del 2026-11-11 (México): [aplicación 10.3.b](etapa-10-aplicacion-mezclador.md)
ya coordina selección/aceptación/instalación; la exposición DTO/IPC/UI sigue en
10.3.c. El informe siguiente conserva el alcance y la evidencia del servicio.

Fecha: 2026-11-11. Base publicada `4dc2c6c` con el núcleo 10.2 local ya presente.
**Servicio de audio implementado localmente; etapa 10 todavía abierta.**
Siguiente: 10.3.b, coordinación documento/aplicación; después DTO/IPC/UI en 10.3.c.

## Entrega

- `AudioHandle` instala sesiones mezcladas manuales/nativas con banco propietario
  y ajustes Copy explícitos, sin lectura de archivos ni decodificación.
- `set_mixer_channel` y `set_mixer_route` comparten FIFO/IDs con transporte y
  triggers: capacidad 128, cola de observaciones 64 y presupuesto de 32 comandos
  por callback. Índices inválidos, modo incorrecto y cola llena no alteran ajustes
  aplicados ni consumen IDs.
- `ServiceSnapshot.mixer` expone ajustes aplicados, rampas, destinos capturados y
  picos de todas las contribuciones del callback. La suma de chunks mantiene los
  picos tempranos y cuenta todos los frames; su secuencia avanza una vez por
  callback no vacío. La API directa del motor conserva ventanas por render.
- Validación de toda la salida, scratch y secuencias de ventanas antes del primer
  chunk; contadores comprobados, buffers de fallo silenciados y sin prefijos de
  audio válidos parcialmente renderizados ante fallos de prevalidación.
- Fallo/cierre limpian observaciones de mezcla; reapertura reinicia ventanas,
  voces y comandos pendientes, conservando los IDs globales del servicio.
- PCM conserva su destrucción en el propietario; el teardown nativo se verifica
  también cuando el backend descarta el callback en otro hilo.

Código en [servicio](../crates/daw-audio/src/service.rs),
[procesador](../crates/daw-audio/src/realtime.rs) y
[adaptador nativo](../crates/daw-audio/src/native.rs). El
[contrato de servicio](../docs/mixer-service.md) detalla IDs y ventanas.
No se cambia el DSP de dominio/motor, dependencias, locks o workflow de 10.2.
El test IPC existente solo añade `mixer: None` a un literal Rust: DTO de producción,
ACL y TypeScript generado aún no exponen la mezcla.

## Aceptación y pruebas

Se añaden **16 pruebas únicas**, con compilación condicional por features:
seis de servicio público, cuatro de realtime, tres de adaptador nativo y tres
de lifecycle/control. Las suites debug/release y sus features se solapan.
El teardown existente amplía su matriz a sesiones legacy/mezcladas y descarte
en propietario/trabajador.

| Área                          | Evidencia                                                                                                                                                          |
| ----------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Selección frente a aplicación | Instalación explícita, ediciones pendientes y no-op de render vacío; observaciones/ramps después de aplicar.                                                       |
| FIFO y capacidad              | Trigger/ruta/trigger conserva dos destinos distintos; mezcla de controles comparte el límite de 32 y rechaza la entrada 129 sin consumir ID.                       |
| Ventanas completas            | 1001 frames con pulsos independientes tempranos, intermedios y tardíos; coincidencia con referencia sin particionar en scratch 1/127/256/513 y f32/f64/i16/u16.    |
| Telemetría                    | Cola llena pierde solo observaciones; después informa último estado/ID y secuencia del callback, sin atribuir estados intermedios perdidos.                        |
| Errores y reinicio            | Mono, vacío, cancelación, índices, modo, configuración exacta, overflow, salida/scratch inválidos, expiración de banco y fallo/reapertura.                         |
| Memoria/propiedad             | Instrumentación sobre comandos, render, conversión, agregación y publicación: cero asignaciones/liberaciones. PCM destruido fuera de procesamiento en propietario. |

Pruebas en [servicio público](../crates/daw-audio/tests/mixer_service_contract.rs),
[realtime](../crates/daw-audio/src/realtime_mixer_tests.rs),
[adaptador](../crates/daw-audio/src/native_mixer_tests.rs) y
[lifecycle](../crates/daw-audio/src/service_mixer_tests.rs).

## Verificación

| Comando                                                                             | Resultado                                                                                                  |
| ----------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------- |
| `npm run check`                                                                     | Correcto después del último ajuste; frontend, bindings, Rust headless/workspace, instrumentación y Clippy. |
| `cargo test -p daw-audio --release --features native-audio,rt-diagnostics --locked` | Correcto; prueba física ignorada por defecto.                                                              |
| `npm run desktop:build -- --debug --no-bundle`                                      | Correcto, después de las suites; ejecutable debug sin instalador.                                          |

Logs no versionados en `target/stage10-3a-20261111/`. El
[manifiesto 10.3.a](evidencia-sesiones-mezclador-10.3a.json) registra huella,
comandos y hashes. Las evidencias de 10.2 y etapa 9 permanecen fechadas y no se
atribuyen al nuevo servicio. No hay commit/push ni CI remota nueva.

La huella actual registra 129 archivos de código/configuración; nueve difieren
respecto de 10.2, incluidos cuatro archivos nuevos de pruebas. Locks, bindings
generados y workflow conservan sus hashes. Revisión final de formato/espacios
correcta y 447 destinos locales de enlaces en 21 documentos, sin ausentes.

## Próximo incremento y límites

**10.3.b:** coordinar `Project::mixer` con carga/preparación e instalación de la
sesión, distinguir documento seleccionado de estado aplicado y propagar errores
sin simular confirmaciones de la cola. Después 10.3.c añade DTO/IPC/ACL y cuatro
tiras de mezcla en React; 10.3.d consolida aceptación local.

Los tests del adaptador no abren hardware. No se realizaron audición física del
mezclador, loopback, desconexión física, estrés prolongado o mediciones nativas
Linux/macOS. La mayor observación Copy aumenta el tamaño fijo del ring; las
capacidades y el trabajo acotado se conservan, sin atribuir cero memoria al
proceso completo. El cierre de etapa 10 y su CI exacta permanecen en 10.4.
