# Etapa 9.3.e — Fixtures de carga y reproducción offline

Reporte fechado de fixtures. La auditoría posterior 9.3.f ya cierra preparación
localmente; ver [cierre posterior](cierre-preparacion-9.3.md). Se conservan los
resultados de este incremento y su alcance original.

Fecha: 2026-11-09. Incremento local posterior a 9.3.d; se conservan las ediciones
previas. Base publicada de código 7acc1db, HEAD documental 1002efa. Su CI histórica
no comprueba estas pruebas/cambios aún no publicados.

## Entrega

Cuatro pruebas nuevas en
[loaded_sampler_contract.rs](../crates/daw-app/tests/loaded_sampler_contract.rs)
verifican archivo WAV temporal → BankPreparationCoordinator → entrega de
SampleBank → kernel Sampler. Solo se añaden pruebas y documentación; no cambia
la API, producción, dependencias, DTO o UI. El escritorio conserva silencio/senoide.

- 18 combinaciones: PCM16/float32 × fmt clásico 16/18 bytes o extensible 40 bytes
  × mono→mono, mono→estéreo o estéreo→estéreo. Cada una recorre cuatro particiones
  máximas 1/127/256/4096: 72 timelines de 8205 frames. Se verifican muestras y
  metadatos del recurso, superposición/gain, fin de voces y cola silenciosa.
  El oráculo suma unidades enteras y convierte una vez a fracciones binarias;
  no depende de otro Sampler ni del resultado del decoder.
- Banco vacío y banco completo de 16 slots con formatos/headers mezclados;
  salida mono y estéreo, con mono duplicado en el segundo caso. Se disparan las
  16 voces y el disparo adicional devuelve PoolFull sin alterar el snapshot.
  La suma esperada llega a 2.8125, confirmando la política raw sin clipping o
  headroom nativo. No implementa transporte, mezclador o scheduling futuro.
- Los 65536 códigos PCM16 se cargan y reproducen para los tres headers a
  44.1 kHz coincidentes, con duplicación estéreo exacta y silencio al terminar.
  No hay resampling. Se verifican ambos extremos y todos los valores intermedios.
- Límites reales: archivo de 16 MiB con metadata saltada por seek, 4096 chunks
  y recurso de 1048576 frames estéreo con señales en extremos. Se comprueba cada
  frame, incluido el final. Un byte/frame/chunk de exceso falla por slot/límite
  tras un asset previo válido; snapshot y puntero del sampler anterior permanecen
  iguales y sus muestras aún se reproducen. El permiso vuelve a estar disponible.

Los archivos de los recursos correctos se borran antes del render. La reproducción
usa únicamente PCM del banco, incluso al destruir el coordinador de carga. Los
triggers del timeline se aplican entre llamadas, dividiendo el driver en sus
posiciones exactas; no se atribuye scheduling dentro del bloque al motor.

## Matriz de aceptación

La [matriz técnica](../docs/sample-loading-acceptance.md) reúne requisitos,
prueba ejecutable y alcance: formatos/conversión, contenedor, límites por recurso
y agregados, bancos vacíos/dispersos/completos, permisos, errores/cancelación,
memoria/hilos y exportador real. Referencia las suites de 9.3.b–d sin repetir
mediciones o presentar todas las funciones de prueba como pruebas distintas.

No equivale a enumerar todos los WAV o bits float32. Los excesos agregados se
comprueban con aritmética y límites estructurales del banco; no se puede crear
un banco válido de 17 recursos máximos. El allocator mide casos representativos,
no RSS de la app. OOM/metadata/open inyectados, reescritura concurrente de igual
longitud, I/O bloqueado y scheduler conservan las limitaciones del contrato.
Hardware y callback nativo del sampler siguen pendientes de 9.4–9.5.

## Verificación

```powershell
cargo test -p daw-app --no-default-features --locked --test loaded_sampler_contract
cargo test -p daw-app --release --no-default-features --locked --test loaded_sampler_contract
cargo clippy -p daw-app --no-default-features --all-targets --locked -- -D warnings
npm run check
npm run desktop:build -- --debug --no-bundle
```

Resultados locales del 2026-11-09:

- Cuatro pruebas focales correctas en debug/release y Clippy correcto.
- npm run check correcto: 85 pruebas headless, 90 workspace, 15 de audio
  instrumentado, ocho de memoria decoder/banco, 13 frontend y nueve escenarios
  CLI. Las suites se solapan, no sumarlas como pruebas únicas. Bindings,
  TypeScript, ESLint, formato, build frontend, rustfmt y Clippy correctos.
- npm run desktop:build -- --debug --no-bundle correcto después del check.
  Ejecutable target/debug/rust-daw-desktop.exe, sin instalador, hardware o captura
  nueva. No hay cambio de funciones/controles desktop.
- Cargo.lock, package-lock.json y src/shared/api/generated/api.ts conservan
  exactamente sus hashes previos; sin dependencias o DTO nuevos.
- 350 destinos locales Markdown/imagen existentes, diez límites del contrato
  comprobados y actionlint correcto sobre el workflow existente. Formato y
  git diff --check correctos.

Logs locales no versionados en target/stage9-3e/check.log y desktop-build.log;
hashes-before.json/hashes-after.json registran la comparación del mismo directorio.
Este reporte conserva los resultados. Las suites headless/workspace existentes
descubren las nuevas pruebas; no se añaden comandos de CI, caché, dependencias
o pasos de hardware. CI remota de este estado local pendiente de publicación e
inspección de su revisión exacta; no se afirma nueva ejecución satisfactoria.

## Continuidad

Siguiente **9.3.f: auditoría y cierre de la preparación**. Revisar criterios de
9.3.a–e contra la matriz y los reportes, registrar evidencia local/revisión y
limitaciones, y cerrar solo ese alcance. La publicación/CI del estado local se
registra por separado, sin afirmar éxito remoto antes de ejecutar/inspeccionar.
Después 9.4 integra sampler con transporte/headroom, servicio, IPC y UI; 9.5
verifica sampler nativo. La etapa 9 completa sigue abierta.

README, arquitectura, contrato, desarrollo, guías, roadmap, checklist y continuidad
se actualizan, conservando reportes y capturas fechados. No hay commit/push,
hardware nuevo, screenshots actualizados o releases en este incremento.
