# Etapa 14 — Arreglo de clips en línea de tiempo

Fecha: 2026-11-16 (México). Base revisada: `f5f7ded`; árbol limpio al iniciar.
El usuario volvió a pedir continuar con la siguiente etapa. Se avanza con el
siguiente hito del roadmap, timeline, conservando **la escucha de 13.4.b pendiente**
y la etapa 13 formalmente abierta. No se interpreta «Vale» como una audición ni
se vuelve a detener el desarrollo por esa observación.

## Secuencia y aceptación

| Paso   | Entrega                                                     | Estado / aceptación                                                                                                                                                                                                      |
| ------ | ----------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| 14.1   | Contrato y modelo de arreglo independiente en `daw-domain`. | Implementado localmente: cuatro filas, 32 regiones por fila, hasta 4096 compases, referencias a clips y snapshots propios; pruebas de límites, solapamientos, huecos y contenido. [Reporte](etapa-14-modelo-arreglo.md). |
| 14.2   | Reproducción determinista en daw-engine.                    | Implementada localmente: fin exclusivo/colas, transporte y tempo, observaciones, 160 variantes de referencia PCM/eventos, límites/overflow y memoria. [Reporte](etapa-14-arreglo-motor.md).                              |
| 14.3.a | Sesiones y coordinación de aplicación.                      | Siguiente: instalación/propiedad de snapshot fuera del callback, nuevo modo explícito, comandos acotados, observaciones y recuperación de lifecycle.                                                                     |
| 14.3.b | Proyecto versionado y exportación de arreglo.               | Integrar documento junto con esquema nuevo; migrar v1/v2 sin pérdidas, preservar assets/rutas, validar referencias y exportar con el mismo motor. No guardar una línea de tiempo bajo JSON v2.                           |
| 14.3.c | Editor desktop y posición observada.                        | Cuatro pistas con crear/mover/redimensionar/eliminar regiones y elección de clip; límites/errores visibles, IPC/DTO/ACL y pruebas; ninguna nota programada por UI.                                                       |
| 14.3.d | Aceptación integrada.                                       | WAV reales, proyectos portables y reapertura/exportación, referencias independientes y capturas reales del flujo.                                                                                                        |
| 14.4   | Diagnósticos, medición nativa y entrega.                    | Runner explícito de arreglo, memoria/tiempo, escucha registrada por separado, revisión entregada y CI de ese SHA; límites de plataforma transparentes.                                                                   |

14.1–14.2 preparan el documento y su reproducción en el motor. Los campos del
proyecto se integrarán junto con un esquema que pueda conservarlos. La
persistencia y UI llegarán después del planificador.
Cada incremento debe dejar check correcto y documentación de lo realizado; no se
atribuyen mediciones de clips/escenas al nuevo modo.

## Alcance musical

Primera versión: clips de un compás, colocados en compases completos en las cuatro
filas actuales. Una región repite su clip durante su longitud. No admite
solapamiento dentro de la misma fila; sí admite regiones contiguas y huecos.
El arreglo termina en el último fin exclusivo de región. Una copia resuelta
conserva contenido propio para separar edición y reproducción.

El [contrato](../docs/arrangement-contract.md) distingue el modelo implementado
de los requisitos futuros del motor y la integración. No se agregan dependencias
ni estructuras de plugins, grabación o MIDI a esta entrega.

Después de 14 siguen piano roll y automatización, en ese orden del roadmap;
sus límites y entregas se concretarán antes de implementarlos. Seeking, bucles,
mapas de tempo y captura de performance también quedan fuera de esta primera
línea de tiempo.
