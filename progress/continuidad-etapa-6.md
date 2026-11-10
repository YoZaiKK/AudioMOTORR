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
Mediciones/audición física y cierre completo siguen pendientes en 9.5.
Ver [reporte actual](etapa-9-voces.md) para SHA completo, enlaces de jobs y pruebas
locales. La etapa 9 completa sigue abierta. 9.3.b carga recursos WAV individuales;
9.3.c prepara bancos con permiso compartido y presupuesto agregado; 9.3.d verifica
fallos/cancelación/cierre entre hilos y junto a la entrega; 9.3.e verifica fixtures
WAV → banco → Sampler offline y matriz de aceptación. 9.3.f cierra preparación
localmente. 9.4.a integra motor y 9.4.b conecta sesiones manual/CPAL y outcomes,
sin abrir hardware en pruebas. 9.4.c añade preparación/cancelación en worker y
controles explícitos. 9.4.d cierra integración local; 9.5.a entrega diagnósticos
sampler y benchmarks sin dispositivo. Siguiente 9.5.b: mediciones físicas. Ver
[reporte actual](etapa-9-diagnosticos-sampler.md). La revisión vigente es 0c346d5
(Phase 9.5.a), con main y origin/main locales coincidentes; su CI remota no se ha
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

Continuar 9.5.b: ejecutar sampler-native-check release en dispositivo/config
identificados, WAV compatibles de seconds+4 a 15 segundos y hashes antes/después.
Registrar ventanas 0/1/16, IDs exactos, voz/frames reales, memoria/tiempos/histogramas,
transporte, finalización, cierre/reapertura y resultados inconclusos. Seguir el
[contrato](../docs/sampler-diagnostics.md) y [protocolo](../docs/sampler-native-verification.md).
La invocación abre audio real; en 9.5.a solo se compiló y probó con backend manual.

9.5.a implementa sampler-bench (matriz 12, mono/estéreo distinto) y sampler-native-check.
offline-diagnostics excluye CPAL, rt-diagnostics conserva audio nativo. MemoryActivity
cuenta asignaciones/liberaciones; callbacks observan todos los canales y un
histograma fijo con percentiles aproximados. Application expone metadatos Rust
del recurso preparado; DTO/IPC/UI, locks y workflow no cambian. El benchmark
release mide CPU offline, sin garantía de deadlines o continuidad física.
Ver [reporte actual](etapa-9-diagnosticos-sampler.md); ahí se registra check/build
y el [nuevo manifiesto](evidencia-diagnosticos-9.5a.json) de 119 archivos.
Check/build correctos: 108 headless/114 workspace/29 audio/nueve diagnósticos CLI/
ocho memoria/24 frontend/nueve escenarios. Release focal y matrices mono/estéreo
correctos, sin abrir hardware. Reportes/manifiestos anteriores conservados como históricos.

9.5.c consolidará cierre y CI del SHA publicado. Siguen pendientes audición,
unplug, loopback, estrés y plataformas nativas no verificadas. No se adelantan
mezclador/secuenciador/persistencia de etapas 10–12.

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
