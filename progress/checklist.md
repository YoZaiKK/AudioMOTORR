# Etapas de implementación

Fuente de requisitos: [documento original](rust-daw-bootstrap-prompt.md).
Registro de decisiones, comandos y limitaciones: [detalles](detalles.md).

Estas son las seis etapas acordadas en el análisis. No equivalen a la numeración
de las once secciones del documento original. Una casilla se marca únicamente
cuando existe implementación o evidencia de la verificación indicada.

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

- [ ] Etapa terminada.
- [ ] Crear comandos IPC y errores estructurados.
- [ ] Generar DTO TypeScript desde Rust y centralizar `invoke`/`listen`.
- [ ] Crear panel de audio, transporte, BPM y estado técnico.
- [ ] Reflejar estado aplicado con eventos de hasta 15 Hz.
- [ ] Limpiar suscripciones y tareas; contemplar StrictMode y cierre.
- [ ] Verificar capabilities, CSP y modo navegador sin funciones nativas.

## Etapa 5 — Verificar el funcionamiento

- [ ] Etapa terminada.
- [ ] Completar pruebas de contratos del motor, colas y lifecycle.
- [ ] Verificar errores de inicialización desde React.
- [ ] Ejecutar comprobaciones frontend y Rust, con y sin audio nativo.
- [ ] Ejecutar CLI `smoke` con varios tamaños de bloque.
- [ ] Comprobar regeneración de bindings y lockfiles.
- [ ] Compilar desktop y registrar smoke real de ventana/audio si está disponible.

## Etapa 6 — Documentar y automatizar

- [ ] Etapa terminada.
- [ ] Completar documentación de arquitectura, tiempo real y decisiones.
- [ ] Completar instrucciones de Windows y limitaciones de otras plataformas.
- [ ] Crear CI headless Linux y frontend/desktop Windows.
- [ ] Documentar pruebas que requieren hardware.
- [ ] Registrar roadmap posterior: oscilador, sampler, mezcla, pasos y clips.
- [ ] Revisar los criterios finales contra el documento original.
