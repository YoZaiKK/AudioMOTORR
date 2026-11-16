# Diagnósticos de clips y escenas

El benchmark funciona sin dispositivo. El verificador nativo está implementado,
y la [matriz física 13.4.b](../progress/etapa-13-medicion-nativa-clips.md) ya pasó
seis casos y seis reaperturas. Falta confirmar escucha; las métricas no la sustituyen.

```powershell
cargo run -p daw-cli --release --features offline-diagnostics --locked -- launcher-bench --channels 2
cargo run -p daw-cli --release --features offline-diagnostics --locked -- launcher-bench --clips beat --bpm 127.25 --quantization bar --block-size 127 --bars 2
```

Por defecto se ejecutan 108 combinaciones por salida: bloques 1/127/256/4096,
clips vacíos/musicales/densos, tres BPM y cuantizaciones paso/pulso/compás.
Usa `--channels 1` para mono. Los presets BPM son 20/95.5/127.25/300; los compases
admitidos son 1..8. Un resultado correcto confirma referencias y memoria, no que
el sistema cumpla siempre un presupuesto de tiempo real. Conserva también los
valores `overruns` y los máximos.

El benchmark compara cada muestra y observación con una referencia independiente,
incluyendo fases, transiciones, Keep/supersesión, StopAll, voces y medidores.
Separa tiempo de procesar del tiempo de comparar, reserva antes de medir y
comprueba que no se asigne ni libere memoria durante el bucle.

Para preparar la siguiente medición nativa:

```powershell
node scripts/sequencer-fixtures.mjs target/launcher-input --channels 2 --sample-rate 48000
cargo build -p daw-cli --release --features rt-diagnostics --locked
target/release/daw-cli.exe devices
```

El generador reutilizado crea cuatro WAV y un manifiesto; exige una carpeta nueva.
El comando nativo requiere `--input-dir` y `--device` explícitos, sin alternativa
automática. Su ejecución produce audio incluso si eliges clips vacíos: las fases
posteriores comprueban controles y reapertura con señal. Sigue el
[protocolo completo](../docs/launcher-native-verification.md) para elegir
formato, registrar resultados y separar escucha de métricas de procesamiento.

La app ya permite editar patrones/clips, mezclar cuatro canales, guardar proyectos
portátiles y exportar WAV. Estos comandos son herramientas CLI de diagnóstico;
no añaden grabación, MIDI, plugins ni una línea de tiempo.
Ver [guía del launcher](10-clips-escenas.md) y
[avance de diagnósticos](../progress/etapa-13-diagnosticos-clips.md).
