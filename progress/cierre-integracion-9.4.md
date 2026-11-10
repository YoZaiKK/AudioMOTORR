# Cierre local de integración 9.4.d

Fecha: 2026-11-09. Base Git 1002efa; se conservan los cambios locales 9.3 y
9.4.a–c. Sin commit/push, hardware abierto, capturas nuevas ni CI remota nueva.
La etapa 9 completa sigue abierta: 9.5 mide y verifica el sampler nativo.

Registro fechado de 9.4.d. [9.5.a](etapa-9-diagnosticos-sampler.md) implementa
posteriormente los diagnósticos aquí pendientes, con nueva huella/evidencia offline.
Se conservan los resultados siguientes; hardware físico sigue sin verificar.

## Auditoría y correcciones

La [matriz integrada](../docs/sampler-integration-acceptance.md) contrasta dieciséis
criterios con implementación y pruebas: propiedad de PCM, worker/permiso de
preparación, cancelación/fallo/IDs, formato exacto, transporte/rampas/headroom,
colas/presupuestos, preflight, aceptación/resultados, pérdida de snapshots,
memoria/cierre, IPC/ACL y comportamiento UI. Cada fila identifica evidencia y
límites. Los contratos de motor, servicio, preparación y desktop siguen vigentes.

Se detectaron y corrigieron dos defectos en useDawSession:

1. Enumeración fallida de dispositivos evitaba consultar preparación. Un banco
   listo quedaba invisible/no descartable. Ahora ambas consultas iniciales son
   independientes y el fallo de dispositivos no oculta el banco.
2. Cleanup del efecto descartaba respuestas viejas, pero no cancelaba el invoke
   ya enviado. Cancelar/reemplazar podía iniciar otro poll mientras el anterior
   seguía en vuelo. Un guard persistente impide solapar polls automáticos a
   través de esos cambios; se reintenta a 200 ms y se ignoran respuestas dispuestas.
   No se afirma serialización de todos los comandos IPC explícitos con el polling.

Tres casos de frontend reprodujeron el fallo antes del cambio: uno de enumeración
y dos de polling (loading → cancelling y cancelación → nueva carga). Después pasan,
junto con las 21 pruebas existentes. Log de reproducción no versionado:
target/stage9-4d/frontend-before.log. Las respuestas se controlan con promesas
retenidas; no se simula un unplug físico ni se mide un deadline de audio.

No cambia DSP, DTO/bindings, dependencias, lockfiles, capacidades de colas o
workflow. La nueva huella difiere de 9.4.c por hook/pruebas de frontend; los
manifiestos anteriores se conservan como snapshots fechados.

## Verificación local

```powershell
npm run test:frontend
npm run lint
npm run typecheck
npm run check
npm run desktop:build -- --debug --no-bundle
```

Resultados locales correctos del 2026-11-09:

- npm run check: 106 pruebas headless, 112 workspace, 26 audio instrumentadas,
  ocho memoria decoder/banco, 24 frontend y nueve escenarios CLI. Las suites se
  solapan y no se suman como pruebas únicas. La prueba física continúa ignorada.
- Bindings deterministas, TypeScript, ESLint, formato, build frontend, rustfmt
  y Clippy correctos dentro del check completo.
- npm run desktop:build -- --debug --no-bundle correcto después del check;
  target/debug/rust-daw-desktop.exe, sin instalador, firma o publicación.
- Cargo.lock, package-lock.json, api.ts generado y workflow mantienen los hashes
  de 9.4.c y los mismos valores antes/después de check/build.

El [manifiesto actual](evidencia-integracion-9.4d.json) identifica 116 archivos de
código/configuración y solo dos modificados respecto de 9.4.c: hook y tests.
Logs locales no versionados en target/stage9-4d/check.log y desktop-build.log;
hashes de control en hashes-before.json/hashes-after.json. Los manifiestos
9.3/9.4.a–c conservan sus resultados fechados. La revisión final pasa formato,
519 destinos locales de enlaces/imágenes Markdown, diez límites del contrato,
actionlint y git diff --check; los 116 archivos coinciden con su manifiesto.

## Criterio de cierre y siguiente etapa

9.4 se cierra como **integración local**, con aceptación por criterio y regresiones
automatizadas; no como validación física completa. Se mantienen pendientes las
mediciones nativas del sampler, audición, desconexión física, loopback, estrés,
otros formatos/dispositivos y Linux/macOS nativos. La CI publicada del kernel
7acc1db no verifica estos cambios locales ni se extrapola a ellos.

Siguiente cambio concreto: **9.5.a**, implementar el runner diagnóstico de sampler,
primero benchmark offline y después recorrido nativo opt-in. Usar las APIs
existentes de preparación/sesión/trigger; buffers de medición preasignados,
workloads de 0/1/16 voces, contadores de asignaciones **y liberaciones** y pruebas
sin dispositivo. Hoy engine-bench/tone-check ejercitan senoide, no sampler;
RealtimeMetrics no expone liberaciones ni percentiles nativos.

El [protocolo nativo](../docs/sampler-native-verification.md) define fixtures,
invocaciones existentes, runner todavía ausente, región medida, aserciones y
resultados inconclusos. 9.5.b ejecutará/verificará hardware; 9.5.c consolidará
reporte, límites y CI del SHA publicado. No se implementa ese runner en 9.4.d ni
se adelanta mezclador, secuenciador o persistencia (10–12).
