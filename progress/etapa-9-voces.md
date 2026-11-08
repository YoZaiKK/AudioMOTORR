# Etapa 9.2 — Banco fijo y kernel de voces

Fecha: 2026-11-08. Base limpia 5da0920 (recursos PCM de 9.1), con ambos jobs
correctos en [run 42528127881](https://github.com/YoZaiKK/AudioMOTORR/actions/runs/42528127881).
9.2 está completada y publicada en 7acc1db, con ambos jobs CI correctos para ese
SHA; la evidencia propia se detalla abajo. La etapa 9 completa sigue abierta:
9.2 añade reproducción raw en biblioteca, sin cargar archivos ni
abrir dispositivos o exponer un sampler en la UI.

## Implementación y aceptación

| Criterio                  | Implementación y evidencia                                                                                                                                                                                                    |
| ------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Banco fijo/inmutable      | SampleBank posee 16 Option<SampleBuffer>; valida rate/mapeo de cada slot y salida mono/estéreo, permite slots vacíos, no expone mutación ni Clone. Construcción/error/destrucción fuera del procesamiento.                    |
| Pool de voces             | Sampler posee 16 estados Copy opcionales: slot, cursor de siguiente frame y gain cuantizado a f32. Selecciona primer índice libre; retorna InvalidSlot/MissingAsset/PoolFull sin modificar estado.                            |
| Muestras                  | One-shot exacto, mono duplicado a estéreo, estéreo L/R conservado, gain constante, suma f32 por índice ascendente sin clipping ni headroom. Oráculos manuales, superposición y cancelación/redondeo.                          |
| Agotamiento/reutilización | Disparo 17 rechazado conservando 16 voces; gain cero sigue ocupando voz; se libera el estado exactamente tras el último frame y se reutiliza el primer hueco.                                                                 |
| Propiedad/cierre          | Fin de muestra y clear_voices solo cambian estado Copy, conservan PCM y contador acumulado. Se puede volver a disparar sin recargar. La destrucción del propietario sí libera el banco fuera del procesamiento.               |
| Errores/particiones       | Buffer desalineado y overflow producen silencio sin avanzar cursores/contador. Vacío no avanza. Muestras, snapshots y cola futura idénticos con bloques 1/127/256/4096 y comandos en posiciones explícitas.                   |
| Memoria medida            | Test opcional rt-diagnostics en daw-audio: cero asignaciones y liberaciones durante 100 rondas de render/disparos/errores/agotamiento/clear, en debug y release. Control positivo de destrucción detecta 4 buffers liberados. |

Sampler es un kernel raw independiente de AudioEngine. No tiene tempo, Play,
Pause, Stop, rampas o colas de servicio. clear_voices es un vaciado inmediato del
kernel; en 9.4 el adaptador lo llamará tras la rampa de release. Su contador
rendered_frames acumula todos los bloques válidos, incluso silenciosos, y no se
reinicia al vaciar voces. Un Sampler nuevo comienza vacío y en frame cero.
Los índices devueltos por trigger son reutilizables, no identificadores duraderos.

snapshot entrega configuración, contador y array Copy de voces para inspección
del kernel. No se añade a los snapshots del servicio ni a DTO/TS. Los comandos
y outcomes con ID de la cola, transporte y headroom 1/16 quedan para 9.4, según
el [contrato actualizado](../docs/sampler-contract.md).

## Pruebas ejecutadas

Nueve pruebas nuevas de comportamiento:

- Ocho en [sampler_contract.rs](../crates/daw-engine/tests/sampler_contract.rs):
  validación de banco, fixtures/mapeo, superposición, agotamiento/gain cero,
  selección/reutilización/clear, errores, particiones y orden de suma f32.
- Una interna en [sampler.rs](../crates/daw-engine/src/sampler.rs): overflow de
  u64, conservación de cursores/snapshot y buffer vacío en contador máximo.

La [prueba de memoria](../crates/daw-audio/tests/sampler_memory_contract.rs)
reutiliza allocation-counter 0.8.1 ya opcional en audio; no añade dependencias al
motor. Banco, PCM, scratch y Gain se preparan antes de medir. Dentro se procesan
100 rondas con 16 disparos, rechazo por pool lleno/asset ausente/índice inválido,
bloques 0/1/127/256/4096, final de muestra, canales mixtos, error de alineación,
snapshot y clear. Resultados tanto en debug como release:

- count_total = bytes_total = 0.
- count_current = bytes_current = 0: con cero asignaciones, prueba que tampoco
  hubo liberaciones dentro de la región medida.
- Destrucción separada: count_total 0, count_current -4 y bytes_current -16468,
  correspondientes a los cuatro PCM retenidos. Esto verifica que la medición
  detecta liberar memoria asignada antes de entrar a measure.

```powershell
cargo test -p daw-engine --no-default-features --locked
cargo test -p daw-engine --release --no-default-features --locked --test sampler_contract
cargo test -p daw-audio --features rt-diagnostics --locked --test sampler_memory_contract
cargo test -p daw-audio --release --features rt-diagnostics --locked --test sampler_memory_contract
cargo clippy -p daw-engine -p daw-audio --features daw-audio/rt-diagnostics --all-targets --locked -- -D warnings
npm run check
npm run desktop:build -- --debug --no-bundle
```

Resultados finales locales correctos:

| Comprobación                              | Resultado                                                                                                                                               |
| ----------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Focales debug/release                     | Ocho tests de kernel en ambos perfiles, overflow interno y prueba de memoria correctos; Clippy daw-engine/daw-audio con todos sus targets correcto.     |
| npm run check                             | 13 tests frontend, 59 Rust headless, 64 Rust workspace, nueve escenarios CLI; bindings, TypeScript, ESLint, formato, frontend build y Clippy correctos. |
| Suite audio instrumentada dentro de check | 15 pruebas correctas, 1 de hardware ignorada. No sumar suites como pruebas únicas: comparten cobertura.                                                 |
| Build desktop posterior al check          | Debug sin instalador correcto; target/debug/rust-daw-desktop.exe.                                                                                       |
| Documentación/workflow                    | Enlaces locales, formato y actionlint correctos; git diff --check limpio.                                                                               |
| Reproducibilidad                          | Lockfiles/TS generado idénticos antes/después del check/build y sin diff respecto a la base.                                                            |

Hashes SHA-256 conservados:

| Archivo                         | SHA-256                                                          |
| ------------------------------- | ---------------------------------------------------------------- |
| Cargo.lock                      | 1A3F4C34AC94A2040057545B28DA8243864CC4E69FA639728C483D00D8AE9C58 |
| package-lock.json               | ABCCDCEAE11E9A00DCF184D6039B2E55FC8397CDF4204C883DDEB2B61BA4CFBD |
| src/shared/api/generated/api.ts | CD72C52812C3AD12D3845A127EB9B16C8B90A41640F2118A6C34B583573E26FE |

9.2 queda completada con evidencia local y remota; la etapa 9 completa sigue abierta. check ya
ejecuta daw-audio con native-audio,rt-diagnostics y descubre el test de memoria
sin abrir hardware. Linux headless ejecuta los tests del kernel y omite el test
condicionado a esa feature. No se modifica workflow.

## Publicación y CI de la implementación

Commit publicado en main:
[7acc1db88915e5fe1b0c17db4837fc3b197d35ad](https://github.com/YoZaiKK/AudioMOTORR/commit/7acc1db88915e5fe1b0c17db4837fc3b197d35ad).
La [ejecución 42530699397](https://github.com/YoZaiKK/AudioMOTORR/actions/runs/42530699397)
corresponde a ese SHA exacto. Resultado completed/success, inspeccionado por API
oficial el 2026-11-08 a las 22:40:03 UTC:

| Job             | Resultado         | Evidencia                                                                                               |
| --------------- | ----------------- | ------------------------------------------------------------------------------------------------------- |
| headless-linux  | completed/success | [Ubuntu 24.04](https://github.com/YoZaiKK/AudioMOTORR/actions/runs/42530699397/job/128884026388)        |
| desktop-windows | completed/success | [Windows Server 2022](https://github.com/YoZaiKK/AudioMOTORR/actions/runs/42530699397/job/128884026489) |

Windows ejecutó regeneración determinista de bindings, check completo (incluido
el test de memoria instrumentado en debug) y build debug sin instalador. Linux
ejecutó el núcleo headless sin features nativas, Clippy y los nueve escenarios CLI.
Los tests focales release pertenecen a la verificación local descrita arriba.
Ningún job abrió hardware ni midió deadlines/continuidad del sampler nativo.

El cierre documental posterior actualiza Markdown y conserva este SHA de código
como evidencia. Cada revisión posterior tendrá su propia ejecución de Actions;
el éxito de 7acc1db no se atribuye automáticamente a otro commit.

## Documentación, alcance y continuidad

Se actualizan README, contrato de sampler/realtime, arquitectura, desarrollo,
guías, roadmap, checklist, plan y continuidad; el reporte 9.1 queda como evidencia
histórica con enlace al actual. No cambian dependencias, lockfiles, bindings,
scripts de check, UI o capturas. No se repite hardware ni se infieren deadlines
nativos a partir de una prueba de memoria en el hilo de tests. Las mediciones
de senoide de etapa 8 tampoco validan el futuro callback del sampler.

Resultados remotos de cada revisión publicada en
[Actions](https://github.com/YoZaiKK/AudioMOTORR/actions/workflows/ci.yml); mantener
separados los resultados de la base y del SHA nuevo.

Siguiente cambio: **9.3, preparación/carga y decodificación acotadas fuera del
procesamiento**. Fijar encodings WAV y límites de archivo/PCM/memoria temporal,
validar rate/canales antes de instalar un banco completo y asegurar cierre ante
fallo/cancelación sin modificar recursos vivos. Después 9.4 conecta transporte,
headroom, servicio/IPC/UI; 9.5 mide y cierra la etapa 9.
