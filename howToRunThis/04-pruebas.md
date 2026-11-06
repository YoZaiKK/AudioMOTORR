# Ejecutar las pruebas

## Comprobación completa

```powershell
npm ci
npm run check
```

Incluye bindings, TypeScript, lint, tests React, formato, build Vite, rustfmt,
tests/Clippy headless y tests/Clippy del workspace. Falla en el primer error.
Necesita prerrequisitos desktop; las pruebas automáticas no abren audio real.

## Por capa

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
como si todas fueran pruebas distintas. Linux/macOS, desconexión física y medición
de deadlines siguen fuera de la evidencia local.

## Automatización de etapa 6

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
