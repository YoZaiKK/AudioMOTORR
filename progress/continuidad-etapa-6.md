# Continuidad del proyecto

Actualizado el 2026-11-08. Etapas 1–8 completadas; etapa 8 publicada en 1e6bca8:
senoide nativa explícita, rampas de 5 ms, volumen y diagnóstico release. Ambos
jobs de CI correctos en run 42524752652, inspeccionado para ese SHA exacto.
El [reporte de etapa 8](etapa-8.md) registra su verificación y publicación/CI.
Estado actual: incrementos 9.1–9.2 completados; kernel 9.2 publicado en 7acc1db
con ambos jobs CI correctos en run 42530699397. Hay banco fijo y 16 voces raw en
motor, sin carga de archivos, AudioEngine/servicio o UI de sampler todavía.
Ver [reporte actual](etapa-9-voces.md) para SHA completo, enlaces de jobs y pruebas
locales. La etapa 9 completa sigue abierta y 9.3 es el siguiente incremento.
El plan actual es [etapas 7–13](plan-etapas-7-13.md).
Las referencias anteriores se conservan como historial fechado.

La etapa 6 añade documentación técnica y CI sin cambiar el producto. Consultar
[cierre y auditoría](cierre-etapa-6.md), [checklist](checklist.md) y [detalles](detalles.md).
El historial de [etapa 4](retomar-etapa-4.md) y el
[reporte de etapa 5](../howToRunThis/resultados.md) se conservan.

El registro histórico anterior integró las etapas 4–6 sobre el commit de etapa 3,
69467ef. El usuario solicitó revisar la documentación y hacer commit/push del estado
completo el 2026-11-06. El historial de Git identifica el commit de esta entrega;
usar git status y git log para comprobar cambios posteriores y sincronización.
Los [resultados de Actions](https://github.com/YoZaiKK/AudioMOTORR/actions/workflows/ci.yml)
corresponden a cada revisión publicada y deben consultarse por separado de las
pruebas locales. Los reportes anteriores conservan su estado de Git al cierre
de cada etapa como evidencia histórica.

## Próximo cambio concreto

Continuar 9.3: preparación/carga y decodificación acotadas fuera del procesamiento
mediante aplicación/adaptador de I/O. Fijar encodings WAV, límites de bytes/frames
y memoria temporal, errores de metadatos/rate/canales y cierre/cancelación seguros.
Crear un banco completo validado antes de empezar; no modificar recursos vivos
ante un fallo. Mantener integración nativa/IPC/UI para 9.4. Ver el
[contrato del sampler](../docs/sampler-contract.md) para límites y políticas.

En 7.3, daw-app coordina render offline con buffers acotados y daw-cli escribe
WAV float32 fuera del render con hound 3.5.1. El render_manual existente solo devuelve un reporte,
no las muestras: no usarlo como si ya fuera un exportador. 7.4/7.5 integran pruebas,
documentación y evidencia de cierre. Seguir el [contrato](../docs/offline-oscillator.md)
y el [roadmap](../docs/roadmap.md). El [contrato nativo](../docs/native-audition.md)
concreta los cambios de etapa 8 y los límites de sus mediciones.

## Cómo verificar

```powershell
npm ci
npm run doctor
npm run bindings:generate
npm run bindings:check
npm run check
npm run desktop:build -- --debug --no-bundle
```

Comparar hashes antes/después cuando existan cambios locales previos. La CI usa
git diff en checkout limpio. Mantener check antes de desktop build. El smoke
opcional requiere hardware y no pertenece a CI; no actualizar capturas salvo
petición explícita. Pruebas IPC con mock pueden necesitar acceso a datos de app
fuera del sandbox; eso no implica abrir audio real.

Pendientes independientes: CI de futuros cambios hasta publicar/inspeccionar su SHA,
Linux nativo/macOS, desconexión física, otros formatos de hardware, loopback y
estrés prolongado. Se midieron allocator y tiempo de procesamiento release sobre
Realtek, sin inferir continuidad acústica ni garantía de deadlines del host.
La documentación técnica es inglesa;
progress y howToRunThis permanecen en español.
