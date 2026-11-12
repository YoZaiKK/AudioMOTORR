# Auditoría de cierre del sampler — 9.5.c

Fecha: 2026-11-11. Revisión de implementación:
`4dc2c6c2e0882b8f131b68088feffa8def84ba83`. **Etapa 9 cerrada dentro del alcance
registrado**, tras confirmación de audibilidad por el usuario en la última
repetición con audífonos. La auditoría de código/evidencia y la inspección de CI
están completadas; ver [cierre final](cierre-etapa-9.md). Los apartados de pruebas
previas conservan sus observaciones y no se sustituyen por esta confirmación.

## Revisión publicada y CI

La API oficial de GitHub se consultó el 2026-11-11 a las 20:07:32 UTC.
La rama remota `main` apuntaba al SHA anterior, coincidente con HEAD y la
referencia local `origin/main`. El
[run 42896571891](https://github.com/YoZaiKK/AudioMOTORR/actions/runs/42896571891)
finalizó el 2026-11-10 con `completed/success` para ese SHA exacto:

| Job                                                                                                 | Resultado | Alcance inspeccionado                                                                                                 |
| --------------------------------------------------------------------------------------------------- | --------- | --------------------------------------------------------------------------------------------------------------------- |
| [headless-linux](https://github.com/YoZaiKK/AudioMOTORR/actions/runs/42896571891/job/130111130782)  | success   | Rustfmt, tests/Clippy de los cinco paquetes headless, nueve escenarios CLI y lockfiles.                               |
| [desktop-windows](https://github.com/YoZaiKK/AudioMOTORR/actions/runs/42896571891/job/130111131257) | success   | Prerrequisitos, regeneración determinista de bindings, check completo, build desktop debug y locks/tipos sin cambios. |

Se inspeccionaron los resultados de los pasos, sin descargar sus logs completos.
Esta CI verifica la implementación entregada, incluidas preparación, integración,
diagnósticos y trazas del runner. No ejecuta hardware ni confirma sonido físico.
Los cambios documentales de esta auditoría son posteriores y aún no tienen SHA/CI
propios. No se realizó commit/push durante esta tarea.

## Trazabilidad de las mediciones

Se contrastó el [manifiesto nativo 9.5.b](evidencia-nativa-9.5b.json) con los
archivos actuales y los artefactos locales conservados:

- Los **120 archivos** de código/configuración coinciden byte a byte con sus
  hashes del manifiesto. La huella agregada sigue siendo
  `102046d6e2625100aacde062c4ce19b7bf5341e6e6ad35b2dfddb8cd93fa04d2`.
- Al contrastarlos con los blobs de Git de `4dc2c6c`, 116 coinciden en bytes;
  cuatro coinciden tras normalizar CRLF a LF: `src-tauri/.gitignore`,
  `src-tauri/Cargo.toml`, `src-tauri/src/main.rs` y
  `src-tauri/src/snapshots.rs`. `git ls-files --eol` confirma CRLF/mixed en el
  worktree y LF en Git. No se modificaron esos archivos ni sus hashes históricos.
- Coinciden los hashes de los **seis logs nativos**, las **dos entradas WAV**,
  los **cuatro logs de check/build/release** y el **log del piloto**.
- El binario release local coincide con el usado en la matriz:
  `fa6aa8d1e54e067b4eb5eff57865d1d68b700ff4ae561c17742a7962e229f534`.

La revisión original detectó las cuatro diferencias de bytes Git/worktree y se
resolvió comprobando su equivalencia de finales de línea. La comprobación final
no deja diferencias de contenido. Artefactos de auditoría locales, no versionados:
`target/stage9-5c-audit-20261111/audit.mjs`, `integrity.json` (primera comparación)
y `integrity-verified.json` (comparación resuelta).
El [manifiesto de esta auditoría](evidencia-cierre-9.5c.json) resume resultados.

## Aceptación contrastada

| Criterio                                                     | Estado                                 | Evidencia / límite                                                                                                                                  |
| ------------------------------------------------------------ | -------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------- |
| Preparación WAV, presupuesto, cancelación y propiedad        | Implementado y verificado              | Contratos/fixtures 9.3, aceptación integrada 9.4 y check del SHA entregado.                                                                         |
| Sampler integrado con transporte, rampas y resultados por ID | Implementado y verificado              | Tests offline/adaptador/mock, seis logs nativos y CI de `4dc2c6c`.                                                                                  |
| Ventanas de 0/1/16 voces y ciclo de vida nativo              | Verificado en el alcance registrado    | Dos entradas mono/estéreo, Realtek/WASAPI, 48 kHz, salida estéreo f32; doce sesiones inicial/reapertura.                                            |
| Memoria y tiempos dentro del procesamiento instrumentado     | Correctos en las ejecuciones guardadas | Cero asignaciones/liberaciones/non-finite/excesos nominales registrados. No es una medición de todo el proceso o del driver.                        |
| Check, build y CI de la implementación publicada             | Correctos                              | Logs locales coincidentes y ambos jobs inspeccionados del SHA exacto.                                                                               |
| Escucha humana del sampler                                   | Confirmada: sonido audible             | El usuario responde «Si se escucha» tras la repetición de 20:59 UTC con audífonos bien conectados; sin inferir continuidad o separación de canales. |
| Cierre final de etapa 9                                      | Completado dentro del alcance          | Evidencia/CI y confirmación reunidas en cierre-etapa-9.md; límites físicos conservados.                                                             |

Las verificaciones completas de producto ya están registradas localmente y en
CI sobre código equivalente. Esta tarea cambia documentación y seguimiento;
no repite suites/build sin un cambio de implementación que lo justifique.
Las comprobaciones documentales realizadas al terminar se anotan en el registro
de implementación.

## Límites que se conservan

Las ventanas de voces son cortas y sus métricas se acumulan durante todo el
lifecycle; los histogramas ofrecen cotas de bins y los agregados pueden omitir
el último callback. Los tiempos medidos no prueban continuidad acústica.
No se realizaron loopback, desconexión física, estrés prolongado, pruebas de
banco máximo nativo, otros formatos/dispositivos físicos o audio nativo
Linux/macOS. Esos límites no se sustituyen por CI o adaptadores mock.

## Repetición solicitada para escucha — 2026-11-11

Ejecutada entre 20:54:01 y 20:54:34 UTC (14:54 hora de México) sobre
Speakers (2- Realtek(R) Audio), mismo device ID, binario release y WAV con hashes
verificados. Dos casos consecutivos: `mono-997.wav` y `stereo-distinct.wav`,
ambos con salida 48000 Hz/estéreo, 16 voces y ventana de cinco segundos.
Cada caso incluye el lifecycle del runner: ganancia, pausa/reanudación,
Stop/retrigger, finalización natural y cierre/reapertura.

Ambos finalizaron con código 0 y `sampler-native-check OK`. Se verificaron los
hashes de entrada antes/después y los logs guardados bajo
`target/stage9-listening-2026-11-11T20-54-01-577Z/`. El manifiesto de esta auditoría
añade esta repetición como evidencia separada de las seis mediciones originales.
No cambió código del motor. En ese momento se solicitó observación humana y
todavía no se había confirmado la escucha; el cierre llegó tras el último par
con audífonos, descrito abajo.

## Próxima acción

**Segunda repetición a petición del usuario:** desde 20:56:18 UTC, esta vez con
audífonos. La enumeración detectó como predeterminada
`Headphones (2- Realtek(R) Audio)`, device ID
`wasapi:{0.0.0.00000000}.{6ad7cba4-f52f-49bb-aea1-e43eee86e121}`.
Se dirigieron explícitamente a esa salida ambos casos mono/estéreo, conservando
48 kHz, salida estéreo, 16 voces y ventana de cinco segundos. Ambos finalizaron
con código 0 y aserciones correctas. Logs y reporte preservados en
`target/stage9-listening-2026-11-11T20-56-18-640Z/`; evidencia añadida como una
repetición adicional sin sustituir la anterior. Pendiente observación humana.

El usuario indicó que no había conectado bien los audífonos y pidió repetir.
El nuevo par se ejecutó de 20:59:47 a 21:00:19 UTC sobre el mismo endpoint
Headphones: mono/estéreo, 16 voces y ventanas de cinco segundos, ambos código 0.
Después confirmó «Si se escucha». Nuevos logs y reporte en
`target/stage9-listening-2026-11-11T20-59-47-430Z/`; la evidencia separa el intento
no oído y la confirmación del último. No se cambiaron niveles de Windows o DSP.

Con ello se registra el [cierre final de etapa 9](cierre-etapa-9.md).
**10.1 ya especificado:** [contrato del mezclador](../docs/mixer-contract.md).
El [núcleo 10.2](etapa-10-mezclador-motor.md) se implementó posteriormente con
huella y pruebas propias. Siguiente integración **10.3**, según el
[plan de ejecución](plan-siguientes-pasos.md).
