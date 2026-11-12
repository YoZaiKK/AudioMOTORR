# Continuidad del proyecto

Actualizado el 2026-11-11. **Etapas 1–9 completadas dentro de su alcance**.
El [cierre del sampler](cierre-etapa-9.md) reúne preparación WAV, motor/servicio,
aplicación/IPC/UI y diagnósticos. La implementación está publicada en `4dc2c6c`,
con CI Linux/Windows correcta en run 42896571891, inspeccionada mediante la API
oficial. La [auditoría 9.5.c](auditoria-cierre-9.5c.md) verifica fuentes y artefactos;
el [manifiesto](evidencia-cierre-9.5c.json) conserva las repeticiones y la
confirmación del usuario «Si se escucha» tras corregir la conexión de audífonos.
Las pruebas físicas no realizadas se mantienen como límites.

**10.1 especificado:** [contrato de mezcla](../docs/mixer-contract.md).
**Núcleo 10.2 implementado localmente:** modelo/DSP, rutas capturadas, rampas,
picos por render, fixtures y memoria instrumentada; ver
[reporte](etapa-10-mezclador-motor.md) y [manifiesto](evidencia-mezclador-10.2.json).
**Servicio 10.3.a implementado localmente:** sesiones mezcladas, controles FIFO
y ventanas de picos por callback; [reporte](etapa-10-sesiones-mezclador.md) y
[manifiesto](evidencia-sesiones-mezclador-10.3a.json).
**Aplicación 10.3.b implementada localmente:** ajustes seleccionados, aceptación
explícita y estado aplicado diferenciados; instalación con ajustes actuales.
[Reporte](etapa-10-aplicacion-mezclador.md) y [manifiesto](evidencia-aplicacion-mezclador-10.3b.json).
**Desktop 10.3.c implementado localmente:** DTO/IPC/ACL, cuatro tiras, rutas y niveles.
[Reporte](etapa-10-desktop-mezclador.md) y [manifiesto](evidencia-desktop-mezclador-10.3c.json).
**Aceptación 10.3.d completada localmente:** referencias de cuatro WAV/canales,
lifecycle, pool y correcciones de UI; [reporte](cierre-integracion-mezclador-10.3d.md)
y [manifiesto](evidencia-integracion-mezclador-10.3d.json).
**Diagnósticos 10.4.a implementados localmente:** mixer-bench, mixer-native-check
y protocolo; [reporte](etapa-10-diagnosticos-mezclador.md),
[manifiesto](evidencia-diagnosticos-mezclador-10.4a.json).
**Matriz nativa 10.4.b medida:** seis casos/reaperturas Speakers Realtek;
[reporte](etapa-10-medicion-nativa-mezclador.md), [evidencia](evidencia-nativa-mezclador-10.4b.json).
**Auditoría local 10.4.c completada:** 136 fuentes y 21 artefactos verificados;
[reporte](auditoria-cierre-mezclador-10.4c.md), [manifiesto](evidencia-auditoria-mezclador-10.4c.json)
y [entrega preparada](entrega-mezclador-etapa-10.md).
**Pendiente:** respuesta auditiva y entrega/CI del SHA nuevo antes del cierre.
Ver [plan de ejecución](plan-siguientes-pasos.md) y [plan general](plan-etapas-7-13.md).
El código y la documentación nuevos son locales, sin commit/push ni CI propia.
Los hashes de etapa 9 no se atribuyen a las modificaciones del mezclador 10.2.
Los reportes anteriores conservan sus resultados y estado de publicación fechados.

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

La matriz **10.4.b** ya pasó con seis casos/reaperturas Speakers Realtek.
La auditoría local de **10.4.c** está completada. Registrar respuesta auditiva
y entregar la revisión preparada para verificar su CI,
conforme al [protocolo](../docs/mixer-native-verification.md). Diagnósticos 10.4.a listos. La aceptación local 10.3.d está completada. [Desktop 10.3.c](etapa-10-desktop-mezclador.md) ya
expone selección/recibos/observación, cuatro tiras, rutas y niveles. Etapa 10
sigue abierta hasta la comprobación nativa y CI correspondiente de 10.4.

## Evidencia histórica anterior al cierre

9.5.b ya ejecutó seis casos release Realtek/WASAPI a 48000 Hz, dos canales f32,
con entrada mono/estéreo distinta y 0/1/16 voces. Cero asignaciones/liberaciones,
non-finite y overruns medidos en las doce sesiones inicial/reapertura. Máximo
observado 432100 ns, ratio 0.04321. Ver [reporte](etapa-9-medicion-nativa.md) y
[evidencia](evidencia-nativa-9.5b.json). Métricas acumuladas de todo el lifecycle,
con histogramas aproximados y un callback de retraso; no son percentiles aislados.

El runner imprime IDs/resultados y métricas de reapertura fuera del callback.
El generador scripts/sampler-stereo-fixture.mjs reproduce el WAV medido exactamente
y rechaza sobrescritura. DSP, DTO/IPC, dependencias y locks no cambian. Check/build
Windows, release focal y verificación de hashes quedan registrados en el reporte.
Artefactos locales: target/stage9-5b-20261110-134708, piloto conservado y matriz
completa trazada. Escucha humana, loopback, unplug, estrés, otros formatos físicos
y plataformas nativas siguen sin confirmar/realizar. Etapa 9 aún abierta.

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
