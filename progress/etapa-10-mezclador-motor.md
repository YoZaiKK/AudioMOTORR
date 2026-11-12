# Etapa 10.2 — Modelo y mezcla de cuatro canales

Actualización posterior del 2026-11-11: [servicio 10.3.a](etapa-10-sesiones-mezclador.md)
ya integra sesiones/FIFO y medidores por callback, con huella propia. El reporte
siguiente conserva el alcance y la verificación del núcleo 10.2.

Fecha: 2026-11-11. Base Git `4dc2c6c`; implementación local posterior al
[cierre de etapa 9](cierre-etapa-9.md). **Núcleo implementado; etapa 10 completa
todavía abierta.** Servicio/aplicación/DTO/IPC/UI y agregación por callback son
10.3; aceptación integrada, mediciones nativas y CI propia corresponden a 10.4.

## Entrega

- [Modelo de dominio](../crates/daw-domain/src/mixer.rs): cuatro canales con
  gain/pan/mute validados, tabla fija de 16 rutas y ajustes Copy. `Project` posee
  sus ajustes editables; modificarlos no cambia un motor ya instalado.
- [Motor de mezcla](../crates/daw-engine/src/mixer.rs): configuración explícita
  mediante `AudioEngine::with_mixed_sampler(bank, tempo, settings)`, comandos
  directos del propietario `set_mixer_channel`/`set_mixer_route`, rampas y snapshots.
- El pool sigue siendo global de **16 voces**. El destino se captura al disparar;
  cambiar una ruta afecta disparos futuros, no voces activas/pausadas. Mute no
  detiene cursores ni libera capacidad anticipadamente.
- Balance lineal con centro a unidad, gain/trigger/master 0–1, headroom fijo 1/16,
  orden f32 de suma por voz dentro del canal y después por canal. Mono ignora pan
  en el DSP y conserva su ajuste. Recursos estéreo mantienen sus dos componentes.
- Rampas de 5 ms con `ceil(rate/200)`, retarget desde el valor actual y edición en
  silencio aplicada inmediatamente. El master conserva su transporte/rampas previos.
- Medidores de contribuciones reales post gain/pan/headroom y master post envolvente;
  cancelación de la suma no oculta los picos de canal. Cada render no vacío define
  su ventana, frames y secuencia; mono expone `right: None`. Agregación de chunks de
  un callback real queda explícitamente para 10.3.

El [contrato 10.1](../docs/mixer-contract.md) describe las fórmulas y la aceptación.
`Sampler` raw y `with_sampler` conservan su API/comportamiento; comparten el avance
de voces con la ruta nueva. No se añaden dependencias ni se modifican IPC/UI.

## Pruebas

Se añaden **16 pruebas únicas**: dos de dominio, doce de integración del motor,
una interna de overflow de medidores y una instrumentada de memoria. Además se
amplían las regresiones existentes de overflow de dispositivo/transporte y reloj
raw a la ruta mezclada. Debug y release ejecutan suites solapadas.

Las [fixtures del motor](../crates/daw-engine/tests/mixer_contract.rs) verifican
muestras de referencia independientes, full scale con 16 voces, rutas capturadas,
gain/pan/mute, transiciones de pausa/Stop, retarget, cancelación/medidores y mono.
Una misma línea temporal con ediciones en frames exactos conserva todas las
muestras, cursores, rutas, estado y picos agregados entre bloques 1/127/256/4096,
en 800/44100/48000 Hz y salida mono/estéreo. Las ventanas individuales de medidores
son observaciones por llamada y no se confunden con estado musical invariante.

La [prueba instrumentada](../crates/daw-audio/tests/mixer_memory_contract.rs)
ejercita 100 ciclos de mezcla, rutas, controles, errores, pausa, fin natural y
Stop: cero asignaciones/bytes y cero variación neta, lo que verifica cero
liberaciones en procesamiento. El control positivo destruye cuatro recursos PCM
fuera de la región medida y observa las liberaciones esperadas.

## Verificación local

| Comprobación                                                                                        | Resultado                                                                                                   |
| --------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------- |
| `npm run check`                                                                                     | Correcto: bindings, TS/lint/frontend/formato/build, CLI, Rust headless/workspace, instrumentación y Clippy. |
| `cargo test -p daw-domain -p daw-engine --release --locked`                                         | Correcto.                                                                                                   |
| `cargo test -p daw-audio --release --features rt-diagnostics --locked --test mixer_memory_contract` | Correcto: cero asignaciones/liberaciones y control positivo correcto.                                       |
| `npm run desktop:build -- --debug --no-bundle`                                                      | Correcto, después de las suites; ejecutable debug sin instalador.                                           |

Logs locales bajo `target/stage10-2-20261111/`. El
[manifiesto 10.2](evidencia-mezclador-10.2.json) identifica fuentes y comandos/logs
con sus hashes. Las huellas/evidencia de etapa 9 se conservan como snapshots
históricos y no se atribuyen a este código nuevo. La huella nueva registra
125 archivos de código/configuración; nueve difieren respecto del snapshot
9.5.b, incluidos cinco archivos añadidos. Lockfiles, tipos generados y workflow
conservan sus hashes. No se creó commit/push ni CI hospedada.

Revisión final: formato y `git diff --check` correctos; 418 destinos locales de
enlaces en 19 documentos, sin ausentes. Los 125 hashes del manifiesto coinciden
con las fuentes finales después de las verificaciones.

## Próximo incremento

**10.3:** instalar ajustes del documento en sesiones mezcladas, enviar comandos
Copy por la FIFO, observar estado aplicado y agrupar picos sobre todos los chunks
de un callback. Después incorporar DTO/IPC/ACL generados y cuatro tiras de mezcla
con pruebas de aplicación/servicio/frontend. Cerrar 10 requiere esa integración,
verificación nativa propia y CI del SHA entregado; la escucha del sampler en
etapa 9 no acredita por sí sola la mezcla nueva.
