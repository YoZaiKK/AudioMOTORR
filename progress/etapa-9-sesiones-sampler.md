# Etapa 9.4.b — Sesiones de sampler y resultados en cola

Fecha: 2026-11-09. Implementación local posterior a 9.4.a; se conservan cambios,
reportes y manifiestos anteriores. Base Git 1002efa, sin commit/push o CI nueva.

Registro fechado de 9.4.b. La integración posterior de Application/DTO/IPC/UI
está implementada en [9.4.c](etapa-9-aplicacion-sampler.md); su nuevo manifiesto
identifica el código actual. Los resultados de este reporte se conservan.

## Implementación

AudioHandle añade initialize_manual_sampler, initialize_native_sampler y
trigger_sample. Recibe banco ya preparado; no carga archivos en controlador o
callback. Native selecciona config soportada con canales/rate exactos y prefiere
f32, sin remuestreo o fallback de dispositivo. Gain sirve para senoide y sampler.

Una variante Copy privada comparte la FIFO existente de 128 comandos para
EngineCommand o trigger slot/gain, con 32 por callback completo. EngineCommand
y EngineSnapshot no cambian. ServiceSnapshot Rust añade SamplerTelemetry opcional:
mask de slots, voces activas, último command ID/result y contadores saturantes
started/rejected. DTO, bindings y comandos IPC no cambian; el literal del test
IPC añade sampler: None para reflejar el nuevo campo Rust.

Aceptado no significa iniciado: errores estáticos de slot/source se rechazan
antes de encolar, sin consumir ID; NotPlaying/PoolFull se resuelven al procesar.
last_applied incluye rechazos. La observación es acumulativa/perdible: un último
resultado solo vale para su ID exacto. Una orden Stop posterior puede avanzar
ack sin sustituir el último resultado de trigger. Los contadores se reinician
por sesión, IDs del servicio no; no se promete log completo ni audibilidad.

AudioEngine::validate_render comprueba el bloque completo después de comandos
y antes de dividir scratch native. Así un overflow posterior no permite avanzar
fragmentos anteriores; conserva estado de render posterior a las órdenes.
Errores de alineación siguen detectándose antes de consumir comandos native.

## Cierre de recursos

El callback CPAL posee CallbackState con Box<Realtime>. Un guard NativeStream
cierra primero el stream y espera la devolución del estado por un buzón de
teardown de capacidad uno; el propietario destruye allí PCM. Si CPAL destruye su
closure en un worker, ese worker transfiere el estado, sin liberar los samples.
El mismo guard cubre error de build/start. El buzón no es una cola realtime:
render no lo toca, no bloquea ni mueve/destruye bancos.

No se revoca el estado con un timeout: teardown puede esperar a que el backend
libere el callback. El deadline de respuesta no limita llamadas/destrucción del
driver. Manual destruye procesador/banco en su propietario. Peticiones con banco
expiradas, busy o fallidas se descartan fuera del procesamiento. No se impone un
presupuesto global a bancos preasignados por callers/coordinadores independientes.
Ver [contrato técnico](../docs/sampler-service.md).

## Pruebas y alcance

Once funciones de prueba nuevas entre variantes de features:

- Cuatro [pruebas públicas](../crates/daw-audio/tests/sampler_service_contract.rs),
  una solo headless: aceptación/procesado/resultado, orden, límites de FIFO/pool,
  Pause/Stop/gain, shutdown/reinicio, IDs monotónicos, owner Drop y native desactivado.
- Realtime: cola de snapshots llena, recuperación del último ID correcto, última
  orden distinta de trigger y saturación de contadores.
- Control: fallo con disparos pendientes/reinicio e IDs agotados; banco expirado
  sin sesión; destrucción de PCM en propietario medida fuera de procesamiento.
- Native: selección exacta de config y preferencia f32; equivalencia bit a bit
  con manual, estados/outcomes y presupuesto 32 aun con 2001 frames divididos;
  devolución desde worker y ruta de build fallido simulada. Con diagnóstico se
  mide cero asignaciones/liberaciones en procesamiento y cero liberaciones de
  PCM en worker de teardown, con destrucción medida en propietario.

Se amplían pruebas previas de overflow para comprobar preflight inmutable del
motor. Regresiones de silencio/senoide siguen pasando. La suite con native y
diagnóstico pasa 26 pruebas (una física ignorada); no abre dispositivos. No se
afirma prueba de driver real, deadlines, RSS o continuidad acústica.

```powershell
cargo test -p daw-audio --no-default-features --locked --test sampler_service_contract
cargo test -p daw-audio --release --no-default-features --locked --test sampler_service_contract
cargo test -p daw-audio --features native-audio,rt-diagnostics --locked
cargo test -p daw-audio --release --features native-audio,rt-diagnostics --locked
cargo test -p daw-engine --release --no-default-features --locked --lib --test sampler_engine_contract
cargo clippy -p daw-audio --no-default-features --all-targets --locked -- -D warnings
cargo clippy -p daw-audio --features native-audio,rt-diagnostics --all-targets --locked -- -D warnings
npm run check
npm run desktop:build -- --debug --no-bundle
```

Resultados locales del 2026-11-09:

- Focales debug/release y Clippy correctos; suite native+diagnóstico: 26 pruebas
  correctas y una física ignorada. La variante headless pública pasa cuatro pruebas.
- npm run check correcto: 101 pruebas headless, 108 workspace, 26 audio
  instrumentadas, ocho memoria decoder/banco, 13 frontend y nueve escenarios CLI.
  Suites solapadas; no sumarlas como pruebas únicas. Bindings, TypeScript,
  ESLint, formato, build frontend, rustfmt y Clippy correctos.
- npm run desktop:build -- --debug --no-bundle correcto después del check:
  target/debug/rust-daw-desktop.exe, sin instalador o hardware abierto.
- Cargo.lock, package-lock.json y api.ts generado mantienen sus hashes previos.
  Sin dependencias, workflow o smoke físico nuevos.

El [manifiesto actual](evidencia-sesiones-9.4b.json) registra 112 archivos de
código/configuración y huella conjunta. Los manifiestos 9.3/9.4.a conservan sus
snapshots históricos. Logs locales no versionados en target/stage9-4b/check.log
y desktop-build.log; hashes comparados en hashes-before.json/hashes-after.json.
No hay publicación/CI remota nueva. La validación final pasa formato, 418 destinos
locales de enlaces/imágenes Markdown, diez límites del contrato, actionlint y
git diff --check. Los 112 archivos del manifiesto coinciden con sus hashes.

## Siguiente paso: 9.4.c

Conectar preparación/sesiones/disparos mediante Application, DTO y comandos IPC/ACL;
generar tipos TypeScript y añadir controles UI explícitos con pruebas mock. Exponer
IDs/counters como strings y distinguir pendiente, procesado y resultado exacto
o desconocido si la observación se perdió. Mantener rutas/decodificación fuera
del servicio/RT y no introducir persistencia (etapa 12). Después 9.4.d integra
regresiones/evidencia; 9.5 mide sampler nativo y cierra etapa 9, aún abierta.
