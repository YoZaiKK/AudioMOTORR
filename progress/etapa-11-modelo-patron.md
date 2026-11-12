# Etapa 11.1 — Contrato y modelo del patrón

Fecha: 2026-11-12. **Implementado localmente**, sobre el cierre de etapa 10
`c8d7260aaa15b46dfb9c337bd2c11df6b3e57d15`. La etapa 11 completa sigue abierta.

## Entrega

- `daw-domain::Pattern`: cuatro filas × 16 celdas on/off; cuatro semicorcheas
  por negra, un compás de 4/4. Inicialmente vacío, slots 0/1/2/3.
- Selección validada de cualquiera de los 16 slots por fila, independiente de
  las rutas del mezclador. Filas distintas pueden seleccionar el mismo sample.
- `events_at` produce hasta cuatro `StepEvent` ordenados por fila, sin heap ni
  reloj interno. Conserva eventos separados cuando comparten slot.
- `Project` posee el patrón; copias y ediciones independientes, sin alterar
  identidad, título, tempo o mezcla. Errores de índices rechazan sin mutación.
- [Contrato del secuenciador](../docs/sequencer-contract.md): bucle, redondeo
  hacia arriba de fronteras absolutas, fase exacta del BPM seleccionado,
  transporte/rampas y momento de aplicación de ediciones. Estas reglas guían
  el motor de 11.2 y la integración de 11.3.

## Verificación

Cinco pruebas nuevas verifican todas las celdas, todos los slots por fila,
silencio inicial, beat de referencia, orden/duplicados, errores atómicos y
propiedad independiente en `Project`. Las cuatro pruebas anteriores del
dominio también pasan.

`cargo test -p daw-domain --locked` pasa sus nueve pruebas. `npm run check`
terminó con código 0: bindings, tipos, ESLint, formato, build web, 39 casos
frontend, nueve escenarios CLI, suites Rust headless/workspace/audio
instrumentado/diagnósticos/memoria y Clippy. Sus suites se solapan; no se suman
como pruebas distintas. El check no abre un dispositivo de audio y no equivale
a un bundle/build completo de escritorio.

El [manifiesto de esta entrega](evidencia-modelo-patron-11.1.json) registra
138 fuentes: una modificada y dos nuevas respecto de etapa 10; las otras 135
conservan sus bytes. Incluye hashes de documentación y log del check, que duró
de 07:44:17 a 07:46:11 UTC del 2026-11-12.
Los logs locales quedan bajo `target/stage11-1-20261112/`.

## Alcance y siguiente paso

No hay todavía planificador ni reproducción de patrones, controles de patrón
en aplicación/IPC/UI, persistencia o exportación de beats. El sampler manual
y el mezclador conservan su implementación. No se repitió audio físico ni se
atribuye CI de etapa 10 a este cambio local. No se hizo commit/push en 11.1.

**Siguiente: 11.2**, fase musical acotada, eventos a offsets de frame dentro del
bloque y reproducción mezclada secuenciada explícita. Verificar referencia de
eventos/PCM independiente, BPM fraccionales, particiones, loops, comandos en
fronteras, pausa/release/reanudación, cambios de pasos/slots, pool agotado,
assets ausentes, overflow y cero asignaciones/liberaciones en procesamiento.
