# Continuidad tras etapa 6

Actualizado el 2026-11-08. Etapas 1–7 completadas; etapa 8 implementada localmente:
senoide nativa explícita, rampas de 5 ms, volumen y diagnóstico release.
El [reporte actual](etapa-8.md) registra verificación y publicación/CI del incremento.
Base publicada 818e84b; implementación de etapa 7 c6cf627 con CI Ubuntu/Windows
correcta. El plan actual es [etapas 7–13](plan-etapas-7-13.md).
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

Completar el cierre local/CI de etapa 8 según el reporte y después comenzar 9.1:
contrato del sampler y fixtures en memoria. Fijar formato/rate/canales, propiedad
inmutable, pool fijo y política de agotamiento antes de escribir reproducción
en daw-engine. Preparar/decodificar recursos fuera del callback en otro incremento.

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

Pendientes independientes: CI de nuevos cambios hasta publicar/inspeccionar el SHA,
Linux nativo/macOS, desconexión física, otros formatos de hardware, loopback y
estrés prolongado. Se midieron allocator y tiempo de procesamiento release sobre
Realtek, sin inferir continuidad acústica ni garantía de deadlines del host.
La documentación técnica es inglesa;
progress y howToRunThis permanecen en español.
