# Etapa 9.3.b — Lector WAV y decodificación acotada

Reporte fechado del incremento individual. El paso posterior 9.3.c ya implementa
coordinación de banco, permiso y presupuesto agregado; ver
[reporte posterior](etapa-9-preparacion-banco.md). Los resultados de abajo
corresponden a 9.3.b y conservan su alcance original.

Fecha: 2026-11-09. Se conservan las ediciones locales previas de auditoría y
contrato. Implementación local sobre código 7acc1db/HEAD documental 1002efa;
no se atribuye la CI histórica a estos cambios todavía no publicados.

## Implementación

- Nuevo módulo [daw-app::sample_loading](../crates/daw-app/src/sample_loading.rs),
  sin dependencias nuevas. decode_wav consume Read + Seek y devuelve un
  SampleBuffer inmutable; load_wav_file abre un archivo regular y usa el mismo
  camino. Ninguno crea Application, servicio, dispositivo o comandos IPC.
- Preflight de toda la extensión RIFF antes de reservar PCM: límite de 16 MiB,
  hasta 4096 chunks, un fmt antes de un data, padding/offsets y tamaños comprobados.
  Metadata desconocida se salta por seek; se inspeccionan también los trailers.
  plst/LIST-wavl se rechazan; LIST de metadata no se procesa recursivamente.
- PCM16/float32 clásico/extensible, mono/estéreo y rate objetivo exacto. Valida
  precisión, GUID/máscaras, byte rate, block align, frame count y compatibilidad.
  PCM16 /32768 y bits float32 conservados, incluido cero con signo. Rechazo de
  NaN/inf/fuera de rango con índice de muestra interleaved.
- Reserva única con try_reserve_exact y capacidad exacta, sin crecimiento o Vec
  del archivo completo. Scratch de decode 2048 bytes más arrays de cabecera,
  acotados por assert de compilación al presupuesto de 64 KiB.
- Cancelación AtomicBool antes/después de I/O, preflight, reserva y bloques de
  hasta 256 frames; comprobación final antes de entregar el recurso. Cierre del
  lector y liberación del PCM parcial en fallo/cancelación. No se promete aborto
  de I/O bloqueado ni detectar reescrituras concurrentes que mantengan longitud.
- LoadError tipado para I/O, metadata, límites, compatibilidad, PCM/cancelación y
  reserva fallida; conserva fuente io::Error. Son errores de biblioteca, no DTO.

El resultado de este paso es **un recurso**. No hay aún preparación atómica de
un banco completo, permiso Busy, control de resultados pendientes o presupuesto
global de banco activo + preparación. Esas responsabilidades siguen en 9.3.c–e.
La app de escritorio mantiene silencio/senoide; sampler nativo/UI queda en 9.4.

## Pruebas y mediciones

- Ocho [pruebas de lector](../crates/daw-app/tests/sample_loading_contract.rs),
  con fixtures propios de contenedor y datos: formatos/precisión/máscaras,
  preservación de muestras/canales, metadata impar/trailing, duplicados/orden,
  límites exactos y uno superior, I/O/lecturas parciales/Interrupted, cancelación,
  cambio de longitud detectable y cierre del lector. Debug y release correctos.
- Una prueba interna inyecta error en la reserva y observa cierre del lector;
  comprueba la ruta AllocationFailed sin provocar OOM real del sistema.
- Una prueba nueva en [export_contract](../crates/daw-cli/tests/export_contract.rs)
  ejecuta el exportador real mono/estéreo (4801 frames, bloque 127, 443.25 Hz),
  carga los WAV extensibles resultantes y compara bits exactos. El recurso
  conserva datos después de borrar el archivo; se rechaza abrir un directorio.
- Tres [pruebas de memoria](../crates/daw-cli/tests/sample_loading_memory_contract.rs)
  reutilizan allocation-counter opcional del CLI. En debug y release:
  entradas float32 de un frame y máximo mono/estéreo usan una reserva exacta,
  sin crecimiento/copias de archivo; cabeceras sobredimensionadas no reservan PCM;
  PCM inválido y cancelación liberan el resultado parcial sin saldo de memoria.

Las medidas usan Cursor prestado: los fixtures codificados se crean fuera de la
región medida. En éxito, count_total/count_current = 1 y bytes_total/bytes_max
son exactamente frames × canales × 4 (hasta 8388608). Destrucción medida aparte
reporta una liberación del mismo tamaño. En fallo tras reserva, saldo cero;
en rechazo por frames excesivos, cero asignaciones. No es una medición de RSS,
del sistema de archivos, de bancos coexistentes o de callback nativo.

```powershell
cargo test -p daw-app --no-default-features --locked --lib --test sample_loading_contract
cargo test -p daw-app --release --no-default-features --locked --test sample_loading_contract
cargo test -p daw-cli --no-default-features --locked --test export_contract
cargo test -p daw-cli --features rt-diagnostics --locked --test sample_loading_memory_contract
cargo test -p daw-cli --release --features rt-diagnostics --locked --test sample_loading_memory_contract
cargo clippy -p daw-cli --features rt-diagnostics --all-targets --locked -- -D warnings
npm run check
npm run desktop:build -- --debug --no-bundle
```

Resultados locales del 2026-11-09:

- npm run check correcto: 69 pruebas headless, 74 workspace, 15 de audio
  instrumentado, tres de memoria del decoder, 13 frontend y nueve escenarios
  CLI. Las suites se solapan; no sumar como pruebas únicas. Pasa también
  bindings:check, TypeScript, ESLint, Prettier, build frontend, rustfmt y Clippy.
- Después del check se reforzaron únicamente fixtures de fallo/cancelación:
  NaN en la muestra 300 y cancelación en la segunda lectura de datos, después
  de decodificar 256 frames. Las pruebas focales de lector/memoria volvieron a
  pasar en debug/release y Clippy; no cambió el código de producción.
- Después de las verificaciones Cargo, npm run desktop:build -- --debug
  --no-bundle correcto, sin instalador. Ejecutable local:
  target/debug/rust-daw-desktop.exe. No se abre audio físico ni se actualizan
  capturas: UI y camino de reproducción nativo no cambian.
- Cargo.lock, package-lock.json y src/shared/api/generated/api.ts conservan
  exactamente sus hashes anteriores. No hay dependencias ni DTO nuevos.
- Revisión documental: 303 destinos locales Markdown/imagen existentes,
  diez límites/arithmetic del contrato comprobados, Prettier y git diff --check
  correctos. actionlint valida el workflow existente, sin modificarlo.

Logs locales: target/stage9-3b/check.log y desktop-build.log; hashes comparados
en hashes-before.json/hashes-after.json del mismo directorio. Estos archivos
de target no son evidencia versionada; este reporte conserva los resultados.
scripts/check.mjs añade ejecución explícita de las tres pruebas de memoria del
CLI, sin hardware. Las suites headless/workspace existentes descubren las
nuevas pruebas de lector/exportación. El workflow conserva sus jobs originales;
no se ha ejecutado CI remota para estos cambios locales.

## Documentación y siguiente paso

Contrato, arquitectura, README, desarrollo, guías, roadmap, checklist y continuidad
se actualizan para separar recurso individual de banco completo. Se conservan
reportes/capturas anteriores y los cambios locales no publicados. No se repite
hardware ni se afirma CI nueva a partir de la de 7acc1db.

Siguiente: **9.3.c, coordinación de banco completo**. Validar 0–16 slots únicos
antes de I/O, ordenar la carga, admitir una sola preparación/resultado pendiente
y aplicar presupuestos agregados. Después 9.3.d–e verifica entrega/cancelación
entre assets y conservación del banco vivo; 9.3.f cierra el incremento antes de
9.4 nativo/IPC/UI.
