# Etapa 9.4.a — Sampler integrado en AudioEngine

Reporte fechado de motor. 9.4.b ya conecta sesiones/outcomes de audio; ver
[reporte posterior](etapa-9-sesiones-sampler.md). El manifiesto 9.4.a conserva
el snapshot anterior y no representa los fuentes posteriores.

Fecha: 2026-11-09. Incremento local sobre preparación 9.3 cerrada, conservando
los cambios previos y su manifiesto histórico. Base Git documental 1002efa;
sin commit/push o CI remota nueva que valide este código.

## Implementación

[AudioEngine](../crates/daw-engine/src/lib.rs) añade with_sampler(bank, tempo),
que posee el SampleBank validado y deriva su configuración. Aplica headroom
1/16 después de suma raw y reutiliza GainRamp de 5 ms (ceil(rate/200) frames),
una vez por frame y compartida por canales.

trigger_sample(slot, Gain) devuelve Result<usize, SamplerTriggerError> Copy:
Ok(index) inicia la voz libre más baja en el próximo frame reproducido. Errores
en orden NoSampler, InvalidSlot, MissingAsset, NotPlaying, PoolFull; rechazo sin
mutación. Es resultado directo, no aceptación de cola o telemetría del servicio.
EngineCommand, EngineSnapshot, DTO y bindings conservan sus campos/variantes.
sampler() ofrece solo inspección prestada del banco/cursores.

Play no dispara o reinicia voces. Solo se dispara con transporte real y solicitado
Playing. Pause avanza durante release y congela supervivientes; Play reanuda
con ataque. Stop termina release, limpia voces y posición de transporte sin
liberar PCM o reiniciar reloj raw. A gain efectivo cero, Pause/Stop son inmediatos.
Play durante release cancela el destino previo sin perder cursores. BPM no cambia
pitch/velocidad. Contadores de motor y reloj raw se validan antes de avanzar
fuente/envolvente: alineación/overflow silencian todo y preservan el estado.

Integración de biblioteca/offline: servicio manual/CPAL no construye aún este
modo. No se añaden comandos/controles UI, dependencias o hardware abierto.

## Evidencia

Diez funciones de prueba nuevas y ampliación del overflow anterior:

- Siete [pruebas de motor](../crates/daw-engine/tests/sampler_engine_contract.rs):
  resultados/orden, 16 voces con headroom, rampas a 48 kHz/44.1 kHz, suma mono y
  estéreo/retarget, transporte, fin natural e interrupción de release, particiones
  1/127/256/4096, rollback y BPM. Oráculos a 800 Hz usan rampas de cuatro frames
  para fracciones exactas; no implican soporte de dispositivo a ese rate.
- Una prueba interna fuerza overflow del reloj raw integrado. El test previo
  de contadores añade sampler a los casos silencio/senoide/rampas.
- Una prueba de [carga](../crates/daw-app/tests/loaded_sampler_contract.rs)
  recorre WAV → banco → AudioEngine con headroom/ataque/release/Stop, borrando
  el archivo antes de reproducir. La suite pasa de cuatro a cinco pruebas.
- Una prueba [instrumentada](../crates/daw-audio/tests/sampler_memory_contract.rs)
  repite 100 ciclos de disparos/agotamiento, volumen, errores, render, Pause/Stop
  y fin natural. Debug/release miden cero asignaciones, bytes y liberaciones
  durante procesamiento. Destruir el motor aparte libera cuatro buffers/16468
  bytes como control positivo. Se conserva el test raw previo; suite de dos.

No mide deadlines, hardware/loopback o RSS. Recursos asignados antes de medir;
el destructor queda fuera de procesamiento. Regresiones de silencio/senoide correctas.

```powershell
cargo test -p daw-engine --no-default-features --locked --lib --test sampler_engine_contract
cargo test -p daw-engine --release --no-default-features --locked --lib --test sampler_engine_contract
cargo test -p daw-app --no-default-features --locked --test loaded_sampler_contract
cargo test -p daw-app --release --no-default-features --locked --test loaded_sampler_contract
cargo test -p daw-audio --features rt-diagnostics --locked --test sampler_memory_contract
cargo test -p daw-audio --release --features rt-diagnostics --locked --test sampler_memory_contract
cargo clippy -p daw-engine -p daw-app --no-default-features --all-targets --locked -- -D warnings
cargo clippy -p daw-audio --features rt-diagnostics --all-targets --locked -- -D warnings
npm run check
npm run desktop:build -- --debug --no-bundle
```

Resultados locales del 2026-11-09:

- Focales debug/release, regresiones y Clippy correctos.
- npm run check correcto: 94 pruebas headless, 99 workspace, 16 audio
  instrumentadas, ocho memoria decoder/banco, 13 frontend y nueve escenarios CLI.
  Las suites se solapan; no sumarlas como pruebas únicas. Correctos también
  bindings, TypeScript, ESLint, formato, build frontend, rustfmt y Clippy.
- npm run desktop:build -- --debug --no-bundle correcto después del check:
  target/debug/rust-daw-desktop.exe, sin instalador ni smoke físico nuevo.
- Cargo.lock, package-lock.json y api.ts generado conservan los hashes previos.
  No se añaden dependencias, workflow o comandos de hardware.

El [manifiesto actual](evidencia-integracion-9.4a.json) registra 111 archivos de
código/configuración y su huella conjunta. No es commit ni CI remota. Logs no
versionados: target/stage9-4a/check.log y desktop-build.log; comparación de hashes
en hashes-before.json/hashes-after.json del mismo directorio. La huella histórica
de 9.3 se conserva y ya no identifica el motor actual. Las suites existentes
descubren las nuevas pruebas, sin nuevos pasos CI. Validación documental final:
Prettier, 393 destinos locales Markdown/imagen, diez límites numéricos del
contrato, actionlint y git diff --check correctos. Los 111 hashes del manifiesto
actual coinciden. No se atribuye CI histórica a este código sin publicar.

## Siguiente paso: 9.4.b

Conectar el modo a sesiones manual/CPAL: construir/mover el banco antes del
procesamiento y cerrar stream antes de destruirlo/reemplazarlo. Añadir disparos
en FIFO y outcomes Copy acotados con command ID, Started/rechazo y contadores,
distinguiendo encolado/procesado/iniciado. Mantener 128 comandos/32 por callback
completo y verificar equivalencia manual/native, reinicio, fallo y teardown.
App/DTO/IPC/UI en 9.4.c; tiempos nativos/cierre total en 9.5. Etapa 9 sigue abierta.
