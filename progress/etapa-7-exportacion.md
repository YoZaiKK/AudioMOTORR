# Etapa 7 — Exportación offline WAV (7.3)

Reporte histórico del incremento local antes de publicarlo. La etapa completa
se publicó en c6cf627 y se cerró tras CI correcta en Linux/Windows; ver
[cierre vigente](cierre-etapa-7.md). Las notas de CI pendiente describen el
momento de esta verificación y se conservan como evidencia fechada.

Fecha: 2026-11-07. Implementación local de 7.3; pruebas/documentación de 7.4
integradas en los comandos actuales. Cierre 7.5 y CI remota del nuevo estado
pendientes. Se conserva el [reporte del motor](etapa-7-oscilador.md) como evidencia
del incremento anterior, incluida CI correcta del SHA 1ef9bd0, no de estos cambios.

## Implementación y decisiones

- daw-app::offline::OfflineRender valida y posee un motor independiente, duración
  exacta y un buffer de como máximo 4096 frames por 2 canales (32 KiB de muestras).
  El callback de destino recibe muestras después del render, se detiene al primer
  error y no se invoca si la duración es cero. No depende de Application ni de
  inicializar su servicio; no modifica documento ni reproducción viva.
- CLI: export-wav --output PATH --frames N. Acepta --block-size, --sample-rate,
  --channels, --frequency y --amplitude. Defaults 256/48000/2/440/0.1; sin segundos,
  override de archivos, playback automático o creación de carpetas.
- hound =3.5.1, solo en daw-cli; consultados documentación y código oficiales.
  Cargo.lock cambia deliberadamente con una dependencia y su checksum; ningún
  cambio en package-lock.json o api.ts. La descarga necesitó autorización de red
  después de que Schannel fallase dentro del sandbox.
- WAV float32 con header extensible de 68 bytes. Preflight del tamaño RIFF y
  byte rate antes de crear archivo; máximo floor((u32::MAX - 60)/(canales * 4))
  frames y sample_rate * canales * 4 representable en u32.
- OpenOptions::create_new protege archivos existentes. BufWriter y finalize
  explícito propagan errores de escritura, seek y flush. Un fallo después de crear
  el destino puede dejar un archivo parcial, que se identifica en el error;
  no se elimina ni reintenta automáticamente. Los handles se cierran.
- No se cambia motor/servicio de tiempo real, DTO, UI ni smoke de silencio.

## Cobertura nueva

- Cuatro tests de aplicación: muestras exactas frente al motor por bloques,
  reutilización del mismo buffer y bloque final parcial, validación/duración cero,
  parada ante fallo del destino e independencia de documento/servicio manual.
- Cuatro tests de integración CLI: WAV reabierto mono/estéreo y frames 0/1/4801
  con bloques 1/127/256/4096; defaults/configuración personalizada/amplitud cero;
  destino existente, directorio y ruta inválida; parámetros y límites inválidos
  rechazados antes de crear archivo. Se comprueban metadatos, muestras y bytes RIFF.
- Dos tests unitarios CLI: errores inyectados al crear header, escribir muestras,
  finalizar con seek/flush y cierre del escritor; límites RIFF/byte rate sin
  renderizar archivos gigantes.
- Tests CLI usan carpetas temporales exclusivas y borran solo sus propios datos.
  Los nueve escenarios Node de silencio quedan intactos. La selección existente
  de paquetes en npm run check y ambos jobs CI ya incluye la cobertura WAV;
  no requiere acciones nuevas ni dependencias nativas adicionales.

## Verificación

- Tests/Clippy de daw-app y daw-cli sin default features, con --locked: correctos.
- npm run check completo: correcto. Doce tests frontend, 41 Rust headless y
  45 Rust workspace, nueve escenarios CLI, bindings, TypeScript, lint, formato,
  build Vite, rustfmt y Clippy con -D warnings. Las suites Rust se solapan;
  no sumar sus cantidades como pruebas independientes. Hardware ignorado por defecto.
- npm run desktop:build -- --debug --no-bundle: correcto después del check.
  No se abre hardware ni se actualizan capturas.
- Export real de 48001 frames a target/stage7-preview-0172a35ca74841e0955ae202eb0bb98b.wav:
  correcto, 384076 bytes. Lectura independiente en Node comprueba RIFF, GUID float,
  48000 Hz, dos canales, 32 bits y todas las muestras frente a senoide de referencia
  (tolerancia 1e-6); pico 0.10000000149011612. Archivo ignorado bajo target, no versionado.
- Prettier, enlaces locales Markdown/imágenes y git diff --check: correctos tras
  actualizar el registro. No hay ejecución remota nueva de estos cambios locales.

### Hashes después de fijar la dependencia y tras verificar

Cargo.lock cambia deliberadamente frente al cierre de 7.2 por hound 3.5.1; el
nuevo hash permanece intacto durante check/build. Los otros dos no cambian.

| Archivo                         | SHA-256 final                                                    |
| ------------------------------- | ---------------------------------------------------------------- |
| Cargo.lock                      | 1BD91E697ABF2880CC0AF6CF40A98753545EE9EA4D23732F94EF7650B5E3CB12 |
| package-lock.json               | ABCCDCEAE11E9A00DCF184D6039B2E55FC8397CDF4204C883DDEB2B61BA4CFBD |
| src/shared/api/generated/api.ts | 7D0EED2708401F1764F0C7FF33F704C995055666F8C406307F518D7F51C752E4 |

## Continuidad

Finalizar reporte/validación local, revisar el conjunto de etapa 7 y publicar
cuando corresponda, comprobando CI del SHA nuevo para cerrar 7.5. No atribuir el
verde de la base a cambios locales sin publicar. Después, etapa 8: mismo
oscilador por CPAL, rampas y mediciones en release con hardware identificado.

La exportación demuestra señal offline y archivos consistentes; no plazos de
callback, continuidad audible nativa ni recuperación tras desconexión física.
