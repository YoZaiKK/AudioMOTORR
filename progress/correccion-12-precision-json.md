# Corrección 12 — Precisión de BPM, gain y pan en JSON

La revisión adicional del round-trip encontró un defecto después de la entrega
inicial `6c8c52e` y su seguimiento documental `866ec3f`. Sus CI/resultados previos
se conservan como capturas históricas; no cubrían este caso numérico.

El parser rápido por defecto de serde_json 1.0.154 puede variar un bit de ciertos
f64 al abrirlos. Ejemplos reproducidos: BPM `91.98927504254557` se leía como
`91.98927504254556`, gain `0.11940403280944911` y pan `-0.9125742275711077` también
cambiaban. Un sondeo determinista de 300000 valores legales encontró 33907
diferencias. La nueva regresión real de guardar/abrir falló antes del cambio:
bits de BPM observados `4636173586699427548` frente a `4636173586699427549`.

Se activa `serde_json/float_roundtrip` explícitamente en daw-app. No cambia la
versión/esquema del archivo, el serializador ni las versiones de dependencias.
La lectura conserva ahora el f64 representado por el literal JSON; no hay
migración implícita ni redondeo a BPM/gains simplificados.

Regresión permanente: `arbitrary_f64_tempo_gain_and_pan_survive_json_and_file_roundtrip_exactly`
en `project_persistence_contract.rs`, con archivo real y 1000 documentos
deterministas que contrastan bits de BPM/gain/pan. La etapa 12 se reabrió para
validar y entregar esta corrección; el cierre inicial conserva su fecha.

Verificación local completada: `npm run check` (63 frontend y 10 IPC entre
sus suites), release de proyectos (7 app + 2 CLI), build desktop y sondeo de
300000 valores con cero diferencias. En WebView2 real se abrió el proyecto
con los tres valores reproducidos y se conservaron exactamente; exportó
254157 frames (250464 musicales + 3693 de cola), 36 notas y cero rechazos,
sin inicializar audio. La regresión recorre además 1000 documentos.

Huella de 164 fuentes: `d0550238859dbe6000f1ca7e4b281322575ff3f706a79e6c65df6e8decb0e26d`.
[Evidencia](evidencia-precision-json-12.json), con checks y artefactos en
`target/stage12-precision-20261112`. Corrección publicada en `4f126dc`.
CI Linux/Windows del SHA exacto correcta en [run 43362943622](https://github.com/YoZaiKK/AudioMOTORR/actions/runs/43362943622); etapa 12 cerrada dentro de su alcance.
[Evidencia de entrega](evidencia-publicacion-precision-12.json). La [etapa 11 está cerrada](cierre-etapa-11.md) tras
«Si se escucha». La incidencia EPERM anterior conserva su alcance registrado.
