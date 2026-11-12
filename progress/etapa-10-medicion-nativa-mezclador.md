# Etapa 10.4.b — Medición nativa del mezclador

Fecha: 2026-11-11, hora de México. **Seis casos nativos correctos, con seis
reaperturas. Confirmación auditiva del usuario pendiente; etapa 10 sigue abierta.**
La medición continúa los [diagnósticos 10.4.a](etapa-10-diagnosticos-mezclador.md)
y el [protocolo](../docs/mixer-native-verification.md), sobre sus mismas fuentes.

## Dispositivo, entradas y alcance

Enumeración actual: `Speakers (2- Realtek(R) Audio)`, WASAPI, ID
`wasapi:{0.0.0.00000000}.{7ca1b2e0-ec85-4d31-bc07-f036d5ce34a3}`.
La salida Headphones usada en etapa 9 no apareció en esta enumeración. Se eligió
el ID disponible explícitamente y se verificó al abrir/reabrir; sin fallback.
Configuración solicitada y observada: 48000 Hz, dos canales, f32.

Se crearon dos fixtures nuevos float32 de diez segundos: seno mono 997 Hz a
pico 0.1 y estéreo con lados 997/440 Hz a picos 0.1/0.05. Se conservaron sus hashes
antes/después. El runner carga cada entrada en cuatro slots independientes:
7.680.000 bytes PCM para la entrada mono y 15.360.000 para la estéreo.

Se ejecutó `mixer-native-check`, binario release con `rt-diagnostics`, para cada
entrada y 0/1/16 voces iniciales, con ventana de un segundo. Todos los casos
incluyen después controles de cuatro canales, captura de rutas, mute sin destruir
voces, pausa/cambios/reanudación, Stop, final natural y reapertura en el mismo
dispositivo. El caso inicial de cero voces también genera sonido después.

## Resultados

| Entrada | Voces iniciales | Resultado | Reapertura |
| ------- | --------------- | --------- | ---------- |
| Mono    | 0               | Correcto  | Correcta   |
| Mono    | 1               | Correcto  | Correcta   |
| Mono    | 16              | Correcto  | Correcta   |
| Estéreo | 0               | Correcto  | Correcta   |
| Estéreo | 1               | Correcto  | Correcta   |
| Estéreo | 16              | Correcto  | Correcta   |

En las **doce sesiones**: callbacks/frames y señal observados; cero asignaciones,
bytes asignados, liberaciones, bytes liberados, muestras no finitas y deadline
misses medidos. Máximo global de procesamiento observado: **206300 ns (206,3 µs)**.
Ratio máximo contra presupuesto nominal: **0.02063 (2,063 %)**. Tamaños de callback
medidos: **480–1056 frames**. Cada proceso terminó correctamente y cerró el audio
a Uninitialized.

Son métricas acumuladas del lifecycle completo; no son percentiles aislados de
las ventanas 0/1/16. Las cotas p50/p99 del histograma y todos los campos brutos se
conservan por sesión. Tiempos/memoria se publican con retraso de un callback;
los probes de muestras incluyen el callback actual. La publicación final ausente
no se interpreta como cero. Se excluyen scheduling/driver/otros hilos y no se
afirma una garantía de tiempo real o continuidad acústica.

## Evidencia y revisión

[Manifiesto nativo](evidencia-nativa-mezclador-10.4b.json): seis invocaciones,
fechas/exit status, doce registros de métricas, dispositivo/configuración,
metadatos de sistema/CPU, inputs, binario, logs y huella de 136 archivos.
Artefactos locales: `target/stage10-4b-20261111/`. Fuente y ejecutable son los
verificados en 10.4.a; no hubo correcciones de código entre check/build y esta
medición. Los manifiestos anteriores se conservan como históricos.

Se preguntó al usuario si escuchó las pruebas por Speakers Realtek. **Todavía no
hay respuesta registrada**: la confirmación de audífonos de etapa 9 no se atribuye
a este mezclador/dispositivo. No se repite audio automáticamente para sustituir
una confirmación pendiente.

## Pendiente para cerrar

- Registrar la confirmación auditiva real; comparación manual por oído de pan/mute
  sigue pendiente y no se deduce de coeficientes o medidores.
- Consolidar auditoría/evidencia y CI del SHA entregado en **10.4.c**. El código
  sigue local, sin commit/push ni CI propios; la CI de `4dc2c6c` es histórica.
- Mantener como límites loopback/continuidad acústica, desconexión física,
  estrés prolongado, salida física mono, otros formatos/rates/dispositivos y
  Linux/macOS nativos no medidos.
