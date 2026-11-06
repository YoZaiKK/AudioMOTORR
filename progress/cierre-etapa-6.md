# Cierre de etapa 6 — Documentación y CI

Reporte histórico del cierre local previo a la solicitud posterior de commit y
push. Para publicación y siguiente trabajo, consultar la
[continuidad vigente](continuidad-etapa-6.md) y el [registro](detalles.md).
Las afirmaciones de ausencia de commit/push o ejecución remota se refieren al
momento del cierre de esta etapa.

Fecha: 2026-11-06. Entorno local: Windows, Rust 1.90.0, Node 22.23.4, npm 10.9.9.
Esta etapa conserva todos los cambios locales de etapas anteriores; no implementa
funciones musicales ni crea commits, push, releases o instaladores.

## Entregables y decisiones

- [Arquitectura](../docs/architecture.md): diagramas Mermaid de dependencias y
  flujo React → IPC → aplicación → servicio → callback; tres estados y cierre.
- [Contrato de tiempo real](../docs/realtime-contract.md): consolida el contrato
  anterior e incorpora DTO, eventos y estado observado actual. El archivo
  [audio-service.md](../docs/audio-service.md) queda como enlace de compatibilidad.
- [ADR 0001](../docs/adr/0001-core-and-desktop-boundaries.md): motor independiente,
  backend manual, CPAL opcional, hilo propietario, SPSC, enteros JSON como strings
  y DTO generados; registra decisiones existentes.
- [Desarrollo y CI](../docs/development.md): Windows, mock/hardware/build,
  plataformas previstas frente a verificadas y comandos exactos.
- [Roadmap](../docs/roadmap.md): siete hitos ordenados con responsabilidades y
  aceptación. Siguiente cambio: oscilador offline dentro de daw-engine.
- [Workflow](../.github/workflows/ci.yml): push a main, PR y manual; permisos
  contents: read; cancelación de ejecuciones obsoletas; checkout v4.2.2 y setup-node
  v4.4.0 fijados por SHA completo, comprobados contra tags oficiales con ls-remote.
  Sin caché Cargo, acciones de terceros, hardware, firma o publicación.

Los reportes y las siete capturas de etapas 4/5 se conservan como evidencia
fechada. La [continuidad actual](continuidad-etapa-6.md) reemplaza únicamente las
instrucciones de siguiente trabajo, no el historial de pausas y verificaciones.

## Validación local

Etapa completada localmente. Herramientas ya instaladas, sin cambios de versión.
La secuencia Windows se ejecutó con los scripts reales del repositorio.

| Comando o comprobación                          | Resultado                                                                                                                                               |
| ----------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------- |
| npm run doctor                                  | Correcto: MSVC 14.44.35207, SDK 10.0.26100.0, WebView2 156.0.4314.58 y versiones fijadas.                                                               |
| npm ci                                          | Correcto, 218 paquetes; 13 avisos moderados. No se ejecutó audit fix.                                                                                   |
| npm run bindings:generate                       | Correcto, hash de tipos idéntico. bindings:check pasa en check.                                                                                         |
| npm run check                                   | Correcto: 12 tests frontend, 25 Rust headless, 29 Rust workspace, nueve escenarios CLI; typecheck, lint, formato, build, rustfmt y Clippy sin warnings. |
| Test de hardware ignorado                       | Permanece ignorado por defecto; no se abre hardware en la suite.                                                                                        |
| npm run desktop:build -- --debug --no-bundle    | Correcto después del check; target/debug/rust-daw-desktop.exe sin instalador.                                                                           |
| actionlint 1.7.7 .github/workflows/ci.yml       | Correcto sin diagnósticos; sintaxis y expresiones. Archivo descargado de la release oficial y SHA-256 cotejado con checksums publicados.                |
| Destinos locales de enlaces Markdown e imágenes | 99 destinos correctos; comprobación Node sobre Markdown del repositorio, excluyendo dependencias/build y URLs externas.                                 |
| Prettier final                                  | Correcto tras actualizar los documentos de cierre.                                                                                                      |
| Lockfiles y tipos antes/después                 | SHA-256 idénticos; tabla siguiente.                                                                                                                     |

Los SHA de las acciones se consultaron con git ls-remote en sus repositorios
oficiales. Se revisaron los comandos del YAML contra package.json, check.mjs y el
ejecutor CLI; Linux no usa workspace para compilar el núcleo. Las utilidades de
validación se alojan en target/stage6-tools, ignorado por Git. Git/descarga de
actionlint necesitaron autorización de red del entorno; check y build locales
posteriores pasaron sin abrir hardware. No se repitieron smoke ni capturas.

### SHA-256 antes y después

| Archivo                         | Hash idéntico                                                    |
| ------------------------------- | ---------------------------------------------------------------- |
| Cargo.lock                      | 7749328186E04BA2E36DF54C1FCFF76F7055D76C83B2301BD93E7EFB315266F1 |
| package-lock.json               | ABCCDCEAE11E9A00DCF184D6039B2E55FC8397CDF4204C883DDEB2B61BA4CFBD |
| src/shared/api/generated/api.ts | 7D0EED2708401F1764F0C7FF33F704C995055666F8C406307F518D7F51C752E4 |

Coinciden con los del [reporte de etapa 5](../howToRunThis/resultados.md). Como ya
existían cambios locales de etapas 4/5, se compara contra hashes al iniciar esta
etapa, no contra HEAD. En checkout limpio CI usa git diff.

## Auditoría del criterio final original

Fuente: sección 11 del [documento original](rust-daw-bootstrap-prompt.md), con
verificaciones de sus secciones 9/10. Las suites headless/workspace comparten
pruebas; sus cantidades no se suman como cobertura independiente.

| Criterio                                      | Implementación                                                                       | Evidencia o limitación                                                                                                               |
| --------------------------------------------- | ------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------ |
| Estructura y responsabilidades reales         | Cinco crates headless y adaptador desktop, workspace/lockfiles únicos.               | Manifiestos y diagrama de arquitectura; sin módulos musicales vacíos.                                                                |
| React abre dentro de Tauri                    | Composición Tauri, Vite y WebView2; modo navegador explícito.                        | Smoke real de etapa 5, siete PNG y metadata fechada; no se repite hardware en etapa 6.                                               |
| Llamadas atraviesan daw-app                   | Diez comandos registrados, DesktopState, Application y servicio compartido.          | ipc_contract.rs con handler real/mock runtime y ACL, tests del cliente React.                                                        |
| Backend manual determinista                   | Render explícito sin timer/hardware, motor compartido.                               | render_contract.rs, service_contract.rs, application_contract.rs y nueve escenarios CLI; particiones 1/127/256/4096.                 |
| Backend nativo abre salida y procesa silencio | CPAL opcional, selección explícita/default, conversión y callbacks.                  | Etapa 5: Realtek WASAPI, f32, 48 kHz, estéreo; otros formatos de hardware no verificados.                                            |
| Controles reflejan estado aplicado            | IDs aceptados/aplicados, snapshots secuenciados, strings/BigInt y pendientes.        | Tests React de carreras y errores, handler IPC, snapshots, recorrido real previo.                                                    |
| Cierre libera recursos                        | Publisher detenido/unido; Application se descarta; owner destruye stream y une hilo. | Tests de cierre/reinicio/handles y publisher; salida normal real de etapa 5. Drivers pueden bloquear, sin cota temporal garantizada. |
| Comprobaciones y reproducibilidad             | npm ci, bindings, check, build debug y workflow Linux/Windows.                       | Resultados locales de esta sección; CI remota pendiente.                                                                             |
| Documentación y decisiones                    | README/arquitectura/contrato/ADR/desarrollo/roadmap enlazados.                       | Inglés técnico, seguimiento y guías en español; comprobación local de enlaces.                                                       |
| Límites de tiempo real honestos               | Callback inspeccionado sin I/O, logs, locks o asignaciones dinámicas.                | Sin instrumentación allocator, medición de deadlines o continuidad/carga en release. Silencio no demuestra ausencia de dropouts.     |
| Dispositivos y plataformas pendientes         | Error/reinicio cubiertos con fallo simulado y smoke de Windows.                      | Desconexión física, Linux/macOS y formatos nativos alternativos sin verificar; no sustituidos por simulación.                        |
| Siguiente cambio concreto y alcance           | Oscilador offline en daw-engine con fase continua, luego WAV fuera del callback.     | Solo roadmap; ningún oscilador, sampler, mezcla, secuenciador o persistencia implementado aquí.                                      |

## Estado de CI y pendientes

**Workflow creado y validado localmente** es distinto de **ejecución remota
satisfactoria**. No se hizo push ni se ejecutaron estos cambios en GitHub Actions.
La primera ejecución Ubuntu/Windows hospedada deberá revisarse en sus logs.
No se afirma compilación Linux/macOS ni audio nativo verificado en esas plataformas.

Permanecen pendientes instrumentación de tiempo real, desconexión física y
formatos adicionales. Estos límites no impiden cerrar documentación/automatización
cuando sus comprobaciones locales pasen. No se ocultan los avisos npm históricos
de etapa 5 ni se ejecuta audit fix para cambiar dependencias fuera de alcance.
