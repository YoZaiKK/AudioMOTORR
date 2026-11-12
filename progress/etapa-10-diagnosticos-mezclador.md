# Etapa 10.4.a — Diagnósticos del mezclador y protocolo nativo

Fecha: 2026-11-11, hora de México. Base Git `4dc2c6c`, con mezcla y aceptación
local hasta 10.3.d preexistentes. **Herramientas implementadas localmente;
verificación física 10.4.b y cierre/CI 10.4.c pendientes. Etapa 10 abierta.**

## Entrega

- `mixer-bench`: camino mezclado explícito, cuatro assets/rutas, gain/pan/mute,
  voces 0/1/16, mono/estéreo y bloques 1/127/256/4096. Contrasta cada muestra y
  los picos de canal con un oráculo independiente, actividad y retriggers.
- Memoria y tiempos preasignados; cero asignaciones/liberaciones en el bucle
  medido y control positivo de cuatro liberaciones PCM en el propietario.
  Tiempos del procesamiento separados del oráculo; p50/p99 exactos ordenados,
  máximo, ratio nominal y overruns observacionales.
- `mixer-native-check`: carga el WAV en cuatro slots, exige dispositivo explícito
  y sesión mezclada nativa. Mide una ventana inicial 0/1/16, después comprueba
  controles de cuatro canales, coeficientes/rampas, captura de rutas, mute sin
  destruir voces, pausa/edición/reanudación, Stop, final natural y cierre/reapertura.
- Métricas instrumentadas de sesión y reapertura; resultados exactos de disparo,
  niveles de ventanas completas y trazas de cada fase. El caso inicial de cero
  voces también genera sonido en sus fases de control posteriores.
- Gates de release/features y opciones inválidas antes de I/O; native-disabled
  no abre el WAV. Los helpers compartidos de espera, opciones y validación de
  métricas conservan la lógica del sampler original; solo se amplía su visibilidad.
- [Contrato de diagnósticos](../docs/mixer-diagnostics.md) y
  [protocolo nativo](../docs/mixer-native-verification.md), con matriz de seis
  casos, datos a conservar y escucha separada de los resultados automáticos.

## Pruebas y mediciones locales

Cuatro casos nuevos únicos: matriz de benchmark; lifecycle manual con WAV real
en mono/estéreo y 0/1/16 voces; gate native-disabled; CLI release de mezcla.
Los dos últimos dependen de features/perfil. Se amplían las pruebas de help y
gates existentes. Las suites debug/release se solapan.

Se ejecutan **24 casos de benchmark offline**, 2000 iteraciones por caso, con
oráculo y memoria correctos. Son mediciones locales sin dispositivo, no tiempos
del callback nativo. El runner nativo compila y sus mismas fases pasan con
backend manual; no se invoca sobre hardware en este incremento.

Resultados, comandos, fechas, hashes de fuentes/logs y binarios release conservados
por separado en el [manifiesto 10.4.a](evidencia-diagnosticos-mezclador-10.4a.json).
Logs y ejecutables locales: `target/stage10-4a-20261111/`. Nueva huella de 136
archivos; seis cambian frente a 10.3.d, incluidos dos módulos nuevos de CLI.
Motor/servicio/aplicación/desktop, locks, permisos, bindings y workflow permanecen
sin cambios; el incremento se concentra en diagnósticos de CLI y sus pruebas.

| Verificación                                   | Resultado                                                                                                                                |
| ---------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------- |
| `npm run check`                                | Correcto: bindings/tipos/lint/formato, 39 casos frontend, cuatro IPC, nueve escenarios CLI, suites Rust y memoria instrumentada, Clippy. |
| Release CLI con `offline-diagnostics`          | Correcto: 11 unitarias y tres pruebas de proceso; incluye gates y matriz mezclada sin CPAL.                                              |
| Release CLI con `rt-diagnostics`               | Correcto: nueve unitarias y tres de proceso; soporte nativo compilado, lifecycle probado en manual.                                      |
| `mixer-bench`, mono/estéreo, 2000 iteraciones  | Correcto: 24 casos, muestras/picos/actividad/memoria y cuatro frees PCM de propietario por caso.                                         |
| `npm run desktop:build -- --debug --no-bundle` | Correcto: escritorio Windows debug, sin instalador.                                                                                      |

## Límites y siguiente paso

Las métricas nativas son acumuladas de todo el lifecycle, incluidas fases de
señal/control y final natural; no se etiquetan como percentiles de una ventana
aislada de voces. Sus percentiles son cotas superiores de histograma. Las fases
de control pueden perder observaciones; un resultado exacto perdido es inconcluso.
El polling tiene timeout, pero I/O del sistema y cierre de driver no son
interrumpibles. No hay prueba de continuidad acústica ni medición de toda la UI.

**Siguiente: 10.4.b**, preparar fixtures nuevos y ejecutar la matriz nativa
mezclada con dispositivo explícito, registrar mediciones y escucha real.
**10.4.c** consolidará evidencia y CI del SHA entregado antes de cerrar etapa 10.
Sin apertura de hardware, escucha, commit/push o CI hospedada nuevos en 10.4.a.
