# Etapa 14.2 — Reproducción de arreglos en el motor

Fecha: 2026-11-16 (México). **Implementada localmente.** Base Git `f5f7ded`
más los cambios locales de 14.1, conservados sin alterar sus tres fuentes.
La etapa 13 mantiene su escucha pendiente; esta continuación no la confirma.

## Entrega y comportamiento

`AudioEngine::with_arranged_sampler` recibe banco preparado, tempo, mezcla y
arreglo resuelto. Construye fuera del procesamiento una representación fija
ordenada por tiempo que conserva las identidades de región originales. No hay
Play implícito ni edición de contenido instalado durante reproducción.

El planificador reutiliza el reloj exacto del secuenciador, el sampler, la mezcla,
las rutas capturadas y las rampas maestras. Las regiones contiguas cambian antes
de emitir la nota de paso cero; los huecos no programan notas y las colas siguen
sonando. Observa región/clip/paso, eventos, resultados de voces y fase aplicada.
Los samples ausentes y el pool lleno producen resultados acotados sin reintentos.

Al alcanzar el fin exclusivo del arreglo, congela posición y reloj musical. La
frontera final se observa al procesar el frame donde cae: renderizar exactamente
la duración puede dejar esa frontera pendiente para el siguiente frame. En tasas
muy bajas, pasos anteriores pueden coincidir en ese mismo frame y se consumen en
orden antes del final. Después, no programa más notas y deja terminar las colas
con el transporte en Playing. No introduce Stop automático ni un fade final.
El arreglo vacío finaliza sin notas en su primer frame Playing.

Pause congela la música inmediatamente y libera la envolvente como los otros
modos; Play reanuda sin duplicados. Stop interrumpido conserva fase; Stop asentado
reinicia reloj/cursores/final y elimina voces, conservando contenido y contadores
de vida del motor. Para volver a reproducir un arreglo terminado, asentar Stop
antes de Play. Cambiar BPM conserva fase musical y posiciones de las regiones.
Detalles y alcance futuro en el [contrato](../docs/arrangement-contract.md).

La validación completa precede a modificar PCM o estado: revisa contadores de
motor/sampler/medidores, reloj, fronteras y cantidad exacta de notas. Errores de
alineación o desbordamiento dejan salida en silencio y estado intacto. El conteo
recorre regiones y fragmentos de 16 pasos, sin depender de la longitud del bloque.
Los modos anteriores conservan sus constructores y comportamiento.

## Verificación

`cargo test -p daw-engine --release --locked`: **91 pruebas correctas**, nueve
nuevas de arreglo. La suite completa del motor conserva regresiones del
secuenciador, launcher, mezcla, sampler, senoide y rampas. Clippy de engine/CLI
con `offline-diagnostics`, todos los targets y `-D warnings`: correcto.

Las dos pruebas de referencia ejecutan **160 variantes**: tasas 1/7/800 Hz,
salidas mono/estéreo, BPM 20/95.5/127.25/300 y el siguiente `f64` tras 120,
particiones 1/127/256/4096/bloque completo y una secuencia de cambios de tempo.
Un oráculo separado descompone IEEE-754 y calcula posiciones desde segmentos
absolutos; simula sus propias voces, mezcla y envolvente sin llamar DSP/reloj del
motor. Compara notas/resultados, offsets, fronteras, fase, PCM y picos. PCM entre
particiones es idéntico; diferencia contra referencia como máximo `1e-7`.
La matriz incluye regiones contiguas, repeticiones, huecos, referencias a un mismo
sample, slots ausentes, rechazo por pool lleno, panorámica, rutas y mute.

Otras pruebas verifican Pause/Stop/reanudación y reinicio con cambios de mezcla,
errores atómicos antes de notas futuras o del final coincidente, buffers vacíos,
fin con colas y las 65537 fronteras del horizonte completo, a 1 y `u32::MAX` Hz.
Se amplía también la prueba existente de contadores globales al nuevo modo.

Memoria instrumentada: `arrangement_memory_contract`, con `offline-diagnostics`,
comprueba seis configuraciones (1/800/48000 Hz × mono/estéreo), las 128 regiones,
transporte/tempo/mezcla, observaciones y buffers 0/1/127/256/4096. Cero
asignaciones/liberaciones durante el procesamiento; al destruir fuera de él se
liberan exactamente los tres buffers PCM, con el tamaño esperado. **Debug y
release correctos.**
`scripts/check.mjs` incluye ahora esta prueba offline para que no quede omitida
por el selector de diagnósticos del check.

`npm run check`: **código 0**, con las 91 pruebas de motor en debug, bindings,
tipos, ESLint, 72 pruebas frontend, formato, build web, nueve escenarios CLI,
suites headless/workspace/audio instrumentado, diagnósticos, memoria y Clippy.
Las suites se solapan; sus ejecuciones no se suman como pruebas distintas.

Los logs y tiempos quedan en `target/stage14-2-20261116/`; el manifiesto
`evidencia-arreglo-motor-14.2.json` identifica las ocho fuentes/scripts añadidos
o modificados, la base local 14.1, documentación y resultados finales. Los tres
hashes fuente de 14.1 siguen idénticos. No se modifican dependencias,
lockfiles, bindings ni workflow.

## Siguiente paso y límites

**14.3.a: sesiones y coordinación de aplicación.** Instalar el snapshot fuera
del callback, definir nuevo modo explícito de sesión, comandos/observaciones
acotados y propiedad/cierre/reapertura. Después, 14.3.b integrará Project,
versionado y exportación; 14.3.c el editor desktop. [Plan](plan-etapa-14.md).

No hay todavía modo de arreglo en el proyecto, backend/IPC/UI ni exportación
temporal. No se abre hardware ni se atribuyen las mediciones o CI del launcher
a este camino. No se reconstruye el paquete desktop ni se cambian las capturas
existentes. Los cambios de 14.1 y 14.2 siguen locales, sin commit/push aquí.
