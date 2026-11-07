# Cierre de etapa 7 — Oscilador offline y WAV

Fecha: 2026-11-07. Estado: **etapa 7 cerrada**, implementación publicada y
verificada localmente y en CI remota para el SHA indicado. Los dos jobs de la
revisión de implementación terminaron en success antes de registrar este cierre.

Commit de implementación: `c6cf62703bcd8207030d6f87d38e51d0804d18e1`,
`feat(engine): add offline sine rendering and WAV export`, publicado en origin/main.
La base anterior era `1ef9bd0`; sus resultados no sustituyen la CI de esta entrega.

## Criterios de aceptación

| Paso | Entrega                                       | Evidencia                                                                                                                                                                               |
| ---- | --------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 7.0  | Revisar CI de la base.                        | Run 42314334447, ambos jobs correctos para 1ef9bd0; preservado en reporte de motor.                                                                                                     |
| 7.1  | Fijar contrato y límites.                     | API con señal opt-in, frecuencia/amplitud finitas, fase/transporte definidos; WAV float32 y hound 3.5.1 solo en CLI.                                                                    |
| 7.2  | Oscilador en daw-engine.                      | Seis tests de señal y overflow ampliado: referencia, fase, parámetros, transporte, BPM y particiones exactas; silencio por defecto conservado.                                          |
| 7.3  | Render offline y exportación.                 | Cuatro tests de aplicación y seis de CLI: buffer reutilizado, estado vivo independiente, WAV reabierto, metadatos/muestras, límites, no sobrescritura y errores de finalización/cierre. |
| 7.4  | Integración, documentación y comprobaciones.  | Check completo, build Windows posterior, contratos/guías y escenarios existentes; selección automática de pruebas en ambos jobs, sin hardware.                                          |
| 7.5  | Publicación y resultado remoto del nuevo SHA. | Run 42419963899: ambos jobs success para c6cf62703bcd8207030d6f87d38e51d0804d18e1.                                                                                                      |

## Verificación local final

Se repitió npm run check antes del commit: 12 pruebas frontend, 41 Rust headless,
45 Rust workspace y nueve escenarios CLI, todos correctos. También bindings,
TypeScript, ESLint, Prettier, build Vite, rustfmt y Clippy con -D warnings.
Las suites Rust se solapan; no sumar sus cantidades como cobertura independiente.
La prueba de hardware sigue ignorada por defecto.

Después se ejecutó npm run desktop:build -- --debug --no-bundle: correcto, con
target/debug/rust-daw-desktop.exe generado sin instalador. La revisión de cambios
y los 157 destinos documentales locales fueron correctos antes de añadir este
reporte. Tras incorporar CI, Prettier, los 167 destinos locales del conjunto
documental y git diff --check fueron correctos antes del commit de cierre.

Los hashes permanecieron como los del [reporte WAV](etapa-7-exportacion.md):
Cargo.lock incluye deliberadamente hound; package-lock y api.ts no cambiaron.
El WAV real y su lectura independiente en Node se conservan como evidencia allí.

## Resultado remoto

API oficial consultada el 2026-11-07 a las 22:27:29 UTC. Run creado a las
22:18:10 UTC y actualizado como terminado a las 22:23:36 UTC. El SHA inspeccionado
coincide exactamente con c6cf62703bcd8207030d6f87d38e51d0804d18e1.

| Evidencia                                                                                           | Resultado                                                                                       |
| --------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------- |
| [Run 42419963899](https://github.com/YoZaiKK/AudioMOTORR/actions/runs/42419963899)                  | completed / success.                                                                            |
| [headless-linux](https://github.com/YoZaiKK/AudioMOTORR/actions/runs/42419963899/job/128546063263)  | completed / success; núcleo y CLI de esta implementación en Ubuntu 24.04.                       |
| [desktop-windows](https://github.com/YoZaiKK/AudioMOTORR/actions/runs/42419963899/job/128546063144) | completed / success; bindings, check completo, build debug y diffs de archivos en Windows 2022. |

La consulta sufrió un 504 temporal, se reintentó y se obtuvo el resultado real;
el 504 no fue un fallo del workflow. La evidencia bruta local está en
target/stage7-implementation-ci.json, ignorada por Git; los enlaces anteriores
son la evidencia remota consultable.

El commit de cierre incorpora solo documentación sobre esa implementación y
sus resultados; el diff confirma que código, dependencias, lockfiles y tipos
no cambian. Su propia ejecución de Actions se comprobará antes de entregar
el estado final.

## Alcance y continuidad

La etapa entrega síntesis/exportación offline mediante motor, aplicación y CLI.
Desktop y los smoke nativos siguen generando silencio; no cambian DTO ni UI.
Los recursos de archivo cierran al finalizar/fallar y los destinos existentes
se preservan. Un fallo después de crear el nuevo archivo puede dejarlo parcial;
la ruta se informa y no se elimina/reintenta automáticamente.

Pendientes independientes: Linux nativo/macOS, desconexión física, formatos de
hardware adicionales y mediciones de allocator/deadlines/continuidad en release.
Exportar señal offline no demuestra comportamiento de tiempo real en hardware.

Siguiente etapa: 8, reutilizar el mismo oscilador por CPAL, añadir rampas y
controles explícitos vía aplicación/IPC/UI, y registrar mediciones en release
sobre hardware identificado. No se implementa etapa 8 durante este cierre.
