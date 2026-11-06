# Continuidad tras etapa 6

La etapa 6 añade documentación técnica y CI sin cambiar el producto. Consultar
[cierre y auditoría](cierre-etapa-6.md), [checklist](checklist.md) y [detalles](detalles.md).
El historial de [etapa 4](retomar-etapa-4.md) y el
[reporte de etapa 5](../howToRunThis/resultados.md) se conservan.

Esta entrega integra las etapas 4–6 sobre el commit de etapa 3, 69467ef.
El usuario solicitó revisar toda la documentación y hacer commit/push del estado
completo el 2026-11-06. El historial de Git identifica el commit de esta entrega;
usar git status y git log para comprobar cambios posteriores y sincronización.
Los [resultados de Actions](https://github.com/YoZaiKK/AudioMOTORR/actions/workflows/ci.yml)
corresponden a cada revisión publicada y deben consultarse por separado de las
pruebas locales. Los reportes anteriores conservan su estado de Git al cierre
de cada etapa como evidencia histórica.

## Próximo cambio concreto

Añadir oscilador senoidal de baja amplitud offline dentro de daw-engine,
reutilizando buffers/transportes existentes y conservando fase entre bloques.
Probar equivalencia por particiones, frecuencia/amplitud y número exacto de frames;
después coordinar exportación WAV fuera del callback mediante aplicación/CLI.
Seguir el orden y aceptación del [roadmap](../docs/roadmap.md), sin anticipar sampler
o secuenciador ni añadir abstracciones genéricas sin uso.

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

Pendientes independientes: confirmar resultado remoto del workflow, Linux/macOS,
desconexión física, otros formatos de hardware y mediciones de allocator/deadlines
y continuidad en release con señal. La documentación técnica es inglesa;
progress y howToRunThis permanecen en español.
