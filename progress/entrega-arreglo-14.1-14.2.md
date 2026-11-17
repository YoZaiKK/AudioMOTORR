# Entrega local — Arreglo 14.1 y motor 14.2

Fecha: 2026-11-16 (México). El usuario solicitó crear el commit con todos los
cambios y la documentación. Esta entrega reúne ambos incrementos sobre
`f5f7ded8f401e2fc9caf4a2cced585060190e6c7`.

## Contenido del commit

- Modelo de arreglo en `daw-domain`: cuatro filas, 32 regiones por fila,
  hasta 4096 compases, referencias validadas y copias de contenido propias.
- Reproducción explícita en `daw-engine`: reloj exacto, regiones contiguas,
  huecos/repeticiones, transporte/tempo, final exclusivo y colas naturales.
- Pruebas de dominio, motor, referencias independientes y memoria; el check
  incorpora la prueba de memoria offline del arreglo.
- README, contrato, roadmap, planes, checklist, reportes y evidencia de 14.1–14.2.

Los manifiestos de [14.1](evidencia-modelo-arreglo-14.1.json) y
[14.2](evidencia-arreglo-motor-14.2.json) conservan su estado al capturarse, cuando
todavía no existía este commit. Sus hashes documentales son históricos; las
actualizaciones de esta entrega no los reescriben. Las once fuentes/scripts
verificados conservan los hashes registrados antes de preparar el commit.
Git normaliza los archivos de texto a LF según `.gitattributes`.

## Validación

Se reutiliza el check completo correcto de 14.2, ejecutado el 17 de noviembre
de 2026, 00:32:05–00:36:09 UTC (16 de noviembre en México): bindings, tipos,
ESLint, 72 pruebas frontend, formato, build web, nueve escenarios CLI, suites
Rust y Clippy. Motor: 91 pruebas correctas en debug/release, con 160 variantes
de referencia; dominio: 26 pruebas. Memoria instrumentada correcta en debug y
release en seis configuraciones. Las suites se solapan y no se suman.

La preparación del commit verifica de nuevo los hashes de las once fuentes,
la integridad de los logs, formato, enlaces locales y diferencias de Git. Solo
se actualiza documentación después de esas pruebas; no se modifica código ni
se repite una medición física. [Detalle del motor](etapa-14-arreglo-motor.md).

## Estado de entrega y siguiente trabajo

Este documento acompaña el commit local de código, pruebas y documentación.
No se ejecuta push ni se atribuye CI remota a esta revisión. Las mediciones y
CI del launcher conservan su revisión histórica; su escucha sigue pendiente.

La etapa 14 continúa: **14.3.a, sesiones y coordinación de aplicación**,
según el [plan](plan-etapa-14.md). El arreglo todavía no está integrado en
Project/archivos, exportación, backend/IPC ni controles desktop.
