# Etapa 9.3.c — Preparación completa del banco

Reporte fechado del coordinador. El paso posterior 9.3.d ya amplía la matriz
de fallos/cancelación y la evidencia de cierre/memoria entre hilos; ver
[reporte posterior](etapa-9-fallos-cancelacion.md). Se conserva el alcance y los
resultados originales de 9.3.c a continuación.

Fecha: 2026-11-09. Implementación local posterior a 9.3.b; se conservan las
ediciones locales anteriores. Base publicada de código 7acc1db, HEAD documental
1002efa. La CI histórica no verifica esta implementación aún no publicada.

## Entrega

[BankPreparationCoordinator](../crates/daw-app/src/sample_loading/bank.rs) prepara
un banco nuevo e inmutable desde 0–16 selecciones slot/path. Valida todos los
slots, duplicados y configuración antes de abrir archivos. Carga por slot
ascendente, con un archivo abierto por vez, usando el decoder acotado de 9.3.b.
Dos slots que nombran el mismo archivo poseen PCM independiente y se cuentan
por separado. Mono conserva sus canales de origen sin expansión.

El coordinador usa un permiso AtomicBool compartido por Arc entre clones,
sin espera ni cola. Busy tiene prioridad antes de admisión; después se observa
cancelación y se valida la solicitud. El permiso abarca preparación y resultado
PreparedBank pendiente. En error/cancelación se destruye el parcial en el hilo
llamador antes de liberar el permiso. No se conserva una selección/path en el
resultado; los errores de asset añaden slot y conservan la fuente del lector.

PreparedBank permite inspección prestada y entrega por install a un
Option<SampleBank> propiedad del llamador. Esta entrega destruye el banco anterior
antes de liberar el permiso; descartar el preparado también libera primero PCM.
Una cancelación posterior al retorno correcto no revoca el banco. install es
transferencia de propiedad de biblioteca, no instalación en el servicio: debe
usarse fuera del procesamiento y tras cerrar cualquier stream del destino.
Application, IPC y UI no reciben comandos nuevos. El escritorio conserva
silencio/senoide; sampler nativo sigue en 9.4.

## Presupuesto y evidencia de memoria

Antes de cada reserva se comprueba aritmética y payload nuevo retenido,
recurso entrante, posible copia del mismo tamaño en finalización y la reserva
conservadora completa de 64 KiB para buffers. Se cuentan los slices PCM reales
del banco activo suministrado; no se acepta un contador externo de memoria.

Límites: 128 MiB de PCM por banco, 136 MiB + 64 KiB por preparación y
264 MiB + 64 KiB incluyendo un banco activo máximo. usage devuelve máximos
**contabilizados conservadoramente**, no picos medidos por el allocator ni RSS.
Una preparación vacía no añade payload/buffers y sí cuenta el activo suministrado.
Coordinadores distintos tienen permisos/presupuestos independientes; los bancos,
copias o llamadas individuales al decoder fuera del coordinador no se controlan
globalmente. Reutilizar un coordinador por propietario de carga.

- Dos bancos reales de 16 slots estéreo, cada recurso con 1048576 frames:
  128 MiB de PCM activo + 128 MiB nuevo. Las pruebas instrumentadas debug/release
  retienen exactamente 16 asignaciones nuevas/134217728 bytes, verifican pico
  medido dentro del presupuesto de preparación y conservan el puntero del banco
  activo. Al descartar el resultado se liberan esas 16 asignaciones completas.
- Fallo por NaN en el octavo asset, después de siete recursos válidos: saldo
  final de asignaciones/bytes cero y permiso reutilizable.
- Unidad de presupuesto sin reservar memoria: techo agregado exacto aceptado,
  un byte superior rechazado sin modificar contabilidad; tamaño por recurso,
  banco activo excesivo y overflow rechazados. Un banco válido de 17 slots o
  recurso de más frames no es construible por los límites previos del motor.

La región instrumentada excluye creación del coordinador, fixtures y banco activo;
al preparar desde archivos incluye temporales Rust de apertura/conversión de
path. El PCM activo se suma explícitamente para comprobar coexistencia de payload,
no para afirmar que se midieron sus asignaciones dentro de esa región. No mide
fragmentación, pilas, caché del SO, CPAL, frontend, RSS ni tiempos de callback.

## Verificación

Seis [pruebas públicas](../crates/daw-app/tests/bank_preparation_contract.rs)
cubren vacío/resultado pendiente/entrega/cancelación tardía, validación previa a
I/O, banco disperso/mixto y paths repetidos, orden del primer error, preservación
del activo, compatibilidad y solicitudes simultáneas. Dos pruebas internas
añaden presupuestos y cancelación antes del siguiente asset o publicación,
comprobando Busy durante el loader. Se suman dos
[pruebas instrumentadas](../crates/daw-cli/tests/sample_loading_memory_contract.rs)
a las tres anteriores del decoder. Estas diez pruebas son nuevas de 9.3.c;
las suites completas se solapan.

```powershell
cargo test -p daw-app --no-default-features --locked --lib --test bank_preparation_contract --test sample_loading_contract
cargo test -p daw-app --release --no-default-features --locked --lib --test bank_preparation_contract
cargo test -p daw-cli --features rt-diagnostics --locked --test sample_loading_memory_contract
cargo test -p daw-cli --release --features rt-diagnostics --locked --test sample_loading_memory_contract
cargo clippy -p daw-app --no-default-features --all-targets --locked -- -D warnings
cargo clippy -p daw-cli --features rt-diagnostics --all-targets --locked -- -D warnings
npm run check
npm run desktop:build -- --debug --no-bundle
```

Resultados locales del 2026-11-09:

- Focales debug/release y Clippy correctos, incluidos lector previo y cinco
  pruebas de memoria. Tras el check se explicitó el uso de dos clones en la
  prueba concurrente; esta suite volvió a pasar debug/release y Clippy sin
  cambios de producción.
- npm run check correcto: 77 pruebas headless, 82 workspace, 15 de audio
  instrumentado, cinco de memoria decoder/banco, 13 frontend y nueve escenarios
  CLI. Suites solapadas, no sumarlas como pruebas únicas. Correctos también
  bindings:check, TypeScript, ESLint, Prettier, build frontend, rustfmt y Clippy.
- npm run desktop:build -- --debug --no-bundle correcto después de las
  verificaciones Cargo. Ejecutable target/debug/rust-daw-desktop.exe, sin
  instalador, smoke físico ni actualización de capturas.
- Cargo.lock, package-lock.json y src/shared/api/generated/api.ts conservan
  exactamente sus hashes previos. No hay dependencias, DTO ni workflow nuevos.
- Prettier, git diff --check, 315 destinos locales de documentación y diez
  límites numéricos del contrato correctos. actionlint valida el workflow existente.

Logs no versionados en target/stage9-3c/check.log y desktop-build.log;
hashes-before.json/hashes-after.json conservan la comparación local. Este reporte
registra el resultado duradero. scripts/check.mjs ya ejecuta la suite instrumentada,
que ahora contiene cinco pruebas. CI remota de este estado local pendiente:
no se ha hecho commit/push ni inspeccionado una ejecución nueva.

## Continuidad

Contrato, arquitectura, README, guías, roadmap y progreso distinguen banco
preparado de sampler usable en escritorio. Se conserva la evidencia fechada de
9.3.b y etapas previas. No se actualizan capturas ni se repite hardware.

Siguiente **9.3.d**: ampliar inyección determinista de fallos y cancelación en
preparación multiactivo, observar cierre/destrucción en el hilo correcto y cubrir
interleavings próximos a la entrega. El incremento actual verifica caminos
básicos, pero no cierra esa matriz completa ni las pruebas ampliadas de 9.3.e.
9.3.f cierra la preparación con evidencias; 9.4 integra transporte/headroom,
servicio/IPC/UI. La etapa 9 completa sigue abierta.
