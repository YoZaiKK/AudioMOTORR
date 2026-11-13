# Cierre de etapa 12 — Proyectos portables y exportación

**Actualización:** cierre reabierto para validar/entregar la
[corrección de precisión JSON](correccion-12-precision-json.md). El texto y CI
siguientes corresponden a la entrega inicial, anterior al caso nuevo.

Etapa 12 completada dentro de su alcance. Implementación entregada en main:
[`6c8c52e820081249a62ab5b664f94853568a4af6`](https://github.com/YoZaiKK/AudioMOTORR/commit/6c8c52e820081249a62ab5b664f94853568a4af6).
Ambos jobs del SHA exacto pasaron en
[run 43348424541](https://github.com/YoZaiKK/AudioMOTORR/actions/runs/43348424541):
[Linux](https://github.com/YoZaiKK/AudioMOTORR/actions/runs/43348424541/job/131633282365)
y [Windows](https://github.com/YoZaiKK/AudioMOTORR/actions/runs/43348424541/job/131633281881).

## Criterios completados

- **12.1:** JSON UTF-8 v1 con identidad exacta, título, BPM, patrón, mezcla,
  rutas y referencias relativas. Validación completa y rechazo explícito de
  versiones desconocidas; estado vivo separado del documento.
- **12.2:** guardar/abrir con copia de assets, destino exclusivo o reemplazo
  explícito, commit de archivo y errores sin perder documento/archivo anterior.
  Round-trip y relocación comprobados tras eliminar los originales.
- **12.3:** render independiente del mismo motor secuenciado, compases exactos,
  colas cut/finish y WAV float32, equivalentes entre bloques 1/127/256/4096.
  Validaciones antes de crear salida, errores propagados y no overwrite.
- **12.4:** aplicación, CLI, DTO/IPC/ACL y UI integradas; 20 casos nuevos,
  check completo/release/build, flujo real WebView2 y CI de la revisión entregada.

La huella de 164 fuentes es
`9a1cdc442eb36256a03bef4a4d4417dadc4739db368812f29cc52ca8e823df39`.
Se contrastó con Git: 160 archivos idénticos byte a byte y cuatro equivalentes
tras normalizar finales de línea. El seguimiento documental no cambia producción.

[Reporte](etapa-12-persistencia-exportacion.md),
[manifiesto local](evidencia-persistencia-exportacion-12.json),
[auditoría/publicación](publicacion-secuenciador-proyectos-11-12.md) y
[evidencia de entrega](evidencia-publicacion-11-12.json).
La [guía de uso](../howToRunThis/09-proyectos.md) describe el flujo actual.

## Alcance conservado

La [etapa 11 está cerrada](cierre-etapa-11.md) tras la confirmación auditiva
«Si se escucha», con sus límites registrados. Hay seis casos físicos y seis
reaperturas exitosos sobre las 156 fuentes medidas; sus tiempos no se atribuyen
a las 164 fuentes nuevas. Una repetición adicional con el ejecutable optimizado
actual no inició casos por errores locales de acceso al archivo. La incidencia
se conserva, sin cambiar ajustes de seguridad ni inventar resultados.

El cierre de 12 valida persistencia/render/exportación offline y su integración;
no certifica continuidad acústica, loopback, unplug ni estrés prolongado. No
incluye guardar voces/posición, master transitorio, dispositivo o historial de
comandos. No hay remuestreo, recuperación ante corte eléctrico ni limpieza
automática de carpetas de assets anteriores. El siguiente producto es **13.1:
contrato de clips/escenas y reglas de lanzamiento/cuantización**.
