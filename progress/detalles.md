# Detalles de implementación

Este archivo conserva el registro granular de cada etapa. El estado resumido se
mantiene en [checklist.md](checklist.md). Los requisitos originales permanecen en
[rust-daw-bootstrap-prompt.md](rust-daw-bootstrap-prompt.md).

## Etapa 1 — Preparar el proyecto

Estado: completada. Inicio: 2026-11-04. Cierre: 2026-11-05.

### Inspección y alcance

- El repositorio contenía únicamente `README.md`, `.git/` y el documento en
  `prompt/`; no existía una aplicación que conservar ni configuraciones de paquetes.
- No se encontraron `AGENTS.md` aplicables en el repositorio ni en sus directorios
  ascendentes inspeccionados.
- Se renombró `prompt/` a `progress/` conservando el documento original.
- Esta etapa prepara una aplicación compilable. El motor, dispositivos, transporte
  y contratos IPC completos corresponden a las siguientes etapas.
- El workspace empieza con `daw-app` y `rust-daw-desktop`, que tienen uso real.
  `daw-domain`, `daw-engine`, `daw-audio` y `daw-cli` se incorporarán cuando se
  implemente su responsabilidad en las etapas 2 y 3, sin paquetes vacíos.

### Decisiones iniciales

- Proyecto JavaScript en la raíz y adaptador desktop en `src-tauri/`.
- Workspace Cargo virtual; metadatos, dependencias y perfiles compartidos.
- Rust edition 2024, resolver 3 y versión Rust fijada.
- npm como único gestor JavaScript, versiones directas exactas y Node fijado.
- TypeScript 5.9.3 se selecciona por compatibilidad con `typescript-eslint`:
  la versión consultada del linter admite TypeScript menor que 6.1, mientras
  `typescript@latest` consultado devuelve 7.0.2.
- La pantalla inicial muestra el estado de preparación; no simula audio.

### Prerrequisitos observados

- Rust y Cargo instalados: 1.90.0, host `x86_64-pc-windows-msvc`.
- Node 22.23.4 y npm 10.9.9.
- Visual Studio 2022 Build Tools detectado mediante `vswhere` con el componente
  `Microsoft.VisualStudio.Component.VC.Tools.x86.x64`.
- WebView2 Runtime 156.0.4314.58 detectado en el registro.
- La primera consulta npm falló por permisos sobre su caché fuera del workspace;
  se reintentó mediante el mecanismo de aprobación del entorno.

### Referencias consultadas

- [Prerrequisitos de Tauri](https://v2.tauri.app/start/prerequisites/).
- [Configuración Tauri + Vite](https://v2.tauri.app/start/frontend/vite/).
- [Inicio de Vite](https://vite.dev/guide/).
- [Workspace Cargo](https://doc.rust-lang.org/cargo/reference/workspaces.html).
- [Tauri Rust](https://docs.rs/tauri/latest/tauri/).
- [Tauri build](https://docs.rs/tauri-build/latest/tauri_build/).
- Metadatos oficiales de paquetes obtenidos mediante `npm view`.

### Versiones instaladas

| Herramienta o paquete                     | Versión          |
| ----------------------------------------- | ---------------- |
| Rust / Cargo                              | 1.90.0           |
| Node / npm                                | 22.23.4 / 10.9.9 |
| React / React DOM                         | 19.3.1           |
| TypeScript                                | 5.9.3            |
| Vite / plugin React                       | 8.3.5 / 6.2.1    |
| Tauri Rust / tauri-build                  | 2.12.5 / 2.7.4   |
| Tauri API JavaScript / CLI                | 2.12.4 / 2.12.4  |
| ESLint / @eslint/js                       | 10.14.0 / 10.0.1 |
| typescript-eslint                         | 8.72.1           |
| eslint-plugin-react-hooks / react-refresh | 7.1.1 / 0.5.7    |
| Prettier                                  | 3.9.10           |
| @types/react / @types/react-dom           | 19.3.0 / 19.3.0  |
| @types/node                               | 22.20.5          |

Las versiones directas JavaScript son exactas en `package.json`. Las dependencias
Rust externas actuales están fijadas en el workspace; las transitivas quedan
registradas en el único `Cargo.lock`. No se necesita que las versiones de los
paquetes Rust y JavaScript de Tauri tengan el mismo número de parche.

### Cambios concretos

- React montado con StrictMode, estilos oscuros y pantalla de estado inicial.
- Vite en `127.0.0.1:1420`, puerto estricto y exclusión de `target/`, `crates/` y
  `src-tauri/` de sus watchers.
- Tauri inicializado con el CLI oficial, conservando `build.rs` e iconos.
- Se sustituyeron el identificador genérico y el código inicial del generador.
  No se mantuvo el plugin de logs porque todavía no tiene un uso necesario.
- `daw-app` proporciona información de aplicación sin depender de Tauri;
  desktop la utiliza para componer el título de ventana.
- CSP explícita para producción; CSP de desarrollo permite el servidor y HMR
  locales y los estilos inyectados de Vite. Capabilities básicas solo para `main`.
- `scripts/check.mjs` orquesta comprobaciones con Node y detiene la secuencia ante
  el primer error; `doctor.ps1` inspecciona sin instalar herramientas.
- `doctor` usa `rustup run` sin `--install` para evitar que la consulta de versiones
  instale una toolchain de forma implícita.
- No se crearon carpetas vacías, controles de transporte sin implementación,
  mocks de dispositivos, tests triviales ni generadores de bindings sin DTO reales.

### Verificaciones ejecutadas

| Comando                                                                                    | Resultado                                                                                                       |
| ------------------------------------------------------------------------------------------ | --------------------------------------------------------------------------------------------------------------- |
| `rustup toolchain install 1.90.0 --profile minimal --component rustfmt --component clippy` | Toolchain fijada instalada con autorización.                                                                    |
| `npm install`                                                                              | Dependencias y lockfile generados.                                                                              |
| `npm ci`                                                                                   | Instalación reproducible completada.                                                                            |
| `npm ls --depth=0`                                                                         | Dependencias directas instaladas sin conflictos.                                                                |
| `npm run doctor`                                                                           | Rust, Cargo, Node, npm, MSVC, SDK y WebView2 detectados.                                                        |
| `npm run typecheck`                                                                        | Correcto.                                                                                                       |
| `npm run lint`                                                                             | Correcto tras importar explícitamente `node:process` en el orquestador.                                         |
| `npm run build`                                                                            | Correcto: TypeScript y bundle de producción de Vite.                                                            |
| `cargo generate-lockfile`                                                                  | Correcto: resolución de 406 paquetes compatibles con Rust 1.90.                                                 |
| `npm audit --omit=dev`                                                                     | Sin vulnerabilidades reportadas en dependencias de producción.                                                  |
| `npm run desktop:build -- --debug --no-bundle`                                             | Correcto: ejecutable en `target/debug/rust-daw-desktop.exe`, sin instalador.                                    |
| `npm run dev` y petición HTTP a `http://127.0.0.1:1420`                                    | HTTP 200 con el contenedor React y la entrada `src/main.tsx`; servidor detenido después. No es un smoke visual. |

La secuencia `npm run check` terminó correctamente, incluido formato frontend y
Rust, typecheck, lint, build frontend y
`cargo clippy --workspace --all-targets --locked -- -D warnings`.

### Incidencias y límites

- Cargo no pudo conectar a crates.io dentro del entorno restringido por un error
  TLS de Schannel (`SEC_E_NO_CREDENTIALS`). Con autorización fuera del sandbox se
  generó el lockfile y comenzó la descarga/compilación nativa.
- Se reemplazó ESLint 9 por ESLint 10 después de que npm señalara que la versión
  inicial ya no tenía soporte.
- La primera compilación desktop detectó una colisión de nombres de archivos PDB
  entre el binario y la biblioteca en Windows. Se dio a la biblioteca el nombre
  `rust_daw_desktop_lib` para mantener distintos sus artefactos.
- `npm audit` completo reporta 10 avisos moderados en herramientas de desarrollo,
  principalmente su árbol de `minimatch`. Las correcciones automáticas propuestas
  incluyen retrocesos de versiones; no se aplicó `audit fix --force` ni un override
  de versión mayor sin verificar compatibilidad. Queda registrado para revisión
  en la etapa de comprobaciones. El árbol de producción no presenta avisos.
- La presencia de WebView2 en el registro no prueba que una ventana se haya
  renderizado. El smoke visual y de audio se registra por separado en la etapa 5.
- Linux/macOS, audio real, motor, transporte, pruebas de comportamiento y contratos
  IPC todavía no están implementados ni verificados.

## Etapa 2 — Construir el motor

Estado: completada. Inicio y cierre: 2026-11-05.

### Cambios y decisiones

- Se incorporaron `daw-domain` y `daw-engine` al workspace y a sus miembros por
  defecto. El motor depende únicamente del dominio; no se añadieron dependencias
  externas. Cargo.lock registra los dos paquetes locales nuevos.
- `ProjectId` encapsula un `u64` no nulo proporcionado por el llamador, sin promesa
  de unicidad global. `Project` valida y recorta el título. El documento inicial
  usa ID 1, «Proyecto sin título» y 120 BPM.
- `Tempo` solo permite valores finitos de 20 a 300 BPM inclusive. Sus campos y
  los de configuración permanecen privados para conservar los invariantes.
- `EngineConfig` acepta frecuencia `u32` y canales `u16` positivos. El render de
  silencio admite esos valores; la compatibilidad con hardware se comprobará
  mediante el backend en la siguiente etapa.
- `AudioEngine` comienza detenido, recibe configuración y tempo explícitos y
  conserva su estado separado del documento editable. `EngineSnapshot` es Copy.
- `render_interleaved` escribe cero en todo el buffer del llamador y acepta
  tamaños variables alineados a frames, incluido el buffer vacío. No asigna,
  bloquea, hace I/O ni formatea errores. Un error de alineación o desbordamiento
  silencia el buffer sin modificar el estado; los incrementos son comprobados.
- El contador del dispositivo avanza siempre que se procesa un bloque válido;
  la posición del transporte solo avanza en reproducción. Pause conserva la
  posición, Play reanuda y Stop reinicia únicamente la posición del transporte.
- Los comandos Copy se aplican explícitamente entre renders. Cambiar el tempo
  conserva los contadores. Las colas, Accepted/Applied y el backend corresponden
  a la etapa 3; las notas futuras requerirán offsets dentro del bloque.
- `npm run check` incorpora las pruebas de dominio y motor. README documenta
  el contrato y los comandos para verificar únicamente estos paquetes.
- Se actualizó el texto de la pantalla inicial a etapa 2 y al siguiente paso:
  conectar el servicio de audio. No se añadieron controles nativos todavía.

### Verificaciones ejecutadas

- `cargo test -p daw-domain -p daw-engine --offline`: correcto; ocho pruebas
  cubren documento, BPM, silencio, configuración, particiones, transporte,
  separación documento/motor, buffers inválidos y desbordamiento de contadores.
- `cargo clippy -p daw-domain -p daw-engine --all-targets --locked --offline -- -D warnings`:
  correcto, sin warnings.
- `npm run check` (ejecutado con `npm.cmd` desde PowerShell): correcto; typecheck,
  lint, formato, build frontend, formato Rust, ocho pruebas del núcleo y Clippy de
  todo el workspace con `--locked` y warnings como errores. La primera ejecución
  detectó formato pendiente en el texto actualizado de App.tsx; se aplicó Prettier
  y la secuencia completa posterior terminó con código cero.
- `git diff --check`: correcto para los archivos ya rastreados; el formato de los
  archivos nuevos queda cubierto por Prettier y rustfmt en la secuencia anterior.

### Límites

- El motor se verifica sin dispositivo, ventana ni Tauri. No hay audio nativo ni
  controles de transporte conectados a React todavía.
- La ausencia de asignaciones se establece por inspección del camino de render;
  no se ejecutó instrumentación del allocator ni medición de plazos de tiempo real.
- No se verificó compilación en Linux/macOS ni funcionamiento con hardware.

## Etapas 3 a 6

Pendientes. Al comenzar cada etapa se agregará su sección con cambios, decisiones,
comandos ejecutados, resultados y verificaciones que no pudieron realizarse.
