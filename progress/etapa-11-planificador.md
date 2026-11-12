# Etapa 11.2 — Planificador y reproducción mezclada secuenciada

Fecha: 2026-11-12. **Implementado localmente** sobre el modelo 11.1, todavía sin
commit/push. Base Git: `c8d7260aaa15b46dfb9c337bd2c11df6b3e57d15` más los cambios
locales de 11.1. La etapa 11 completa sigue abierta.

## Comportamiento implementado

- Constructor explícito `AudioEngine::with_sequenced_sampler`: detenido al
  inicializar, comparte banco, pool global de 16 voces, rutas, mezcla y rampas.
- Fase musical exacta en `u128`, denominador fijo `15 × sample_rate × 2^48`.
  Los BPM validados se representan sin aproximación adicional. Fronteras
  absolutas redondeadas hacia arriba; no se suma una duración de paso redondeada.
- Render hasta cada frontera, dispara filas en orden antes del frame asignado
  y continúa. Ventanas semiabiertas: una frontera al final queda pendiente.
  Varias fronteras en un mismo frame conservan orden musical y después por fila.
- Play continúa sin duplicar notas; Pause/Stop congelan la música al solicitarse
  y dejan terminar el release de las voces. Stop resetea al asentarse, retiene
  configuración/banco y conserva contadores de diagnóstico de toda la sesión.
- Cambios directos de BPM/pasos/slots/patrón conservan fase; patrón completo
  reemplazado de forma atómica. Notas emitidas y colas existentes se conservan.
- Slots ausentes/pool lleno rechazan solo esa nota, sin reintentos ni voice
  stealing. Triggers manuales previos comparten capacidad y orden de aplicación.
- Snapshot Copy con patrón aplicado, posición exacta del próximo frame, última
  frontera realmente observada, contadores y último evento/resultado. El último
  evento no es un historial ni una confirmación de comando de aplicación.
- Preflight del bloque completo antes de tocar audio, voces, medidores o fase;
  errores de alineación/overflow dejan salida silenciosa y estado intacto.

Detalle y API: [contrato actualizado](../docs/sequencer-contract.md).

## Verificación

Cinco pruebas unitarias nuevas: oráculo racional independiente por descomposición
IEEE-754, particiones 1/127/256/4096/bloque completo, tasas 1/7/800/44100/48000,
cinco BPM (incluido un ULP junto a 120), deriva en 100 000 pasos a 127.25 BPM,
tasa `u32::MAX`, conteo exacto de patrones densos y overflow atómico incluso
cuando ocurriría después del primer frame.

Diez pruebas de integración nuevas: inicialización/errores, pausa y frontera
pendiente, Stop interrumpido/asentado, BPM y fase, ediciones y colas, ausencias,
pool compartido, fronteras coincidentes, silencio y posición observada. El beat
de referencia compara PCM mono/estéreo con cuatro fuentes distintas, gain/pan/mute,
20/95.5/127.25/300 BPM y cinco particiones. También compara una sesión con cambios
de patrón/rutas/BPM/gain y transporte, conservando muestras y estado final.

Una prueba instrumentada nueva recorre seis combinaciones: tasas 1/800/48000 ×
salida mono/estéreo. Notas, rechazos, ediciones, release, reanudación, Stop y
finalización registran cero asignaciones y liberaciones en procesamiento.
Control positivo: exactamente cuatro buffers PCM se liberan al destruir el motor
fuera de la ventana medida. El preflight de relojes del motor también incluye
ahora el modo secuenciado.

La primera pasada release detectó un falso overflow al contar notas densas:
la reducción de un array `u128` produjo bits altos adicionales en esa ejecución
optimizada. El conteo por paso/compás usa ahora `u64` (máximos 4/64); solo se
amplía a `u128` el producto por repeticiones. La regresión verifica conteos desde
distintas posiciones, incluso `u64::MAX` fronteras, y se conserva el log del fallo
inicial. Los resultados finales corresponden al código corregido.

Verificación final correcta:

- `npm run check`, código 0, 08:08:54–08:10:45 UTC: bindings, tipos, lint,
  formato, build web, 39 casos frontend, nueve escenarios CLI, suites Rust
  headless/workspace/audio instrumentado/diagnósticos/memoria y Clippy.
- Release del motor: nueve unitarias y diez de integración seleccionadas,
  todas correctas. Incluyen las cinco unitarias y diez de integración nuevas.
- Prueba de memoria release: correcta en las seis combinaciones, con el
  control positivo de liberación de PCM. También pasa en debug durante el check.

Son 16 pruebas nuevas entre motor y memoria. Las suites se solapan; no se suman
sus ejecuciones como pruebas distintas. El check compila pruebas de escritorio
y build web, sin realizar un bundle/build completo de la aplicación.

Resultados completos y hashes en el
[manifiesto 11.2](evidencia-planificador-11.2.json); logs locales en
`target/stage11-2-20261112/`. Los manifiestos 11.1/10 conservan su captura histórica.

## Siguiente incremento y límites

**11.3:** sesiones secuenciadas manuales/CPAL, comandos acotados, coordinación
del documento/preparación y estado aplicado, DTO/IPC/ACL y cuadrícula de 4 × 16
con posición observada. El nuevo constructor y controles actuales son del
propietario de procesamiento; no aceptan comandos por cola ni actualizan Project.
El próximo incremento concreto es **11.3.a, servicio de audio**, según el
[desglose actualizado](plan-siguientes-pasos.md).

No se modificó la interfaz para reproducir patrones ni se abrió hardware. La
aceptación integrada, diagnósticos nativos y CI de la revisión entregada siguen
en 11.4; no se atribuyen resultados de etapa 10 al secuenciador. Persistencia y
exportación de beats siguen en 12.
