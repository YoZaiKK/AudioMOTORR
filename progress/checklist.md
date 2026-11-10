# Etapas de implementación

Fuente de requisitos: [documento original](rust-daw-bootstrap-prompt.md).
Registro de decisiones, comandos y limitaciones: [detalles](detalles.md).

Estado actual (2026-11-10): etapas 1–8 completadas; etapa 8 publicada en 1e6bca8,
con mediciones release sobre Realtek y CI Linux/Windows correcta en run 42524752652.
Ver [reporte de etapa 8](etapa-8.md) para checks, build, smoke y límites. Etapa 7 está
publicada en c6cf627 con CI Ubuntu/Windows correcta; [cierre](cierre-etapa-7.md).
Etapa 9 en curso: 9.1–9.2 completadas; implementación 7acc1db con ambos jobs CI
correctos en run 42530699397. Se dispone de PCM/fixtures, banco fijo y voces raw;
preparación cerrada en 9.3 y sesiones manual/CPAL en 9.4.b verificadas sin hardware;
app/IPC/UI implementados en 9.4.c con mocks; integración auditada localmente en
9.4.d. Evidencia física/mediciones pendientes en 9.5.
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
pendientes escucha y consolidación 9.5.c.
Ver [reporte actual](etapa-9-diagnosticos-sampler.md).
Las etapas 10–13 siguen planificadas. Se conservan los reportes históricos.
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
- [ ] Publicar/inspeccionar CI de revisión exacta de 9.3 cuando se solicite; no inferir éxito remoto.
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
- [ ] 9.5.b: incorporar confirmación de escucha humana; no inferir sonido físico/continuidad desde probes.
- [ ] 9.5.c: cierre con regresiones, límites y CI del SHA publicado.

## Etapa 10 — Mezclador de cuatro canales

- [ ] Implementar gain/pan/mute y política de nivel de salida con pruebas.
- [ ] Acumular medidores en motor y publicar telemetría acotada.
- [ ] Integrar documento/aplicación/UI y validar cuatro canales con fixtures.

## Etapa 11 — Secuenciador de 16 pasos

- [ ] Modelar patrones y programar eventos a offsets dentro del bloque.
- [ ] Definir redondeo/drift para tempos con frames fraccionales por paso.
- [ ] Probar particiones, BPM y transporte sin duplicar/perder eventos de límite.

## Etapa 12 — Persistencia y exportación de beat

- [ ] Versionar documento y resolver migraciones/recursos relativos fuera del callback.
- [ ] Implementar guardar/abrir y errores de versión, rutas y assets ausentes.
- [ ] Verificar round-trip y exportación consistente de un beat completo.

## Etapa 13 — Clips y escenas

- [ ] Definir modelo, lanzamiento, cuantización y transición de clips/escenas.
- [ ] Implementar coordinación y controles manteniendo documento/reproducción separados.
- [ ] Probar cambios con distintos bloques/tempos antes de planificar timeline/piano roll/automatización.
