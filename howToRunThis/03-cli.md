# CLI y archivos de prueba

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

## Salida nativa

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
