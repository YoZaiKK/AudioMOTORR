# Etapa 9.5.a — Diagnósticos de sampler

Fecha: 2026-11-10. Cambios locales sobre 1002efa, conservando 9.3/9.4 y sus
manifiestos fechados. No se abre hardware, no hay capturas, commit/push, publicación
o CI remota nueva. 9.5.b realizará mediciones físicas; la etapa 9 sigue abierta.

## Implementación

- Feature offline-diagnostics de CLI sin CPAL; rt-diagnostics conserva audio
  nativo e incluye la anterior. Sin dependencias nuevas. Comandos release
  sampler-bench y sampler-native-check, ayuda y validación antes de I/O.
- Benchmark de 12 combinaciones (bloques 1/127/256/4096, voces 0/1/16), mono o
  estéreo con canales distintos. Banco/buffers/timing preasignados, warmup,
  retrigger por bloque tras finalización, oráculo independiente de cada muestra,
  actividad real, percentiles exactos por rango más cercano y overruns observados.
- Contador compartido MemoryActivity detecta asignaciones y liberaciones, incluidas
  las de recursos previos a medir. El benchmark exige cero actividad de memoria en
  procesamiento y una liberación PCM exacta después, como control positivo.
- Instrumentación nativa opcional añade liberaciones/bytes, frames, histograma
  fijo de 64 bins y probes en todos los canales. p50/p99 nativos son cotas superiores
  aproximadas; max conserva el máximo observado. DTO/IPC/desktop no cambian.
- Runner nativo requiere WAV y device ID explícitos. Application prepara y ofrece
  metadatos Rust del recurso listo. Confirma IDs, voces/frames, gain, Pause congelado,
  resume, Stop/PCM retenido, retrigger, finalización natural y reapertura con
  contadores nuevos. Pruebas con WAV real y backend manual, sin dispositivo.
  También confirma por ID NotPlaying antes de Play y PoolFull con 16 voces activas.
  La sesión exige recurso de seconds+4 a 15 s, dentro de los límites WAV existentes.
- El runner valida métricas acumuladas de toda la sesión; no afirma percentiles
  aislados por ventana ni continuidad acústica. El hash del WAV se registra fuera
  del runner. Las liberaciones de teardown se separan de procesamiento.

Ver [contrato de diagnósticos](../docs/sampler-diagnostics.md) y
[protocolo nativo actualizado](../docs/sampler-native-verification.md).

## Pruebas y mediciones locales

Focales debug/release pasan: matriz/oráculo/allocator, lifecycle manual 0/1/16,
argumentos, feature/profile gates, native deshabilitado antes de abrir archivo,
rechazo de métricas inválidas, cuantiles vacíos/límites y probes estéreo/contadores.
Clippy pasa con offline-diagnostics y rt-diagnostics. cargo tree confirma ausencia
de CPAL en la feature offline. Release CLI pasa ocho unitarias y dos de proceso.

Benchmark release ejecutado sin dispositivo: 12 casos estéreo × 2000 iteraciones
y 12 mono × 1000. Ambos registran cero asignaciones/liberaciones dentro de la
región medida y destrucción PCM positiva posterior. Logs sin versionar:
target/stage9-5a/bench-stereo.log y bench-mono.log. Son mediciones CPU offline,
incluyen timer y no miden jitter, driver, salida física o garantía de deadline.

Ejemplos observados, 48 kHz, bloque 4096 y 16 voces:

| Salida  | Iteraciones | p50 ns | p99 ns | Máximo ns | Máximo / duración nominal | Overruns |
| ------- | ----------- | ------ | ------ | --------- | ------------------------- | -------- |
| Estéreo | 2000        | 208500 | 354200 | 423300    | 0.004961                  | 0        |
| Mono    | 1000        | 188000 | 355400 | 601000    | 0.007043                  | 0        |

Los otros bloques y cualquier outlier se conservan en los logs. El benchmark
reporta overruns; no se reitera para esconderlos. El contador voice_frames refleja
colas silenciosas de bloques finales cuando el recurso termina dentro del bloque.
La tabla usa la repetición tras añadir las comprobaciones de rechazo del runner;
las mediciones iniciales se conservan en bench-stereo-initial.log y bench-mono-initial.log.

Comandos:

```powershell
cargo test -p daw-cli --release --features offline-diagnostics --locked --bin daw-cli --test diagnostics_contract
cargo test -p daw-audio --release --features native-audio,rt-diagnostics --locked
cargo build -p daw-cli --release --features rt-diagnostics --locked
cargo run -p daw-cli --release --features offline-diagnostics --locked -- sampler-bench --iterations 2000
cargo run -p daw-cli --release --features offline-diagnostics --locked -- sampler-bench --iterations 1000 --channels 1
npm run check
npm run desktop:build -- --debug --no-bundle
```

Resultados finales locales del 2026-11-10:

- npm run check correcto: 108 pruebas headless, 114 workspace, 29 audio
  instrumentadas, nueve diagnósticos CLI offline, ocho memoria decoder/banco,
  24 frontend y nueve escenarios CLI. Suites solapadas; no sumar como únicas.
- Bindings deterministas, TypeScript, ESLint, formato, build frontend, rustfmt y
  Clippy correctos. La prueba física sigue ignorada.
- Build Windows debug sin bundle correcto después del check:
  target/debug/rust-daw-desktop.exe. El runner CLI nativo también compila en release.
- Cargo.lock, package-lock.json, api.ts generado y workflow mantienen hashes de
  9.4.d y sus valores antes/después de check/build. Se añade una feature, no dependencias.

El [manifiesto actual](evidencia-diagnosticos-9.5a.json) identifica 119 archivos de
código/configuración y 13 nuevos/modificados respecto de 9.4.d. Registra hashes
de los logs de benchmark y del binario offline medido antes de reconstruir CLI
con features nativas. Logs locales no versionados en target/stage9-5a/check.log,
desktop-build.log, release-cli.log, release-audio.log y native-build.log.
La revisión final pasa formato, 550 destinos locales de enlaces/imágenes Markdown,
diez límites del contrato, actionlint y git diff --check. Los 119 archivos del
manifiesto coinciden con sus hashes. No implica ejecución remota ni audición física.

## Siguiente: 9.5.b

Ejecutar el runner release en un dispositivo/configuración identificados con WAV
conocidos, hashes antes/después y ventanas 0/1/16; registrar salidas completas,
asignaciones/liberaciones, histogramas/tiempos y resultados exactos/inconclusos.
Separar audición física de muestras generadas y de loopback. 9.5.c consolidará
aceptación, límites de plataformas/desconexión/estrés y CI del SHA publicado.
No atribuir los checks de 7acc1db o mediciones de senoide a este sampler.
