# Etapa 13.1 — Modelo y contrato de clips/escenas

Fecha: 2026-11-13 (México). **Implementado localmente**, sobre
`ac37226` (árbol limpio al iniciar). Etapas 1–12 cerradas dentro de su alcance,
según cierre de 12, README y los planes vigentes. La etapa 13 sigue abierta.

## Entrega

- `daw-domain::Clip`: 16 pasos on/off y selección validada de sample.
- `ClipLauncher`: cuatro filas × 16 slots estables y 16 escenas. Sin heap, PCM,
  reloj, dependencias nuevas ni efectos sobre la sesión de audio.
- `Scene` con Keep/Stop/Launch por fila; validación completa antes de reemplazar,
  rechazo de referencias ausentes y eliminación protegida de clips referenciados.
- Resolución pura en snapshots propios; edición posterior sin alias. Conversión
  explícita de Pattern a escena 0 y proyección inversa exacta de sus cuatro filas.
- `LaunchQuantization`: paso, negra y compás; compás predeterminado.
- [Contrato](../docs/clips-scenes-contract.md) fija fronteras, fase local por
  fila, relanzamiento, precedencia de transiciones/notas, colas de voces,
  supersesión FIFO, transporte, separación seleccionado/aceptado/observado,
  futura migración v1→v2 y exportación explícita de una escena.

El modelo permanece independiente de Project hasta 13.3, cuando el formato v2
pueda conservarlo íntegro. El guardado/lectura/exportación v1 actual conserva
su comportamiento; no se añaden datos que el guardado existente perdería.

## Verificación

`cargo test -p daw-domain --locked`: **16 pruebas correctas**, incluidas siete
nuevas de clips/escenas y las nueve anteriores. Cubren todas las celdas y slots
de samples/clips/escenas, errores sin mutación, referencias ausentes/eliminación,
Keep/Stop/Launch, duplicados, snapshots independientes y round-trip de Pattern.
`cargo clippy -p daw-domain --all-targets --locked -- -D warnings`: correcto.

`npm run check`: **código 0**, 2026-11-14 01:03:52–01:07:08 UTC (13 de noviembre
en México). Incluye bindings, tipos, ESLint, 63 pruebas frontend, formato,
build web, nueve escenarios CLI, suites Rust headless/workspace, audio
instrumentado, diagnósticos, memoria de carga y Clippy. Las suites se solapan;
no se suman sus ejecuciones como pruebas distintas. No equivale a un bundle
completo de escritorio ni a una nueva medición de audio físico.

Log local: `target/stage13-1-20261113/check.log`. El
[manifiesto](evidencia-modelo-clips-13.1.json) identifica base Git, hashes de
las tres fuentes añadidas/modificadas, contrato y log. Sin cambios de lockfiles
o dependencias. Los enlaces locales de los nueve documentos revisados son válidos.

## Siguiente paso y límites

**13.2:** implementar modo opt-in del motor, reloj global exacto y fases por fila,
acciones pendientes acotadas y transiciones cuantizadas. Contrastar eventos y
PCM independientes en distintos bloques/tempos, transporte, escenas simultáneas,
supersesión, recursos ausentes, pool lleno, overflow y memoria de procesamiento.
**13.3:** servicio/aplicación, IPC/UI, Project/formato v2 y exportación de escena.
**13.4:** aceptación integrada, diagnósticos/escucha nativos y cierre/entrega/CI.

13.1 no reproduce clips ni introduce controles de UI o un archivo v2. Las reglas
de scheduling/persistencia del contrato son requisitos para 13.2/13.3, no
implementación ya disponible. No se abrió hardware, no se repitió escucha ni
se atribuye CI previa al cambio local. Sin commit/push en este incremento.
