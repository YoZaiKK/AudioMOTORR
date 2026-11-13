# Etapa 11.3.d — Aceptación local del secuenciador

Fecha: 2026-11-12. Base Git `2d4cc285434383b451116925f4227219d4bc5171`, más
los cambios locales 11.3.b/c. **Aceptación integrada completada localmente.**
La etapa 11 sigue abierta hasta medición/escucha nativa y entrega/CI en 11.4.

## Entrega

Se añade un archivo de pruebas Rust con dos casos y 42 combinaciones:
24 de PCM decodificado en el motor y 18 de documento/preparación/aplicación/servicio.
La [matriz de aceptación](../docs/sequencer-integration-acceptance.md) vincula los
criterios con pruebas, capas y límites. La revisión no encontró un defecto de
producción que requiriera cambios: las 152 fuentes anteriores conservan sus hashes.
La nueva huella incorpora el archivo de pruebas como fuente 153.

Cuatro WAV reales PCM16/float32, con fuentes mono/estéreo, se preparan mediante
el coordinador y la aplicación asíncrona. Se eliminan antes de instalar/renderizar
para comprobar que se usa el PCM retenido. Las ediciones después de preparar
instalan el patrón y mezcla actuales; el BPM seleccionado sobrevive al cierre
de una sesión anterior incluso sin que esa sesión procesara su comando.

El beat tiene 18 notas por compás y se procesa hasta 257 frames después de la
repetición: 17 fronteras y 20 notas iniciadas. Salidas mono/estéreo a 48 kHz;
95.5, 127.25 y 300 BPM; bloques 1/127/256/4096 para PCM, y 127/256/4096 para
aplicación. Incluye coincidencias, colas solapadas, rutas permutadas, gain/pan,
mute con voces activas y rampa master de 5 ms hasta 0.5.

La referencia calcula las fronteras por fórmula racional absoluta, mantiene su
propio pool de voces y evalúa la señal por edad de cada sample. No llama al
planificador ni al iterador de eventos. Contrasta cada muestra y picos de toda
la ventana con tolerancia `1e-7`; fase/eventos/contadores/rutas son exactos.
La aplicación contrasta snapshots, medidores, voces, reloj, silencio y watermark.
Las notas programadas no alteran los informes ni contadores de disparo manual.
El API de aplicación no expone PCM; la comparación por muestra se hace en el
motor con el banco decodificado y se enlaza con la sesión mediante observaciones.

## Auditoría de cobertura y regresiones

El check actual vuelve a ejecutar las suites existentes de aplicación, servicio,
motor, IPC y UI. Cubren edición en frontera pendiente, Pause/Play/Stop, selección
frente a aplicación, FIFO/cola llena, timeout inconcluso, errores de preparación,
cancelación, slots vacíos/pool lleno, pérdida y orden de snapshots, reapertura,
precisión u64/u128, permisos y modos navegador/legacy. El contrato RT vuelve a
comprobar memoria y destrucción fuera del procesamiento.

La UI aporta 54 casos totales y el IPC siete. Son capas complementarias; no se
atribuye reproducción física a los mocks. La inspección real WebView2 de 11.3.c
se conserva como captura histórica de edición/layout con audio sin inicializar.
Se contrastan sus fuentes con las actuales: no cambia la producción y no se
repite la apertura visual en este incremento.

## Verificación

| Comprobación                                                                                                          | Resultado                                                                                                                                                           |
| --------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `npm run check`                                                                                                       | Código 0, 23:22:21–23:23:42 UTC. Bindings, tipos, lint, 54 casos frontend, formato/build web, nueve escenarios CLI, Rust headless/workspace/instrumentado y Clippy. |
| `cargo test -p daw-app --release --locked --test sequencer_acceptance_contract --test sequencer_application_contract` | Código 0, 23:21:59–23:22:08 UTC: dos casos nuevos y nueve existentes.                                                                                               |
| `npm run desktop:build -- --debug --no-bundle`                                                                        | Código 0, 23:23:56–23:24:08 UTC: ejecutable Windows debug con assets, sin instalador.                                                                               |

Los dos casos nuevos también pasaron en debug y se incluyen en el check general;
no se suman ejecuciones repetidas como casos distintos. Logs y metadatos locales
en `target/stage11-3d-20261112/`. El
[manifiesto](evidencia-integracion-secuenciador-11.3d.json) registra hashes,
comandos, resultados y vínculo al manifiesto anterior, sin reescribir evidencia histórica.

## Siguiente paso

**11.4.a: diagnósticos explícitos del secuenciador y protocolo nativo.**
Añadir rutas CLI offline/nativa sobre las APIs actuales, con patrón/BPM definidos,
fases/eventos/rechazos identificables y medidas de procesamiento. Ejecutar matriz
offline release, documentar comandos y preparar el runner/fixture para 11.4.b.
Los diagnósticos actuales de sampler/mezclador inicializan otros modos y no
sirven como mediciones del secuenciador.

11.4.b realizará matriz física, controles/reaperturas y escucha identificada;
11.4.c consolidará auditoría, entrega y CI del SHA correspondiente antes del
cierre. Persistencia y exportación de beats siguen en 12; clips/escenas en 13.
Este incremento no abre audio físico ni aporta nueva escucha, commit/push o CI
hospedada. Loopback, desconexión física, estrés prolongado y otras plataformas
conservan sus límites de verificación previos.
