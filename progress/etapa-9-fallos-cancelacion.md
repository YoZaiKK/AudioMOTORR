# Etapa 9.3.d — Fallos, cancelación y cierre

Reporte fechado de fallos/cancelación. El paso posterior 9.3.e ya añade fixtures
de extremo a extremo y consolida la matriz de aceptación; ver
[reporte posterior](etapa-9-fixtures-integracion.md). Los resultados de abajo
mantienen el alcance original de 9.3.d.

Fecha: 2026-11-09. Incremento local posterior a 9.3.c, conservando los cambios
anteriores. Código publicado de referencia 7acc1db y HEAD documental 1002efa;
no se atribuye su CI histórica a este estado sin publicar.

## Implementación y alcance

El coordinador conserva su API pública y comportamiento. Se amplía la costura
privada de adaptadores con el constructor del banco: producción sigue usando
SampleBank::new y comprueba cancelación después de construirlo, antes de devolver
PreparedBank. Las pruebas pueden detenerse exactamente en esa frontera con
canales, sin sleeps, tiempos probabilísticos o inyección pública. No se modifica
el motor, servicio, IPC, frontend, dependencias o workflow.

Se añaden cuatro pruebas internas en
[failure_tests.rs](../crates/daw-app/src/sample_loading/bank/failure_tests.rs):

- Matriz de 17 casos en el segundo asset, después de cargar el primero:
  open/metadata, lectura de header/datos, seek inicial/datos, truncación, cambio
  de longitud, reserva fallida, NaN y cancelación en header, pre-reserva,
  reserva, segundo bloque de datos, error de lectura concurrente, comprobación
  final y cierre del lector. No se intenta el tercer asset.
- Retorno de banco sincronizado: cancelación antes del último checkpoint
  rechaza el banco completo; después del retorno conserva el resultado pendiente
  y permite entregarlo. Busy se mantiene hasta entrega/descarte.
- Read suspendido en el segundo bloque: no termina al activar cancelación;
  se reanuda explícitamente y el checkpoint posterior observa Cancelled,
  tanto si Read devuelve datos como si devuelve un error.
- Un error ya devuelto por el adaptador se conserva cuando la cancelación
  todavía no fue observada en otro checkpoint. No se promete orden total entre
  errores y cancelaciones concurrentes.

La matriz registra intento/apertura/cierre por slot y ThreadId. Comprueba un
solo lector abierto, cierre en el trabajador que preparó, permiso Busy durante
el cierre, puntero/muestras del banco activo intactos y reutilización del
permiso tras cada fallo. Las fuentes io::Error conservan operación y kind en
la cadena de errores. Open/metadata/reserva fallidos son resultados simulados
del adaptador; no se provoca fallo físico de disco o OOM real. Los demás
escenarios recorren el decoder real con un lector Read + Seek instrumentado.

## Memoria y transferencia entre hilos

Tres pruebas nuevas en
[sample_loading_memory_contract.rs](../crates/daw-cli/tests/sample_loading_memory_contract.rs)
reutilizan el contador opcional existente; la suite pasa de cinco a ocho pruebas:

- Trabajador con tres assets válidos de 4096 frames estéreo antes de un archivo
  inexistente, header incorrecto o NaN. Cada caso tiene saldo cero de nuevas
  asignaciones y bytes en ese trabajador; banco activo conservado y permiso
  reutilizable. Las aperturas aquí sí usan archivos temporales reales.
- Cancelación del decoder en un trabajador después de 256 frames: una reserva
  de 4096 bytes y saldo final cero de asignaciones/bytes en ese hilo.
- Resultado creado por trabajador con cuatro buffers de 32768 bytes: retiene
  cuatro asignaciones/131072 bytes al devolverlo. Descartarlo en el receptor
  libera allí esas cuatro asignaciones. Entregarlo al holder libera allí el
  buffer anterior de ocho bytes; la destrucción posterior del banco nuevo
  libera los 131072 bytes. Cancelación después del retorno no revoca el valor.

El contador es local al hilo, por lo que los saldos observan las liberaciones
en el hilo medido. Creación del trabajador, fixtures, coordinador y banco previo
se preparan fuera de la región de medida. No se instrumenta cada caso de la
matriz de fallos con el allocator: las trazas de lectores y estas mediciones
representativas aportan evidencias complementarias. No se mide RSS, callback
nativo, latencia de disco ni todas las intercalaciones posibles del scheduler.
Los límites de cancelación cooperativa/I/O bloqueado del contrato se conservan.

## Verificación

Siete funciones de prueba nuevas: cuatro internas con matriz/interleavings y
tres instrumentadas. Se conserva la cobertura previa del lector y coordinador.

```powershell
cargo test -p daw-app --no-default-features --locked --lib --test bank_preparation_contract
cargo test -p daw-app --release --no-default-features --locked --lib --test bank_preparation_contract
cargo test -p daw-cli --features rt-diagnostics --locked --test sample_loading_memory_contract
cargo test -p daw-cli --release --features rt-diagnostics --locked --test sample_loading_memory_contract
cargo clippy -p daw-app --no-default-features --all-targets --locked -- -D warnings
cargo clippy -p daw-cli --features rt-diagnostics --all-targets --locked -- -D warnings
npm run check
npm run desktop:build -- --debug --no-bundle
```

Resultados locales del 2026-11-09:

- Focales debug/release y Clippy correctos: siete pruebas unitarias de daw-app
  (cuatro nuevas), seis públicas de banco y ocho instrumentadas de memoria
  (tres nuevas), además de las suites descubiertas por el check.
- npm run check correcto: 81 pruebas headless, 86 workspace, 15 de audio
  instrumentado, ocho de memoria decoder/banco, 13 frontend y nueve escenarios
  CLI. Suites solapadas, no sumarlas como pruebas únicas. Bindings, TypeScript,
  ESLint, formato, build frontend, rustfmt y Clippy correctos.
- npm run desktop:build -- --debug --no-bundle correcto después del check.
  Ejecutable local target/debug/rust-daw-desktop.exe, sin instalador, nueva
  captura o smoke físico; las fuentes y controles desktop conservan su alcance.
- Cargo.lock, package-lock.json y src/shared/api/generated/api.ts mantienen
  exactamente sus hashes anteriores. No hay dependencias, DTO ni workflow nuevos.
- Revisión documental: 324 destinos locales Markdown/imagen existentes, diez
  límites numéricos del contrato correctos, formato y git diff --check correctos.
  actionlint valida el workflow existente.

Logs locales no versionados: target/stage9-3d/check.log y desktop-build.log;
hashes-before.json/hashes-after.json del mismo directorio registran la comparación.
Este reporte conserva los resultados. La CI remota de este estado sigue pendiente
de publicar e inspeccionar su revisión exacta; no se deduce de estos checks locales.

## Siguiente paso

**9.3.e: fixtures de extremo a extremo y matriz de aceptación.** Completar el
camino archivo WAV → banco preparado → kernel Sampler, con oráculos de muestras
mono/estéreo, PCM16/float32 clásico/extensible, orden y particiones de render;
consolidar los casos de formatos/límites ya cubiertos y los que falten. No añade
funciones musicales o comandos UI. 9.3.f registra cierre de la preparación;
9.4 integra transporte/headroom, servicio e IPC/UI y 9.5 verifica sampler nativo.

README, contrato, desarrollo, guías, roadmap, checklist y continuidad se actualizan.
Los reportes/capturas previos conservan su evidencia fechada. Sin nuevo smoke
físico, commit/push o ejecución remota de CI para estos cambios locales.
