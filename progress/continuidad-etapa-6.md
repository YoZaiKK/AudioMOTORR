# Continuidad del proyecto

Actualizado el 2026-11-10. Etapas 1–8 completadas; etapa 8 publicada en 1e6bca8:
senoide nativa explícita, rampas de 5 ms, volumen y diagnóstico release. Ambos
jobs de CI correctos en run 42524752652, inspeccionado para ese SHA exacto.
El [reporte de etapa 8](etapa-8.md) registra su verificación y publicación/CI.
Estado actual: incrementos 9.1–9.2 completados; kernel 9.2 publicado en 7acc1db
con ambos jobs CI correctos en run 42530699397. Hay banco fijo y 16 voces raw en
motor. 9.4.a integra AudioEngine con transporte/rampas/headroom; sesiones de
audio conectadas en 9.4.b con pruebas de adaptador. 9.4.c añade Application/DTO/IPC/UI
con mocks; 9.4.d audita/cierra integración local y corrige recuperación/polling.
9.5.b registra seis mediciones Realtek/WASAPI correctas; confirmación de escucha
y cierre completo/CI siguen pendientes.
Ver [reporte actual](etapa-9-voces.md) para SHA completo, enlaces de jobs y pruebas
locales. La etapa 9 completa sigue abierta. 9.3.b carga recursos WAV individuales;
9.3.c prepara bancos con permiso compartido y presupuesto agregado; 9.3.d verifica
fallos/cancelación/cierre entre hilos y junto a la entrega; 9.3.e verifica fixtures
WAV → banco → Sampler offline y matriz de aceptación. 9.3.f cierra preparación
localmente. 9.4.a integra motor y 9.4.b conecta sesiones manual/CPAL y outcomes,
sin abrir hardware en pruebas. 9.4.c añade preparación/cancelación en worker y
controles explícitos. 9.4.d cierra integración local; 9.5.a entrega diagnósticos
sampler y benchmarks sin dispositivo. 9.5.b registra mediciones nativas; falta
confirmación auditiva y consolidación 9.5.c. Ver [reporte actual](etapa-9-medicion-nativa.md).
La base vigente es b0ba589 (Update config), más cambios locales de trazas/fixture;
su CI remota no se ha
inspeccionado aquí. Ver [revisión de configuración](revision-readme-configuracion.md).
El manifiesto 9.3 se conserva como histórico, sin coincidir con este nuevo motor.
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

Incorporar la confirmación auditiva humana solicitada y consolidar 9.5.c:
revisar aceptación, límites y CI del SHA que se publique. No se da la escucha por
confirmada desde probes; no se ha hecho commit/push de este trabajo.

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
