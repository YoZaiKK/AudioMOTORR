# CLI y archivos de prueba

Las [pruebas nativas 9.5.b](../progress/etapa-9-medicion-nativa.md) registran seis
casos Realtek/WASAPI correctos con IDs/resultados y métricas de reapertura. Para
reproducir la entrada estéreo en una ruta nueva:

```powershell
node scripts/sampler-stereo-fixture.mjs target/stereo-native.wav
```

Genera 10 s a 48 kHz, float32 estéreo, 997 Hz a 0.1 a la izquierda y 440 Hz a
0.05 a la derecha; no sobrescribe ni abre audio. Se comprueba el hash al generar.

## Diagnósticos sampler (9.5.a, 2026-11-10)

Sin dispositivo ni CPAL:

```powershell
cargo run -p daw-cli --release --features offline-diagnostics --locked -- sampler-bench --iterations 2000
cargo run -p daw-cli --release --features offline-diagnostics --locked -- sampler-bench --channels 1 --iterations 1000
```

Por defecto prueba bloques 1/127/256/4096 y voces 0/1/16. Puedes elegir
--block-size, --voices, --channels y --iterations. Reporta oráculo/actividad,
asignaciones/liberaciones, p50/p99/max y overruns; no demuestra deadlines nativos.
El PCM sintético estéreo tiene canales distintos y todos los buffers se preparan
antes de medir. Se comprueba liberación del banco después, fuera del procesamiento.

sampler-native-check ya está implementado, pero abre audio físico y corresponde
a 9.5.b. Requiere --input WAV y --device ID explícitos, --sample-rate/--channels
compatibles, --voices 0/1/16 y --seconds 1–5. El recurso debe durar seconds+4 a
15 s; se mantienen los límites WAV existentes. Para ejecutarlo y registrar hashes,
seguir el [contrato de diagnósticos](../docs/sampler-diagnostics.md) y
[protocolo](../docs/sampler-native-verification.md). En esta entrega solo se compila
y verifica su lifecycle con backend manual. rt-diagnostics habilita audio nativo;
offline-diagnostics lo rechaza antes de leer el archivo.

## Diagnóstico de señal nativa y rampas (etapa 8)

```powershell
cargo run -p daw-cli --release --features rt-diagnostics --locked -- engine-bench --block-size 127 --iterations 20000
cargo run -p daw-cli --release --features rt-diagnostics --locked -- tone-check --seconds 2
```

engine-bench no abre hardware: mide bloques preasignados, asignaciones y tiempos
del motor (bloque 1–4096; iteraciones 1–1000000). tone-check sí reproduce una senoide,
cambia volumen, pausa/reanuda/para y reabre el mismo dispositivo. Admite duración
1–60 segundos, --frequency y --device. Ambos rechazan builds debug y requieren
la feature explícita; el desktop normal no lleva instrumentación.

tone-check exige progreso, rampas asentadas, señal finita y no nula, cero
asignaciones/excesos del presupuesto nominal y saltos de muestra acotados.
No mide continuidad acústica, driver, jitter de llegada ni CPU global. Ver
[contrato técnico](../docs/native-audition.md) y [mediciones reales](../progress/etapa-8.md).
No forma parte de CI ni sustituye la exportación WAV determinista.

## Procesamiento offline

```powershell
cargo run -p daw-cli --locked -- smoke --frames 48001 --block-size 127
cargo run -p daw-cli --locked -- smoke --seconds 1 --block-size 4096
```

Esperar `smoke OK`, con frames exactos, 48000 Hz, dos canales y `silence=true`.
El código de salida es cero. El último bloque puede ser parcial: nunca se redondea
el total al tamaño de bloque. No se abre ningún dispositivo.

Elegir frames o segundos enteros, no ambos. El bloque admite 1–4096 frames y
`--frames 0` comprueba una ejecución vacía sin aplicar el Play encolado.

## Ejecutar los casos incluidos

```powershell
node howToRunThis/run-cli-cases.mjs
```

Compila la CLI sin native-audio una vez y ejecuta nueve casos de
[cli-cases.json](test-files/cli-cases.json). Verifica salida y exit code, incluidos
errores esperados. `PASS ... exit=1` significa que un argumento inválido se rechazó
correctamente. Un desacuerdo termina el ejecutor con error.

No interpreta comandos de shell del JSON: solo ejecuta el subcomando smoke del
binario construido. Los datos son fixtures del ejecutor, no archivos musicales.
La [salida esperada](test-files/expected-output.txt) sirve de ejemplo y no se usa
como un reporte inventado de pruebas.

## Exportar una senoide a WAV sin hardware

```powershell
cargo run -p daw-cli --no-default-features --locked -- export-wav --output target/tono.wav --frames 48001 --block-size 127
cargo run -p daw-cli --no-default-features --locked -- export-wav --output target/tono-mono.wav --frames 44100 --sample-rate 44100 --channels 1 --frequency 997.25 --amplitude 0.25
```

Usar nombres nuevos: el comando no sobrescribe archivos ni crea carpetas. La
carpeta target existe después de compilar; si se elige otra ruta, preparar su
directorio padre. No hay reproducción automática ni se abre un dispositivo.

Se requieren --output y --frames. Defaults: 48000 Hz, dos canales, senoide de
440 Hz, amplitud pico 0.1 y bloques de 256 frames. Opciones: --sample-rate,
--channels (1 o 2), --frequency, --amplitude y --block-size (1–4096). No acepta
--seconds. --frames 0 crea un WAV vacío válido; el bloque final puede ser parcial.

Frecuencia finita, mayor que cero y menor que la mitad del sample rate; amplitud
finita entre cero y uno. WAV es float32, no PCM entero. El número de frames y
byte rate deben caber en sus campos RIFF; requests fuera de límite fallan antes
de crear archivo. Ver [contrato y límites exactos](../docs/offline-oscillator.md).

Esperar export-wav OK y exit code cero solo tras finalizar. Errores de creación,
escritura o finalización dan código no cero. Un fallo posterior a crear el archivo
puede dejar un resultado parcial en la ruta indicada; inspeccionarlo y quitarlo
explícitamente antes de reutilizar el nombre. No se sobrescribe un destino existente.

El motor produce las muestras, daw-app coordina bloques y la CLI escribe el WAV.
El servicio de audio, transporte vivo y documento permanecen independientes.
Los nueve fixtures JSON anteriores siguen probando solo smoke; la cobertura WAV
está en tests Rust de CLI/aplicación, ejecutados también por check y CI.

## Salida nativa (silencio)

```powershell
cargo run -p daw-cli --features native-audio --locked -- devices
cargo run -p daw-cli --features native-audio --locked -- live-smoke --seconds 2
cargo run -p daw-cli --features native-audio --locked -- live-smoke --seconds 2 --device "<ID enumerado>"
```

`devices` muestra IDs, salida predeterminada y configuraciones anunciadas por el
driver. Copiar el ID completo al último comando. La inicialización usa la
configuración predeterminada soportada por esa salida; no hay selector de sample
rate o formato en esta etapa.

`live-smoke` confirma callbacks, Play/Pause/Stop y cierre. Su duración admite 1–60
segundos más esperas acotadas de confirmación. El número de frames es observado,
no una duración exacta offline. Sin la feature, las funciones nativas devuelven
NativeAudioDisabled. La salida sigue siendo silencio.
