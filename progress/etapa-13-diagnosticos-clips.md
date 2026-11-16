# Etapa 13.4.a — Diagnósticos de clips y protocolo nativo

Estado: **implementada y verificada localmente**, 2026-11-16 (México).
Entrada: [aceptación integrada 13.3.e](cierre-integracion-clips-13.3e.md).
La etapa 13 sigue abierta; faltan medición/escucha física 13.4.b y entrega/CI 13.4.c.

## Entrega

- `launcher-bench`: fuente launcher explícita, clips vacíos/musicales/densos,
  Step/Beat/Bar, BPM racionales, bloques 1/127/256/4096 y salida mono/estéreo.
  Parcial con Keep/Stop, supersesión independiente, StopAll y relanzamiento.
- Oráculo independiente de cada muestra, evento, voz/rechazo, fase, origen por fila,
  pendientes/transiciones/IDs y medidores. Reservas previas; cronómetro solo para
  solicitudes del kernel y render, validación fuera del tiempo y dentro del
  contador de memoria. Cero reservas/liberaciones/bytes en procesamiento;
  liberación positiva de los cuatro PCM fuera del motor medido.
- `launcher-native-check`: dispositivo y cuatro WAV explícitos; ciclo compartido
  con pruebas de backend manual. Inicio vacío/detenido, escena inicial separada
  de interpretación, recibos exactos, timeline racional, pausa/ediciones, Keep y
  supersesión, BPM, recurso ausente, mute con notas, StopAll y colas naturales,
  Stop, cierre y reapertura vacía sin replay hasta un nuevo lanzamiento explícito.
- Warmup de al menos 66 callbacks antes de lanzar y en la sesión reabierta,
  para superar los 64 callbacks excluidos de la instrumentación incluso con
  bloques físicos grandes. Las esperas tienen límites y no reintentan lanzamientos.
- Métricas acumuladas por sesión, fases e IDs/counters exactos. Salida elegida
  verificada sin fallback; validación de opciones y gate nativo antes de I/O.
- Se reutilizan generador `scripts/sequencer-fixtures.mjs`, metadatos/preparación
  WAV y comprobaciones de niveles. Los comandos anteriores conservan su comportamiento.
- README con dos imágenes reales del editor/exportación, tabla de capacidades
  actuales y límites. Índice de guías corregido para no describir etapas 11/12
  ya cerradas como pendientes. Las capturas son históricas de 13.3.d/e, etiquetadas;
  esta entrega no cambia la UI ni abre una ventana gráfica.

[Contrato de diagnóstico](../docs/launcher-diagnostics.md),
[protocolo nativo](../docs/launcher-native-verification.md) y
[guía práctica](../howToRunThis/11-diagnosticos-clips.md).

## Pruebas y mediciones

**Siete casos nuevos únicos:** dos unitarios de benchmark/parser, dos del runner
nativo (parser y lifecycle), un gate native-disabled y dos contratos CLI release.
Features/perfiles solapan casos; no se suman como tests únicos. El lifecycle
manual usa WAV reales en 18 combinaciones de canales × clips × cuantización.

- Release con `offline-diagnostics`: 20 unitarios y siete contratos CLI correctos.
- Release con `rt-diagnostics`: 16 unitarios y siete contratos CLI correctos.
  Compila el runner nativo; las pruebas no abren hardware.
- El contrato CLI ejecuta 216 combinaciones y el preset adicional de 20 BPM;
  también prueba help, options/gates y fallo antes de leer recursos inválidos.
- Clippy de CLI con ambas features, `--all-targets --locked -- -D warnings`: correcto.
- `npm run check`: código 0, con 72 frontend, bindings/tipos/lint/formato,
  build web, nueve escenarios CLI y suites Rust headless/workspace/instrumentadas,
  diagnósticos/memoria y Clippy. Sin sumar suites solapadas.
- Build desktop Windows debug con assets después del check: correcto, sin instalador.
  Controles, IPC/ACL y bindings permanecen iguales; se conserva la evidencia
  WebView2 de 13.3.e como captura histórica, sin atribuirle una nueva ejecución.

Matriz offline final: **216 mediciones**, dos salidas × cuatro bloques × tres
clips × tres BPM × tres cuantizaciones, dos compases por ventana tras warmup.
CPU Intel i7-1185G7, Windows 10.0.26200 x64. Todas las referencias y memorias
pasan; teardown: cuatro liberaciones, 240000 bytes mono / 480000 estéreo.

| Matriz             | Salida  | Casos | Mayor p99 entre casos | Máximo individual | Excesos nominales |
| ------------------ | ------- | ----- | --------------------- | ----------------- | ----------------- |
| Final              | Mono    | 108   | 976400 ns             | 976400 ns         | 43                |
| Final              | Estéreo | 108   | 642300 ns             | 1114600 ns        | 32                |
| Inicial conservada | Mono    | 108   | 841600 ns             | 5825800 ns        | 910               |
| Inicial conservada | Estéreo | 108   | 5682800 ns            | 5682800 ns        | 59                |

La primera matriz se conserva completa. Se recompiló tras añadir warmup explícito
al runner nativo y se registró una matriz final del binario actualizado; el bucle
del benchmark no cambió. La inicial coincidió parcialmente con compilación/tests
de la variante nativa. No se seleccionaron repeticiones para obtener tiempos
favorables ni se borraron excesos. La inicial necesitó corregir el parser externo
del orden de campos de MemoryActivity, reutilizando su primer log mono sin medirlo
otra vez. La matriz final se identifica con hashes propios.

Los 75 excesos finales y sus ratios se registran caso por caso. Son presupuestos
nominales del motor/cronómetro/scheduler, principalmente para bloques de un frame
(20833 ns a 48 kHz), no fallos de callbacks físicos. Los 24 casos finales con
rechazos PoolFull se contrastan con el oráculo. Ni estos tiempos ni los anteriores
de sampler/mezclador/secuenciador demuestran rendimiento nativo del launcher.

## Evidencia y continuidad

Logs/binarios/matrices en `target/stage13-4a-20261116/`: `release-offline.log`,
`release-native-feature.log`, `check.log`, `desktop-build.log`,
`launcher-offline.exe`, `launcher-native.exe`, `bench.json` y `bench-final.json`.
El [manifiesto](evidencia-diagnosticos-clips-13.4a.json) conserva fuentes acumuladas,
hashes de binarios/logs, ambas matrices y enlaces revisados.

Solo cambia CLI/diagnósticos/pruebas: seis fuentes, dos nuevas; main.rs amplía la
huella heredada. Dominio, motor, servicio, aplicación, desktop, dependencias,
lockfiles, bindings e imágenes conservan sus hashes de la etapa anterior.
No hay commit/push ni CI nueva. Los manifiestos previos son históricos.

Siguiente: **13.4.b, matriz nativa release y escucha en dispositivo identificado**,
según el protocolo. Después: 13.4.c, entrega/CI y cierre. Grabación, MIDI, plugins,
timeline, piano roll, automatización y resampling/time stretching no entran aquí.
