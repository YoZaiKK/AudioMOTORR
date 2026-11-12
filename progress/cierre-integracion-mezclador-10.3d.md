# Etapa 10.3.d — Aceptación local de la mezcla

Fecha: 2026-11-11, hora de México. Base Git `4dc2c6c`, con implementación local
hasta 10.3.c preexistente. **Integración aceptada localmente; etapa 10 sigue
abierta hasta la verificación nativa y CI de 10.4.**

## Entrega y hallazgos

La [matriz de aceptación](../docs/mixer-integration-acceptance.md) vincula cada
requisito con código, fixtures y límites. Se añaden tres pruebas de aplicación
con WAV reales y tres regresiones de interfaz: seis casos únicos nuevos.

Las pruebas de UI reprodujeron dos defectos antes de corregirlos:

- Una consulta inicial lenta podía devolver ajustes anteriores después de una
  consulta explícita y una edición aceptada, sustituyendo la selección visible.
  La consulta inicial queda invalidada por una consulta explícita o un recibo
  aceptado; se ignoran tanto su respuesta como su error tardío.
- Un recibo offline seguía diciendo «audio sin inicializar» después de abrir
  una sesión mezclada. Ahora describe la actualización de selección sin comandos
  de audio y conserva la observación de la sesión como fuente del estado actual.

Las dos regresiones originales fallan antes de la corrección; una tercera cubre
el error tardío de consulta. Pasan junto con las regresiones previas.

## Referencia integrada

- Cuatro WAV float32, mono y estéreo con lados distintos, decodificados en un
  banco completo. Los archivos se eliminan antes de instalar/reproducir para
  demostrar que el audio usa recursos retenidos.
- Oráculos racionales independientes por canal y master, con ganancia de master
  0.5 y sin derivar los valores esperados de la implementación. Primer punto:
  picos master `(1/128, 1/64)`.
- Mismo calendario de ediciones y comprobaciones con particiones 1/127/256/4096:
  rutas capturadas, mute, pausa, cambios en silencio, reanudación, Stop, cierre
  y reapertura. Ventanas y secuencias corresponden a cada callback; no se exige
  el mismo número de callbacks entre particiones distintas.
- Capacidad global de 16 voces repartidas en cuatro canales, nivel completo
  con 16 voces unitarias y rechazo de la decimoséptima al procesar. Voces muteadas
  completan y recuperan capacidad, conservando el banco. Salidas mono/estéreo.
- Dos canales opuestos mantienen picos propios y se cancelan en el master;
  `render_manual` confirma silencio. No es una medición acústica.

Las nuevas pruebas de aplicación observan los picos reales calculados durante
render; el API no devuelve un buffer PCM. Las pruebas de motor existentes sí
contrastan muestras individuales, rampas y particiones.

## Verificaciones y evidencia

Resultados, comandos, fechas y hashes se registran en el
[manifiesto 10.3.d](evidencia-integracion-mezclador-10.3d.json). Logs locales en
`target/stage10-3d-20261111/`. La huella cubre 134 archivos de código/configuración:
cuatro cambian frente a 10.3.c, incluido el nuevo archivo de aceptación Rust.
Los manifiestos y capturas anteriores se conservan como evidencia histórica.

| Comprobación                                                                                      | Resultado                                                                                                                                         |
| ------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------- |
| `npm run check`                                                                                   | Correcto: bindings, tipos, lint, 39 pruebas frontend, formato, build web, nueve escenarios CLI, suites Rust headless/workspace, memoria y Clippy. |
| `cargo test -p daw-app --release --no-default-features --locked --test mixer_acceptance_contract` | Correcto: tres pruebas nuevas con referencias independientes.                                                                                     |
| `npm run desktop:build -- --debug --no-bundle`                                                    | Correcto: ejecutable de escritorio Windows debug, sin instalador.                                                                                 |

Las suites debug/release se solapan y no se suman como casos únicos. El
manifiesto añade la revisión final de formato, enlaces y hashes.

La producción Rust de domain/engine/audio/app, procesamiento, dependencias,
bindings, permisos, CLI y estilos se mantienen iguales a 10.3.c. Las únicas
correcciones de producción están en el hook de sesión y el texto del panel.
La instrumentación de memoria RT vuelve a ejecutarse con el check general;
no se atribuye cero asignaciones a aplicación/IPC/React.

## Siguiente paso y límites

**10.4: preparar diagnósticos explícitos del camino mezclado y un protocolo
nativo**, ejecutar medición/escucha, registrar evidencia y verificar CI del SHA
entregado antes de cerrar etapa 10. El runner actual `sampler-native-check`
inicializa el sampler original y `sampler-bench` mide ese camino; sus resultados
no prueban la mezcla nueva.

Sin nueva apertura de hardware, audición, commit/push o CI hospedada. La escucha
confirmada de etapa 9 se conserva como histórica. Desconexión física, loopback,
estrés prolongado y otras configuraciones/plataformas siguen pendientes.
No cambia la limitación de identificar un cierre/reinicio externo omitido por
completo sin una identidad pública de sesión. Secuenciador, persistencia y clips
mantienen su orden en 11–13.
