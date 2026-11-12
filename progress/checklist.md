# Etapas de implementación

Fuente de requisitos: [documento original](rust-daw-bootstrap-prompt.md).
Registro de decisiones, comandos y limitaciones: [detalles](detalles.md).

Estado actual (2026-11-12): etapas 1–10 completadas dentro de su alcance;
11.1 y 11.2 implementados localmente; ver [modelo de patrón](etapa-11-modelo-patron.md)
y [planificador](etapa-11-planificador.md). Siguiente: servicio/aplicación/IPC/UI 11.3.
ver [cierre del sampler](cierre-etapa-9.md). Etapa 8 publicada en 1e6bca8,
con mediciones release sobre Realtek y CI Linux/Windows correcta en run 42524752652.
Ver [reporte de etapa 8](etapa-8.md) para checks, build, smoke y límites. Etapa 7 está
publicada en c6cf627 con CI Ubuntu/Windows correcta; [cierre](cierre-etapa-7.md).
Historial de etapa 9: 9.1–9.2 completadas; implementación 7acc1db con ambos jobs CI
correctos en run 42530699397. Se dispone de PCM/fixtures, banco fijo y voces raw;
preparación cerrada en 9.3 y sesiones manual/CPAL en 9.4.b verificadas sin hardware;
app/IPC/UI implementados en 9.4.c con mocks; integración auditada localmente en
9.4.d. Mediciones físicas correctas en 9.5.b; audibilidad confirmada por el usuario
tras la última repetición con audífonos.
Ver [reporte del kernel](etapa-9-voces.md).
9.3.a fija el contrato; 9.3.b implementa lectura/decodificación de recursos
individuales; 9.3.c implementa banco completo/permisos/presupuesto agregado.
9.3.d verifica fallos/cancelación/cierre y propiedad entre hilos.
9.3.e verifica fixtures WAV → banco → Sampler offline y matriz de aceptación.
9.3.f cierra preparación localmente. 9.4.a integra sampler en AudioEngine con
transporte/headroom/rampas, verificado offline. 9.4.b conecta sesiones/outcomes
en audio. 9.4.c conecta Application/DTO/IPC/UI con mocks; 9.4.d audita/cierra
integración local y corrige recuperación/polling. 9.5.a implementa diagnósticos
sampler y benchmark offline sin CPAL. Seis mediciones 9.5.b correctas;
escucha confirmada. Auditoría de código/evidencia y CI de `4dc2c6c` correctas;
ver [auditoría 9.5.c](auditoria-cierre-9.5c.md) y cierre final de etapa 9.
Ver [reporte actual](etapa-9-diagnosticos-sampler.md).
Contrato 10.1, núcleo 10.2 y servicio 10.3.a implementados localmente;
coordinación de aplicación 10.3.b también implementada localmente; desktop 10.3.c implementado localmente; aceptación 10.3.d completada; diagnósticos 10.4.a listos; matriz nativa 10.4.b medida; CI propia y escucha confirmada; [etapa 10 cerrada](cierre-etapa-10.md). Las etapas 11–13
siguen sin implementación completa. Los apartados anteriores conservan el
historial de cada incremento; el cierre enlazado arriba registra el estado actual.
Plan vigente: [etapas 7–13](plan-etapas-7-13.md).
Registro actual: [continuidad del proyecto](continuidad-etapa-6.md).
Auditoría de objetivos, evidencia y plan detallado de 9.3:
[estado y siguiente etapa](estado-y-siguiente-etapa.md).
Git identifica las revisiones publicadas; consultar [Actions](https://github.com/YoZaiKK/AudioMOTORR/actions/workflows/ci.yml)
para el resultado remoto. CI creada y validada localmente no implica CI remota satisfactoria.

Las etapas 1–6 corresponden al fundamento original; las 7–13 continúan con los
siete hitos musicales en el orden acordado. No equivalen a las once secciones
del documento original. Una casilla se marca únicamente cuando existe
implementación o evidencia de la verificación indicada. Planificar no equivale
a implementar ni cerrar una etapa.

## Etapa 1 — Preparar el proyecto

- [x] Etapa terminada y compilada en Windows (2026-11-05).
- [x] Inspeccionar el repositorio y conservar el documento original.
- [x] Renombrar `prompt/` a `progress/` y crear el seguimiento.
- [x] Consultar y fijar versiones estables compatibles de Rust y frontend.
- [x] Configurar React + TypeScript + Vite en la raíz.
- [x] Inicializar Tauri 2 en `src-tauri/`.
- [x] Crear el workspace Cargo virtual con edition 2024 y resolver 3.
- [x] Generar un único lockfile de npm y uno de Cargo en la raíz.
- [x] Añadir comandos de desarrollo, build, formato, lint y diagnóstico.
- [x] Verificar instalación reproducible y compilación inicial.
- [x] Actualizar README y registrar resultados y limitaciones.

## Etapa 2 — Construir el motor

- [x] Etapa terminada y verificada en Windows (2026-11-05).
- [x] Crear `daw-domain` con documento mínimo, IDs y tipos validados.
- [x] Crear `daw-engine` con configuración y render de silencio.
- [x] Separar contador del dispositivo y posición musical.
- [x] Implementar Play, Pause, Stop y validación de BPM.
- [x] Comprobar comportamiento con diferentes particiones de bloques.

## Etapa 3 — Conectar el audio

- [x] Etapa terminada y verificada en Windows con audio real (2026-11-06).
- [x] Crear `daw-audio` con backend manual determinista.
- [x] Implementar CPAL opcional mediante `native-audio`.
- [x] Resolver dispositivos, canales y formatos negociados.
- [x] Implementar hilo de control y colas SPSC acotadas.
- [x] Distinguir comandos aceptados y aplicados, y publicar snapshots.
- [x] Implementar errores, inicialización, cierre y reinicio.
- [x] Integrar los casos de uso en `daw-app` y crear la CLI `smoke`.

## Etapa 4 — Conectar la interfaz

Estado: completada y verificada (2026-11-06), incluido smoke real de ventana y audio.

- [x] Etapa terminada y verificada en Windows (2026-11-06).
- [x] Crear comandos IPC y errores estructurados.
- [x] Generar DTO TypeScript desde Rust y centralizar `invoke`/`listen`.
- [x] Crear panel de audio, transporte, BPM y estado técnico.
- [x] Reflejar estado aplicado con eventos de hasta 15 Hz.
- [x] Limpiar suscripciones y tareas; contemplar StrictMode y cierre.
- [x] Verificar capabilities, CSP y modo navegador sin funciones nativas.

## Etapa 5 — Verificar el funcionamiento

- [x] Etapa terminada y verificada en Windows (2026-11-06).
- [x] Completar pruebas de contratos del motor, colas y lifecycle.
- [x] Verificar errores de inicialización desde React.
- [x] Ejecutar comprobaciones frontend y Rust, con y sin audio nativo.
- [x] Ejecutar CLI `smoke` con varios tamaños de bloque.
- [x] Comprobar regeneración de bindings y lockfiles.
- [x] Compilar desktop y registrar smoke real de ventana/audio si está disponible.
- [x] Crear howToRunThis en español con escenarios ejecutables y siete capturas reales.

## Etapa 6 — Documentar y automatizar

- [x] Etapa terminada localmente (2026-11-06); ver [cierre y evidencia](cierre-etapa-6.md).
- [x] Completar documentación de arquitectura, tiempo real y decisiones.
- [x] Completar instrucciones de Windows y limitaciones de otras plataformas.
- [x] Crear CI headless Linux y frontend/desktop Windows; validar con actionlint.
- [x] Documentar pruebas que requieren hardware y conservar evidencia anterior.
- [x] Registrar roadmap posterior: oscilador offline/WAV, nativo/rampas, sampler, mezcla, pasos, persistencia/exportación y clips.
- [x] Revisar los criterios finales contra el documento original.
- [x] Ejecutar check completo y después build Windows; comprobar lockfiles/bindings intactos.

Base CI 1ef9bd0 confirmada: Ubuntu headless y Windows desktop correctos.
Pendientes al cierre histórico de etapa 6: CI de cambios nuevos, Linux nativo/macOS,
desconexión física, otros formatos de hardware y mediciones de allocator/deadlines
y continuidad en release con señal. El estado actual de señal/mediciones está
en el [reporte de etapa 8](etapa-8.md).

## Plan posterior — definido el 2026-11-07

- [x] Definir etapas 7–13, responsabilidades, dependencias y aceptación.
- [x] Desglosar etapa 7 y fijar contrato propuesto de señal/transporte/WAV.
- [x] Enlazar plan, roadmap, README y continuidad sin anunciar funciones inexistentes.

## Etapa 7 — Oscilador offline y exportación WAV

Estado: completada el 2026-11-07; implementación c6cf627, CI remota correcta.
Detalle en [plan](plan-etapas-7-13.md) y [cierre](cierre-etapa-7.md).

- [x] 7.0: inspeccionar jobs de CI de 1ef9bd0; ambos success, evidencia en reporte.
- [x] 7.1: concretar API/políticas; hound 3.5.1 elegido y fijado en CLI durante 7.3.
- [x] 7.2: implementar oscilador opt-in en daw-engine con fase/transporte y pruebas.
- [x] Conservar modo de silencio por defecto y verificar particiones 1/127/256/4096.
- [x] 7.3: coordinar render acotado en daw-app y WAV float32 en CLI, sin hardware.
- [x] Cubrir duración exacta/cero, metadatos, muestras, destino existente y errores de I/O.
- [x] 7.4 local: cobertura WAV descubierta por check/CI, documentación real y checks/build correctos.
- [x] Confirmar CI remota de c6cf627: run 42419963899, ambos jobs success.
- [x] 7.5: registrar evidencia local/remota y cerrar la etapa según aceptación.

## Etapa 8 — Oscilador nativo, rampas y mediciones

- [x] Reutilizar oscilador por CPAL y exponer controles explícitos mediante aplicación/IPC/UI.
- [x] Probar rampas/transiciones y conservar lifecycle/errores.
- [x] Medir allocator, tiempos, carga y continuidad de muestras en release; registrar hardware y límites.
- [x] Registrar check completo, build Windows y smoke UI del incremento.
- [x] Registrar publicación/CI de 1e6bca8 por separado: run 42524752652, ambos jobs success.

No verificados: loopback acústico, unplug físico, estrés prolongado y plataformas
nativas Linux/macOS. La medición de muestras no prueba ausencia de dropouts.

## Etapa 9 — Sampler y voces

- [x] 9.1: fijar formato, límites, propiedad y política de 16 voces/agotamiento.
- [x] Implementar SampleBuffer inmutable, validación y compatibilidad de salida.
- [x] Añadir fixtures en memoria y cuatro pruebas de recursos (sin reproducción).
- [x] Registrar check completo, build Windows, formato/enlaces y hashes de 9.1.
- [x] 9.2: implementar banco fijo y kernel de voces con oráculos de muestras/particiones.
- [x] Implementar kernel raw con pool fijo y rechazo del nuevo disparo al agotarse.
- [x] Medir cero asignaciones/liberaciones en procesamiento, con control positivo de destrucción.
- [x] Registrar check completo/build Windows y reproducibilidad del incremento 9.2.
- [x] Publicar 7acc1db y verificar CI Linux/Windows del SHA exacto: run 42530699397.
- [x] 9.3 local: cargar/decodificar recursos fuera del callback y verificar propiedad/cierre; publicación/CI propia pendientes.
- [x] 9.3.a: fijar formatos, 16 MiB/archivo, presupuestos de preparación/coexistencia y cancelación; aplicados en 9.3.b–c.
- [x] 9.3.b: lector/preflight y decoder de recurso individual, con fixtures y medición de reservas.
- [x] Registrar check completo/build y reproducibilidad del lector 9.3.b.
- [x] 9.3.c: banco completo, permiso compartido hasta entrega/descarte y presupuesto agregado.
- [x] Registrar check completo/build y reproducibilidad del coordinador 9.3.c.
- [x] 9.3.d: matriz de fallos/cancelación, cierre en preparación e interleavings de entrega; memoria entre hilos.
- [x] Registrar check completo/build y reproducibilidad de 9.3.d.
- [x] 9.3.e: fixtures WAV → banco → Sampler offline y matriz de aceptación de formatos/límites.
- [x] Registrar check completo/build y reproducibilidad de 9.3.e.
- [x] 9.3.f: auditar aceptación y cerrar preparación local; checks/build 9.3.e y manifiesto de código/configuración.
- [x] Preparación 9.3 incluida en 4dc2c6c; CI Linux/Windows del SHA inspeccionada en run 42896571891 el 2026-11-11.
- [x] 9.4: integración local de transporte/headroom, outcomes del servicio e IPC/UI; aceptación física pendiente en 9.5.
- [x] 9.4.a: AudioEngine con sampler/headroom 1/16, rampas y transporte, resultados directos Copy y pruebas sin dispositivo.
- [x] Registrar check completo/build y evidencia de 9.4.a.
- [x] 9.4.b: sesiones manual/CPAL, propiedad/cierre del banco y outcomes acotados en servicio; hardware no abierto.
- [x] Registrar check completo/build y evidencia de 9.4.b.
- [x] 9.4.c: aplicación, worker de preparación/cancelación, DTO generados, IPC/ACL y controles UI con pruebas mock.
- [x] Registrar check completo/build Windows y manifiesto de 116 archivos de 9.4.c; hardware y CI nueva pendientes.
- [x] 9.4.d: auditoría por criterio, regresiones de enumeración/polling y protocolo 9.5; cierre local de integración.
- [x] Registrar check/build Windows de 9.4.d, 24 pruebas frontend y manifiesto de 116 archivos; sin CI remota ni hardware nuevos.
- [x] 9.5.a: sampler-bench y sampler-native-check opt-in, feature offline sin CPAL, contadores de asignaciones/liberaciones, probes todos los canales e histograma fijo; tests sin hardware.
- [x] Registrar check/build Windows, release focal/CLI nativa compilada, benchmarks offline de 24 casos y manifiesto de 119 archivos; sin medición física nueva.
- [x] 9.5.b: seis mediciones release Realtek/WASAPI, entradas mono/estéreo y 0/1/16 voces, IDs, cierre/reapertura y evidencia.
- [x] 9.5.b: registrar «Si se escucha» tras el último par mono/estéreo por Headphones Realtek; audibilidad confirmada, continuidad acústica sin medir.
- [x] 9.5.c: auditar huella de 120 archivos, logs/entradas/binario y CI Linux/Windows de 4dc2c6c; conservar límites e historial.
- [x] 9.5.c: cierre con regresiones/check/build de implementación, límites, CI de 4dc2c6c y escucha registrada en cierre-etapa-9.md.

## Etapa 10 — Mezclador de cuatro canales

- [x] 10.1: contrato de enrutamiento/ganancia/pan/mute, headroom, rampas, medidores y fixtures especificado en docs/mixer-contract.md.
- [x] 10.2: estado de dominio y mezcla opt-in, rutas capturadas, gain/pan/mute, rampas y política de salida con fixtures.
- [x] Verificar particiones y cero asignaciones/liberaciones de procesamiento; control positivo de destrucción PCM fuera del render.
- [x] Acumular picos de canal/master en ventanas de render con secuencia/frames, mono explícito y overflow sin mutación.
- [x] 10.3.a: sesiones mezcladas, controles por FIFO y agrupación de medidores por callback con observaciones Copy acotadas.
- [x] 10.3.a: verificar cola/IDs, pérdida de observaciones, chunks/conversiones, errores/reinicio, memoria y PCM en propietario.
- [x] 10.3.b: coordinar documento/aplicación con instalación de ajustes actuales, recibos offline/en vivo y edición posterior a aceptación.
- [x] 10.3.b: validar cola llena/errores sin mutación, preparación/cancelación, banco consumido, reapertura y ausencia de replay/auto Play.
- [x] 10.3.c: integrar DTO/IPC/ACL/UI y validar cuatro tiras de mezcla con fixtures.
- [x] 10.3.c: distinguir selección/recibos/targets/rampas, pérdida de sesión, errores, mono/estéreo y modo navegador.
- [x] 10.3.d: auditar aceptación integrada local y evidencia final de mezcla.
- [x] 10.3.d: contrastar cuatro WAV/canales y ciclo completo con particiones, pool/mono/estéreo/cancelación; corregir consulta inicial obsoleta y recibo offline.
- [x] 10.4.a: comandos mixer-bench/mixer-native-check y protocolo; release/gates, referencia de cuatro canales, lifecycle manual y 24 benchmarks offline.
- [x] 10.4.b: seis casos mixed nativos y seis reaperturas Speakers Realtek, 48000 Hz estéreo f32, fuentes mono/estéreo y 0/1/16 voces; evidencia conservada.
- [x] 10.4.b: registrar «Se escucha» tras la repetición solicitada mono/estéreo, 16 voces y cinco segundos de ventana, por Speakers Realtek.
- [x] 10.4.c: auditoría local de 136 fuentes, 21 artefactos, doce sesiones/24 benchmarks y Git/CI histórica; entrega revisable preparada.
- [x] 10.4.c: entregar revisión e7f543f e inspeccionar Linux/Windows success en run 43147510037 del SHA exacto.
- [x] Cerrar etapa 10 con escucha registrada, evidencia y CI propias; límites explícitos en cierre-etapa-10.md.

## Etapa 11 — Secuenciador de 16 pasos

- [x] 11.1: contrato de patrón, resolución, eventos, bucle, transporte, BPM y ediciones.
- [x] 11.1: modelo fijo de cuatro filas × 16 pasos, selección de slots y pruebas de dominio.
- [x] 11.2: programar eventos a offsets dentro del bloque e integrarlos con mezcla.
- [x] Redondeo hacia arriba y fase entera/fraccional exacta del BPM seleccionado.
- [x] Referencias racional/PCM, particiones, BPM, bucles, transporte, ediciones y errores atómicos.
- [x] 11.2: cero asignaciones/liberaciones instrumentadas y liberación PCM fuera del procesamiento.
- [ ] 11.3: servicio/aplicación/IPC y cuadrícula con posición observada.
- [ ] 11.4: referencia integrada, diagnósticos, verificación nativa y CI de la entrega.

## Etapa 12 — Persistencia y exportación de beat

- [ ] Versionar documento y resolver migraciones/recursos relativos fuera del callback.
- [ ] Implementar guardar/abrir y errores de versión, rutas y assets ausentes.
- [ ] Verificar round-trip y exportación consistente de un beat completo.

## Etapa 13 — Clips y escenas

- [ ] Definir modelo, lanzamiento, cuantización y transición de clips/escenas.
- [ ] Implementar coordinación y controles manteniendo documento/reproducción separados.
- [ ] Probar cambios con distintos bloques/tempos antes de planificar timeline/piano roll/automatización.
