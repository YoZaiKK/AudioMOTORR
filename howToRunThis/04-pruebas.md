# Ejecutar las pruebas

## Diagnósticos sampler 9.5.a

```powershell
cargo test -p daw-cli --features offline-diagnostics --locked --bin daw-cli --test diagnostics_contract
cargo test -p daw-cli --release --features offline-diagnostics --locked --bin daw-cli --test diagnostics_contract
cargo test -p daw-audio --features native-audio,rt-diagnostics --locked
cargo build -p daw-cli --release --features rt-diagnostics --locked
npm run check
npm run desktop:build -- --debug --no-bundle
```

Prueban oráculo estéreo distinto, actividad/retrigger, allocator/free, metadatos
del banco preparado, recorrido manual y validaciones antes de I/O/hardware.
check ejecuta también la variante offline-diagnostics; release verifica la CLI
real y el gate NativeAudioDisabled. No abre hardware. Ver
[reporte actual](../progress/etapa-9-diagnosticos-sampler.md) y [CLI](03-cli.md).

## Sampler en Application/IPC/UI 9.4.c

```powershell
cargo test -p daw-app --no-default-features --locked --lib --test sampler_application_contract
cargo test -p rust-daw-desktop --locked --test ipc_contract
npm run bindings:generate
npm run bindings:check
npm run test:frontend
npm run check
npm run desktop:build -- --debug --no-bundle
```

Comprueban worker de WAV real, banco/sesión manual, cancelación/IDs obsoletos,
fallos, cierre, resultados exactos, DTO/strings, ACL y carreras de frontend.
No abren hardware. Los tipos generados cambian deliberadamente; los lockfiles
deben mantenerse. Ver [reporte actual](../progress/etapa-9-aplicacion-sampler.md),
[contrato](../docs/sampler-desktop.md) y [uso del sampler](06-sampler.md).
La [auditoría 9.4.d](../progress/cierre-integracion-9.4.md) cierra integración local
y añade tres regresiones de recuperación de dispositivos/polling. Siguiente:
runner diagnóstico 9.5.a implementado; mediciones físicas 9.5.b y cierre 9.5.c pendientes.

## Sesiones y outcomes 9.4.b

```powershell
cargo test -p daw-audio --no-default-features --locked --test sampler_service_contract
cargo test -p daw-audio --features native-audio,rt-diagnostics --locked
cargo test -p daw-audio --release --features native-audio,rt-diagnostics --locked
```

Comprueban encolado/procesado/inicio o rechazo, límites e IDs, shutdown/fallo/
reinicio, banco expirado, configuración exacta, equivalencia de chunks y cierre
de PCM en propietario. Native aquí prueba adaptadores y teardown simulado, sin
abrir dispositivos; la prueba física existente sigue ignorada. Ver
[reporte actual](../progress/etapa-9-sesiones-sampler.md) y
[contrato técnico](../docs/sampler-service.md). DTO/IPC/UI se incorporan en 9.4.c.

## Sampler en AudioEngine 9.4.a

```powershell
cargo test -p daw-engine --no-default-features --locked --lib --test sampler_engine_contract
cargo test -p daw-engine --release --no-default-features --locked --lib --test sampler_engine_contract
cargo test -p daw-audio --features rt-diagnostics --locked --test sampler_memory_contract
cargo test -p daw-audio --release --features rt-diagnostics --locked --test sampler_memory_contract
```

Siete pruebas de motor más overflow interno, caso WAV → AudioEngine y medición
de cero asignaciones/liberaciones durante procesamiento. Son pruebas offline;
sesiones y disparos en cola del servicio ya se conectan en 9.4.b. Ver
[reporte actual](../progress/etapa-9-integracion-motor.md).

## Lector WAV, banco y reproducción offline 9.3.b–e

```powershell
cargo test -p daw-app --no-default-features --locked --test sample_loading_contract
cargo test -p daw-app --no-default-features --locked --lib --test bank_preparation_contract
cargo test -p daw-app --release --no-default-features --locked --lib --test bank_preparation_contract
cargo test -p daw-app --no-default-features --locked --test loaded_sampler_contract
cargo test -p daw-app --release --no-default-features --locked --test loaded_sampler_contract
cargo test -p daw-cli --no-default-features --locked --test export_contract
cargo test -p daw-cli --features rt-diagnostics --locked --test sample_loading_memory_contract
```

Comprueban formatos y contenedor, límites, fallos/cancelación/cierre, carga exacta
de los WAV del exportador y memoria del decoder/banco. Se añaden validación de
slots, orden, permiso compartido, resultado pendiente y coexistencia de bancos
estéreo máximos. 9.3.d añade 17 casos de fallo/cancelación, entrega sincronizada
y Read suspendido. check ejecuta las ocho pruebas instrumentadas explícitamente,
incluida liberación de PCM en trabajador/receptor.
No hay comando CLI de importar ni control de UI nuevo; son APIs de biblioteca.
Ver [reporte del lector](../progress/etapa-9-lectura-wav.md) y
[reporte del banco](../progress/etapa-9-preparacion-banco.md) y
[reporte de fallos/cierre](../progress/etapa-9-fallos-cancelacion.md).
9.3.e verifica 18 combinaciones formato/header/canales sobre cuatro particiones,
todos los códigos PCM16, bancos y límites con archivos reales; ver
[reporte actual](../progress/etapa-9-fixtures-integracion.md) y
[matriz de aceptación](../docs/sample-loading-acceptance.md). Preparación cerrada
localmente en [9.3.f](../progress/cierre-preparacion-9.3.md). AudioEngine integrado
offline en 9.4.a; servicio/desktop siguen en 9.4.b–d.

El cierre conserva [hashes de código/configuración](../progress/evidencia-preparacion-9.3.json)
y reutiliza check/build de 9.3.e, ya que solo cambió documentación. Es evidencia
histórica: ese manifiesto ya no coincide con los fuentes posteriores de 9.4.a.
El reporte nuevo registra sus checks; CI nueva requiere publicar/inspeccionar.

## Comprobación completa

```powershell
npm ci
npm run check
```

Incluye bindings, TypeScript, lint, tests React, formato, build Vite, rustfmt,
tests/Clippy headless y tests/Clippy del workspace. Falla en el primer error.
Necesita prerrequisitos desktop; las pruebas automáticas no abren audio real.

## Por capa

La nueva base del sampler (9.1) se comprueba sin hardware ni archivos de audio:

```powershell
cargo test -p daw-engine --no-default-features --locked --test sample_resource_contract
```

Cuatro pruebas verifican recursos PCM mono/estéreo, bits y metadatos conservados,
errores de entrada, límite por frames y compatibilidad de salida. Los fixtures
son arrays/generadores en memoria; no son proyectos importables ni archivos que
la interfaz pueda cargar. Aún no prueban voces o reproducción del sampler.
Ver [reporte](../progress/etapa-9-sampler.md) y [contrato](../docs/sampler-contract.md).
check y los jobs existentes descubren estas pruebas al seleccionar daw-engine.

El kernel de voces (9.2) se verifica también sin dispositivos:

```powershell
cargo test -p daw-engine --no-default-features --locked --test sampler_contract
cargo test -p daw-audio --features rt-diagnostics --locked --test sampler_memory_contract
cargo test -p daw-audio --release --features rt-diagnostics --locked --test sampler_memory_contract
```

Ocho pruebas de kernel y una interna de overflow comprueban muestras exactas,
suma por orden de voz, mapeo de canales, superposición, agotamiento, reutilización,
clear/retrigger, errores y particiones. La prueba instrumentada mide asignaciones
y liberaciones, con una destrucción separada que verifica que el contador sí
detecta liberar recursos. No abre hardware ni demuestra deadlines nativos.
check ya ejecuta la variante instrumentada; el job headless sin features la
omite y sí ejecuta los tests de comportamiento. Ver [reporte 9.2](../progress/etapa-9-voces.md).

```powershell
npm run test:frontend
npm run typecheck
npm run lint
npm run format:check
cargo fmt --all -- --check
cargo test -p daw-domain -p daw-engine -p daw-audio -p daw-app -p daw-cli --no-default-features --locked
cargo clippy -p daw-domain -p daw-engine -p daw-audio -p daw-app -p daw-cli --no-default-features --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
node howToRunThis/run-cli-cases.mjs
```

La selección headless excluye Tauri y CPAL. El workspace incluye Tauri y habilita
audio nativo mediante desktop, sin abrir hardware en tests normales. El mock runtime
comprueba el handler IPC y los permisos reales, pero no crea una ventana WebView2 real.

Las pruebas cubren transporte y particiones, colas llenas y FIFO, telemetría,
aceptación/aplicación, cierre/reinicio, concurrencia, errores, argumentos CLI,
precisión JSON, limpieza StrictMode y tasa de publicación. Etapa 5 añade comandos
mezclados al cruzar 32 por callback y cierre con cambios de tempo pendientes.

## Bindings y lockfiles

```powershell
npm run bindings:check
npm run bindings:generate
npm run bindings:check
Get-FileHash Cargo.lock,package-lock.json,src/shared/api/generated/api.ts -Algorithm SHA256
```

`bindings:check` no modifica archivos; generate sí escribe el contrato derivado de
Rust. Sin cambios DTO, su hash debe ser idéntico antes/después. Comparar también
lockfiles antes y después de npm ci y las comprobaciones locked. No usar
generate-lockfile o npm install para disimular un fallo de reproducibilidad.

## Pruebas con hardware

```powershell
cargo test -p daw-audio --features native-audio --locked --test service_contract native_failure_recovery_and_restart_on_same_service -- --ignored
npm run desktop:build -- --debug --no-bundle
powershell -NoProfile -File scripts/desktop-smoke.ps1
```

La primera está ignorada por defecto porque necesita un dispositivo; un test
ignorado no es un fallo. La segunda secuencia abre la ventana y comprueba la UI
real. Ver [walkthrough](02-interfaz.md) para actualizar capturas explícitamente.

El reporte de etapa 5 está en [resultados](resultados.md); cuenta suites por
separado, porque headless y workspace comparten pruebas. No sumar sus cantidades
como si todas fueran pruebas distintas. Linux nativo/macOS, desconexión física y
loopback acústico siguen fuera de la evidencia. La base 1ef9bd0 ya tiene
[CI headless Ubuntu y desktop Windows correcta](https://github.com/YoZaiKK/AudioMOTORR/actions/runs/42314334447).

## Pruebas del oscilador de motor (7.2)

```powershell
cargo test -p daw-engine --no-default-features --locked --test oscillator_contract
```

Seis pruebas verifican señal contra referencia, validación, fase/transporte,
particiones, errores y BPM independiente. El overflow unitario también protege
fase. npm run check ya descubre estos tests sin cambiar el workflow. El
[reporte actual](../progress/etapa-7-oscilador.md) separa CI de la base de las
pruebas locales nuevas. Los smoke ordinarios continúan en silencio; desktop
permite desde etapa 8 seleccionar explícitamente una senoide.

## Pruebas de exportación WAV (7.3)

```powershell
cargo test -p daw-app --no-default-features --locked --test offline_contract
cargo test -p daw-cli --no-default-features --locked --test export_contract
cargo test -p daw-cli --no-default-features --locked --bin daw-cli
```

Comprueban samples exactos del motor, metadatos, mono/estéreo, frames cero y
bloques parciales, memoria por bloques e independencia de la app viva. Además:
destinos existentes y rutas inválidas, límites RIFF/byte rate y errores inyectados
de header, muestras, seek/flush final con cierre del escritor. Los archivos se
crean en directorios temporales exclusivos; no requieren hardware.

check y ambos jobs CI ya ejecutan estas pruebas al seleccionar daw-app/daw-cli;
los nueve escenarios Node siguen intactos. Cargo.lock incluye el cambio
deliberado hound 3.5.1 en CLI; no se compara su nuevo hash con el antiguo para
afirmar que no cambió. Tras fijar dependencias, los checks locked deben mantenerlo.
Ver [reporte WAV](../progress/etapa-7-exportacion.md).

Etapa 7 cerrada: el commit c6cf627 pasó ambos jobs en
[run 42419963899](https://github.com/YoZaiKK/AudioMOTORR/actions/runs/42419963899).
La [auditoría de cierre](../progress/cierre-etapa-7.md) registra el SHA exacto y
las verificaciones locales. Los reportes previos conservan el estado observado
al realizar cada incremento; su CI pendiente no describe el cierre actual.

## Automatización de etapa 6

Etapa 8 añade cuatro pruebas de rampas en daw-engine, una de servicio de senoide,
una del procesamiento nativo por chunks, y cobertura IPC/frontend de fuente y
volumen. check descubre estas pruebas y además ejecuta daw-audio con
native-audio,rt-diagnostics y Clippy del CLI diagnóstico. El test nativo compara
muestras con referencia y cuenta asignaciones; no abre un dispositivo.

Para verificar a mano la instrumentación y luego la UI real:

```powershell
cargo test -p daw-audio --features native-audio,rt-diagnostics --locked
cargo clippy -p daw-cli --features rt-diagnostics --all-targets --locked -- -D warnings
npm run check
npm run desktop:build -- --debug --no-bundle
powershell -NoProfile -File scripts/desktop-smoke.ps1 -Sine
```

Las mediciones release con hardware son opcionales y se describen en
[CLI](03-cli.md); sus resultados actuales están en el [reporte de etapa 8](../progress/etapa-8.md).
La nueva captura se guarda en target/desktop-stage8.png, sin sustituir las siete
imágenes históricas. El backend, mock IPC y frontend son coberturas diferentes.

El [workflow](../.github/workflows/ci.yml) comprueba el núcleo en Ubuntu 24.04 y
frontend/desktop en Windows Server 2022. El job Windows usa el mock runtime
existente, ejecuta check y después build debug sin instalador. Ningún job abre
hardware ni actualiza capturas. Se comprueban los lockfiles y tipos generados.

Consultar [desarrollo y CI](../docs/development.md), el
[contrato principal](../docs/realtime-contract.md) y el
[reporte de etapa 6](../progress/cierre-etapa-6.md). La sintaxis/expresiones se
validan con actionlint; los resultados remotos de cada revisión se consultan en
[Actions](https://github.com/YoZaiKK/AudioMOTORR/actions/workflows/ci.yml).
Las comprobaciones locales no confirman el resultado remoto. El reporte de etapa 5 se conserva
como evidencia fechada, sin sustituir sus capturas o mediciones.
