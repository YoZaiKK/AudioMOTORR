# Etapa 7 — Incremento de oscilador en motor

Reporte histórico de 7.2. La exportación descrita aquí como siguiente trabajo ya
se implementó en [7.3](etapa-7-exportacion.md); consultar ese reporte y el checklist
para el estado vigente. Se conservan las cifras y límites de este incremento.

Fecha: 2026-11-07. Estado: 7.0–7.2 completados para este incremento local;
7.3–7.5 pendientes. La etapa 7 completa permanece abierta. Se conservan los
cambios documentales previos y los reportes/capturas históricos.

## Base CI confirmada (7.0)

API oficial de GitHub consultada el 2026-11-07 para el SHA
`1ef9bd0afb35bf380004f87c65ca7ccace132357`.

| Ejecución                                                                                           | Resultado                                                                            |
| --------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------ |
| [Run 42314334447](https://github.com/YoZaiKK/AudioMOTORR/actions/runs/42314334447)                  | completed / success; creada 2026-11-06 23:49:02 UTC.                                 |
| [headless-linux](https://github.com/YoZaiKK/AudioMOTORR/actions/runs/42314334447/job/128223168780)  | success: formato, cinco paquetes, Clippy, nueve escenarios y lockfiles.              |
| [desktop-windows](https://github.com/YoZaiKK/AudioMOTORR/actions/runs/42314334447/job/128223168415) | success: prerrequisitos, npm ci, bindings, check, desktop debug y archivos intactos. |

La conexión requirió ejecución autorizada tras fallar en sandbox. Esta evidencia
confirma Ubuntu headless y Windows desktop de la base publicada. No confirma
compilación Linux nativa, macOS, hardware ni las modificaciones locales de hoy.

## Contrato y código (7.1–7.2)

- Constructor nuevo: AudioEngine::with_sine(config, tempo, frequency_hz, amplitude),
  con parámetros f64 y errores InvalidFrequency/InvalidAmplitude. No se añade
  dependencia ni cambia lockfile, DTO, UI o backend.
- Módulo interno oscillator.rs: fase f64 normalizada, muestras f32, fase por frame
  y misma muestra por canal. Frecuencia finita positiva por debajo de Nyquist;
  amplitud finita entre cero y uno. La elección de escritor WAV queda en 7.3.
- Play reanuda fase, Pause silencia y conserva fase/posición, Stop reinicia ambas;
  BPM no cambia frecuencia. Validaciones de render preceden a cualquier avance.
- AudioEngine::new mantiene silencio, incluidos servicio nativo, backend manual
  y CLI existentes. La señal está disponible solo mediante API explícita del motor.
- Seis tests nuevos cubren señal contra referencia con tolerancia absoluta 1e-6,
  sample rates 44100/48000/96000, canales 1/2/6, validación, amplitud cero,
  transporte, BPM, errores, buffers vacíos y particiones 1/127/256/4096.
- Las particiones comparan todas las muestras, snapshot y una cola de muestras
  posterior para detectar diferencias de fase. Igualdad exacta en mismo build;
  no se promete identidad de bits entre plataformas.
- La prueba interna de overflow ahora comprueba ambos contadores y fase tanto
  en silencio como con oscilador. Inspección del render: sin asignaciones, I/O,
  locks o serialización; sin instrumentación/timing aún.

## Verificación local

- cargo test -p daw-engine --no-default-features --locked: 13 tests correctos
  (seis nuevos, seis previos de contrato y un unit test ampliado).
- cargo clippy -p daw-engine --no-default-features --all-targets --locked -- -D warnings:
  correcto. Rustfmt aplicado.
- npm run check: correcto; 12 pruebas frontend, 31 Rust headless, 35 Rust workspace
  y nueve escenarios CLI. Incluye bindings, TypeScript, ESLint, Prettier, build
  Vite, rustfmt y Clippy de ambos alcances con -D warnings. Las suites Rust se
  solapan; no sumar sus cantidades como cobertura independiente.
- La prueba de hardware permanece ignorada por defecto. No se abre audio ni se
  actualizan capturas. npm run desktop:build -- --debug --no-bundle: correcto
  después del check, ejecutable target/debug/rust-daw-desktop.exe sin instalador.
- Documentación: Prettier y destinos locales Markdown/imágenes comprobados;
  git diff --check sin errores. El contrato implementado y la planificación
  distinguen señal de motor existente, CLI silenciosa y WAV pendiente.

## Siguiente paso y límites

Implementar 7.3: render acotado independiente en daw-app y escritura WAV float32
desde CLI. Todavía no hay comando de exportación, reproducción nativa de la señal,
rampas, sampler o secuenciador. Las pruebas nuevas ya quedan incluidas por
selección de paquete en check y CI; no requieren modificar el workflow.

Los hashes iniciales de Cargo.lock, package-lock.json y api.ts son los del
[cierre de etapa 6](cierre-etapa-6.md), idénticos tras check y build. No hubo
cambio de dependencias o tipos generados. Los cambios de hoy permanecen locales;
ninguna ejecución remota de ellos se afirma como correcta sin publicarlos.
