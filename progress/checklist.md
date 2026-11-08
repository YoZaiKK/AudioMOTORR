# Etapas de implementación

Fuente de requisitos: [documento original](rust-daw-bootstrap-prompt.md).
Registro de decisiones, comandos y limitaciones: [detalles](detalles.md).

Estado actual (2026-11-08): etapas 1–7 completadas; etapa 8 implementada localmente
con mediciones release sobre Realtek. Ver [reporte de etapa 8](etapa-8.md) para
checks, build, smoke y estado de publicación/CI del incremento. Etapa 7 está
publicada en c6cf627 con CI Ubuntu/Windows correcta; [cierre](cierre-etapa-7.md).
Las etapas 9–13 siguen planificadas. Se conservan los reportes históricos.
Plan vigente: [etapas 7–13](plan-etapas-7-13.md).
Registro actual: [continuidad tras etapa 6](continuidad-etapa-6.md).
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
- [ ] Registrar publicación/CI de la revisión nueva por separado de pruebas locales.

No verificados: loopback acústico, unplug físico, estrés prolongado y plataformas
nativas Linux/macOS. La medición de muestras no prueba ausencia de dropouts.

## Etapa 9 — Sampler y voces

- [ ] Implementar sampler con pool fijo y política de agotamiento definida.
- [ ] Cargar/decodificar recursos fuera del callback y verificar propiedad/cierre.
- [ ] Cerrar con fixtures deterministas y regresiones de tiempo real.

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
