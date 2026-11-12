# Etapa 9.5.b — Mediciones nativas del sampler

Actualización posterior del 2026-11-11: [etapa 9 cerrada](cierre-etapa-9.md) con
auditoría/CI inspeccionada y audibilidad confirmada tras repetir con audífonos.
El informe siguiente conserva su estado original, anterior a esa confirmación.

Fecha: 2026-11-10. **Seis casos nativos correctos en hardware Realtek**, con
entrada mono/estéreo y ventanas de 0/1/16 voces. Escucha humana solicitada pero
sin confirmación registrada al redactar este informe. La medición de procesamiento
no demuestra sonido acústico, continuidad de salida o ausencia de dropouts.
La etapa 9 sigue abierta; falta esa confirmación y la consolidación/CI de 9.5.c.

## Entorno y alcance

- Base Git b0ba5895d4b5d4b764727f7d1af1943c522ebc0c, árbol limpio al iniciar.
  La matriz usa un cambio local de trazabilidad del runner; no hay commit/push.
- Windows NT 10.0.26200.0, CPU Intel Core i7-1185G7 (identificación nominal 3.00 GHz). Rust/Cargo 1.90.0,
  Node 22.23.4, npm 10.9.9. CPAL/WASAPI, salida Speakers (2- Realtek(R) Audio).
- Device ID explícito: wasapi:{0.0.0.00000000}.{7ca1b2e0-ec85-4d31-bc07-f036d5ce34a3}.
  Formato seleccionado y observado: 48000 Hz, dos canales, f32. La salida anuncia
  dos canales; no se prueba salida mono ni otros formatos físicos.
- Ventanas de voces de un segundo por progreso de frames, más gain, Pause,
  resume, Stop/retrigger, finalización natural, shutdown y reapertura. Las métricas
  son acumuladas de toda la sesión; no son percentiles aislados de cada ventana.
  El caso cero voces incluye después un trigger de comprobación del ciclo de vida.
- Son ejecuciones cortas sobre el host de desarrollo compartido. Sin loopback,
  desconexión física, estrés prolongado o mediciones nativas Linux/macOS.

Artefactos locales no versionados: target/stage9-5b-20261110-134708. El
[manifiesto nativo](evidencia-nativa-9.5b.json) conserva datos, IDs, histogramas y
hashes de entradas/logs/binarios. Los manifiestos anteriores permanecen fechados.

## Entradas y reproducción

Ambos WAV son float32, 480000 frames/10 s a 48 kHz:

| Archivo             | Canales | Señal / amplitud pico                         | SHA256 antes y después                                           |
| ------------------- | ------- | --------------------------------------------- | ---------------------------------------------------------------- |
| mono-997.wav        | 1       | Senoide 997 Hz / 0.1, export-wav              | f40ec0d716b0c09c9d1b6c1c3bfb9e70262ec0c44c5cd04c78eeb3af88cc76c8 |
| stereo-distinct.wav | 2       | Izquierda 997 Hz / 0.1; derecha 440 Hz / 0.05 | accf828c3b709bcffb7e57a72313e1a29fa6eda9a18a64b2b01875136e19ed4b |

Hashes idénticos tras todas las sesiones. Payload PCM: 1920000 bytes mono,
3840000 estéreo; cada banco contiene un recurso, compartido por sus voces.
No es un estrés nativo de los 16 assets ni del presupuesto máximo de banco.

Se añade [generador estéreo independiente](../scripts/sampler-stereo-fixture.mjs)
para reproducir el archivo en una ruta nueva. Se verificó igualdad exacta del hash
y rechazo de sobrescritura sin cambiar bytes. No utiliza DSP del motor ni abre
audio. Los archivos parciales tras un error de escritura requieren revisión manual.

Desde una carpeta nueva, con un device ID enumerado:

```powershell
New-Item -ItemType Directory -Path target/sampler-native-repro
cargo run -p daw-cli --release --features rt-diagnostics --locked -- export-wav --output target/sampler-native-repro/mono-997.wav --frames 480000 --sample-rate 48000 --channels 1 --frequency 997 --amplitude 0.1
node scripts/sampler-stereo-fixture.mjs target/sampler-native-repro/stereo-distinct.wav
cargo run -p daw-cli --release --features rt-diagnostics --locked -- devices
cargo run -p daw-cli --release --features rt-diagnostics --locked -- sampler-native-check --input target/sampler-native-repro/stereo-distinct.wav --device "<ID enumerado>" --sample-rate 48000 --channels 2 --voices 16 --seconds 1
```

Repetir cada entrada con voices 0/1/16 y guardar stdout/stderr, código de salida,
comando, tiempos y hashes. El generador se verificó después de medir y reproduce
exactamente la entrada inicial; sus operaciones quedan fuera de la región medida.

## Resultados completos

Todas las celdas corresponden a la primera sesión de cada caso. p50/p99 son
**cotas superiores de bins**, no percentiles exactos. Un bin puede superar el máximo
observado. Callbacks de 480 a 1056 frames en las seis sesiones; las reaperturas
también anuncian progreso. El máximo relativo usa la duración del callback real.

| Entrada | Voces | Callbacks medidos | Frames medidos | p50 superior ns | p99 superior ns | Máximo ns | Máximo / presupuesto |
| ------- | ----- | ----------------- | -------------- | --------------- | --------------- | --------- | -------------------- |
| Mono    | 0     | 1109              | 532896         | 16383           | 65535           | 432100    | 4.321 %              |
| Mono    | 1     | 1111              | 533856         | 16383           | 65535           | 76100     | 0.761 %              |
| Mono    | 16    | 1128              | 542016         | 16383           | 131071          | 127700    | 1.277 %              |
| Estéreo | 0     | 1108              | 532416         | 16383           | 65535           | 97100     | 0.971 %              |
| Estéreo | 1     | 1110              | 533376         | 16383           | 65535           | 90900     | 0.909 %              |
| Estéreo | 16    | 1127              | 541536         | 16383           | 131071          | 143400    | 1.434 %              |

Se verificó en las doce sesiones (inicial y reapertura de cada caso): cero
asignaciones/bytes asignados, liberaciones/bytes liberados, muestras no finitas
y excesos del presupuesto nominal de procesamiento. Esto mide el callback
instrumentado, no todo el proceso ni jitter/driver/salida física. Los probes de
ambos canales son finitos y acotados. Agregados de tiempo/memoria publican un
callback después; los probes incluyen el callback actual y puede faltar el último
agregado al cerrar. Los contadores no representan una captura acústica.

Las ventanas confirman al menos 48000 frames con exactamente 0/1/16 voces
activas. Pausa mantiene transporte/voces mientras avanza el reloj de dispositivo;
Stop conserva PCM para retrigger y limpia voces. Cada ejecución acaba Uninitialized
después de reconstruir banco/reabrir el mismo dispositivo con contadores nuevos.
No se mide RSS ni se prueba ausencia general de fugas a largo plazo.

## IDs y trazabilidad

El piloto mono/0 inicial, con runner de b0ba589 limpio, también pasó y se conserva
en mono-v0.log con sus hashes. Mostró que los IDs solo se validaban internamente.
Se añade impresión de controles/resultados fuera de callback y métricas de la
reapertura. Se vuelve a verificar el runner y se registra toda la matriz con ese
cambio; el piloto no se sustituye ni se mezcla con la tabla.

Cada proceso tiene su propio espacio de IDs. Ejemplos de la matriz:

- ID 1: NotPlaying, procesado antes de Play, con cero voces.
- En casos de 16 voces, IDs 3–18: Started/Ok(0–15); ID 19: PoolFull conservando
  las 16 voces. Trigger de banco retenido ID 25 y de reapertura ID 28: Ok(0).
- Los IDs de aceptación y del resultado exacto coinciden; también se guardan
  applied_id, voces, callbacks y controles/ganancia. No se atribuye un resultado
  posterior a otro ID. El manifiesto contiene todos los resultados, no solo ejemplos.

## Verificación de código y pendientes

Check completo y build Windows ejecutados antes de abrir audio y de nuevo tras
añadir trazas: correctos. Suite: 108 headless, 114 workspace, 29 audio instrumentadas,
nueve diagnósticos CLI, ocho memoria decoder/banco, 24 frontend y nueve escenarios
CLI; suites solapadas. Release del runner: siete unitarias y dos pruebas de proceso,
sin hardware en esos tests. Clippy y rustfmt correctos. El generador nuevo se
verifica con lint/formato y reproducción/hash/sobrescritura. El manifiesto registra
120 archivos de código/configuración; cambian solo runner/generador respecto del
inicio de la medición. Revisión final correcta: formato/lint/rustfmt, 593 destinos
locales de enlaces/imágenes Markdown, diez límites del contrato, actionlint y
git diff --check. Coinciden los 120 hashes y se conservan locks/bindings/workflow.

La escucha humana está pendiente de respuesta; no se marca como confirmada por
el éxito de WASAPI/probes. **Siguiente:** incorporar esa observación y realizar la
consolidación 9.5.c con límites y CI del SHA que se publique. No hay commit/push,
firma, instalador, screenshot nuevo o CI remota nueva. Etapa 9 sin cierre final.
