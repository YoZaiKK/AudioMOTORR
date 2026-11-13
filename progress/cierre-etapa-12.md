# Cierre de etapa 12 — Proyectos portables y exportación

Etapa 12 completada dentro de su alcance, incluida la corrección de precisión
JSON encontrada después de la entrega inicial. Corrección publicada en
[`4f126dc`](https://github.com/YoZaiKK/AudioMOTORR/commit/4f126dc0d8c56b17c77160b4da93a4d251bcb0e4).
Ambos jobs del SHA exacto pasaron en [run 43362943622](https://github.com/YoZaiKK/AudioMOTORR/actions/runs/43362943622):
[headless-linux](https://github.com/YoZaiKK/AudioMOTORR/actions/runs/43362943622/job/131680490606) y [desktop-windows](https://github.com/YoZaiKK/AudioMOTORR/actions/runs/43362943622/job/131680490782).

## Criterios completados

- **12.1:** JSON UTF-8 v1 con identidad exacta, título, BPM, patrón, mezcla,
  rutas y referencias relativas. Rechazo explícito de versiones desconocidas,
  archivos inválidos y assets ausentes; documento separado del estado vivo.
- **12.2:** guardar/abrir con copia de assets, destino exclusivo o reemplazo
  explícito y errores sin perder documento/archivo anterior. Round-trip y
  relocación probados tras eliminar los originales. BPM/gain/pan conservan
  exactamente sus bits con serde_json/float_roundtrip.
- **12.3:** render independiente del mismo motor secuenciado, compases exactos,
  colas cut/finish y WAV float32, consistente entre bloques 1/127/256/4096.
  Validaciones previas a crear salida, errores propagados y no overwrite.
- **12.4:** aplicación, CLI, DTO/IPC/ACL y UI integradas; 21 casos nuevos
  únicos en conjunto, check completo, release/build y flujo real WebView2.

## Verificación final de la corrección

Check completo correcto, incluidos 63 frontend y 10 IPC entre sus suites.
Release de proyectos: 7 casos app + 2 CLI. Regresión permanente de archivo
real y 1000 documentos deterministas; sondeo independiente de 300000 valores
con cero diferencias, frente a 33907 antes del cambio. La nueva regresión
falló antes de activar la opción. Sin cambio de esquema ni de versiones.

WebView2 real abrió los tres valores reproducidos sin variación y exportó
254157 frames (250464 musicales + 3693 de cola), 36 notas y cero rechazos,
sin inicializar audio. Huella de 164 fuentes:
`d0550238859dbe6000f1ca7e4b281322575ff3f706a79e6c65df6e8decb0e26d`.
Equivalencia con Git: 160 archivos idénticos y cuatro solo con diferencias
de finales de línea. El seguimiento de cierre solo modifica documentación.

[Corrección y reproducción](correccion-12-precision-json.md),
[evidencia local](evidencia-precision-json-12.json) y
[evidencia de publicación/CI](evidencia-publicacion-precision-12.json).
La entrega inicial `6c8c52e`, su huella `9a1cdc44…`, sus capturas y
[CI inicial](https://github.com/YoZaiKK/AudioMOTORR/actions/runs/43348424541)
conservan su alcance histórico en el [reporte](etapa-12-persistencia-exportacion.md).
[Guía de uso](../howToRunThis/09-proyectos.md).

## Alcance conservado

La [etapa 11 está cerrada](cierre-etapa-11.md) tras «Si se escucha».
Sus seis casos físicos y seis reaperturas corresponden a las 156 fuentes
medidas; sus tiempos no se atribuyen a revisiones posteriores. Una repetición
optimizada adicional no inició casos por EPERM. Se conserva la incidencia
sin cambiar ajustes de seguridad ni inventar resultados.

El cierre de 12 valida persistencia/render/exportación offline e integración.
No certifica continuidad acústica, loopback, unplug ni estrés prolongado.
No incluye guardar voces/posición, master transitorio, dispositivo o historial
de comandos. No hay remuestreo, recuperación ante corte eléctrico ni limpieza
automática de carpetas anteriores. Siguiente producto: **13.1, contrato de
clips/escenas y reglas de lanzamiento/cuantización**, todavía sin iniciar.
