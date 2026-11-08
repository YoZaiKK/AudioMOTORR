# Etapa 9 — Recursos PCM y contrato del sampler (9.1)

Fecha: 2026-11-08. Base limpia main 80a2121, cierre de etapa 8. Ese SHA pasó
[CI Linux/Windows, run 42525499571](https://github.com/YoZaiKK/AudioMOTORR/actions/runs/42525499571).
La evidencia de la base no verifica los cambios nuevos. Etapa 9 permanece en
curso: este incremento prepara recursos y fija decisiones; todavía no reproduce
samples ni añade carga de archivos, comandos o controles de sampler.

## Implementación y decisiones de 9.1

- daw-engine exporta SampleBuffer, SampleError y MAX_SAMPLE_FRAMES. El recurso
  consume Vec<f32>, valida datos y conserva Box<[f32]> privado/inmutable, sin
  Clone, acceso mutable o copia de PCM por consulta. Debug solo expone metadatos.
- EngineConfig valida rate/canales positivos; el recurso restringe PCM a 1/2
  canales interleaved, finito y dentro de -1..=1. Exige al menos un frame completo
  y como máximo 1048576 frames (4 MiB mono / 8 MiB estéreo de payload).
- validate_for_output exige el mismo rate y salida mono/estéreo. Permite mono
  hacia mono/estéreo, conserva estéreo hacia estéreo y rechaza downmix. Solo
  comprueba compatibilidad; aún no implementa esos mapeos en reproducción.
- Construcción/validación/destrucción fuera del procesamiento. El constructor
  puede realocar o liberar el Vec rechazado; no es una operación realtime. Un
  futuro loader debe verificar tamaño/presupuesto antes de crear el Vec.
- Contrato fijado para 9.2: banco inmutable de 16 slots y pool de 16 voces;
  elegir la primera libre y rechazar el nuevo disparo al agotarse, conservando
  todas las voces existentes. Las voces solo contendrán índices/cursor/gain/estado;
  no serán propietarias de buffers ni liberarán PCM al terminar o hacer Stop.
- Se define suma determinista del kernel sin clipping y headroom 1/16 para la
  integración posterior, transporte con rampas, outcomes y diferencia entre
  comando aceptado/procesado y trigger iniciado/rechazado. Son criterios futuros,
  no comandos, snapshots ni reproducción ya implementados.

El [contrato técnico](../docs/sampler-contract.md) distingue cada parte implementada
de las decisiones futuras. No se añade un grafo genérico de instrumentos.

## Fixtures y pruebas

Los [fixtures en Rust](../crates/daw-engine/tests/support/sampler_fixtures.rs)
contienen MONO_PULSE, STEREO_STEPS, MONO_TAIL y un generador determinista de 4103
frames. Usan fracciones binarias exactas y no necesitan archivos, JSON o hardware.
El contrato fija oráculos futuros de salida, superposición y agotamiento.

Cuatro [tests de recursos](../crates/daw-engine/tests/sample_resource_contract.rs):

1. Propiedad de PCM, bits/metadatos conservados, canales estéreo distintos,
   ceros con signo y representación Debug acotada.
2. Rechazo de canales no admitidos, vacío, frames parciales, NaN, infinitos y
   amplitud fuera de rango; índice de la primera muestra inválida.
3. Aceptación del máximo exacto y rechazo del frame adicional para mono/estéreo,
   verificando que el límite es por frames, no por número de muestras.
4. Compatibilidad de rate/canales, con mono hacia mono/estéreo y rechazo de
   rate distinto, estéreo hacia mono y salida de más canales, sin alterar PCM.

```powershell
cargo test -p daw-engine --no-default-features --locked --test sample_resource_contract
cargo clippy -p daw-engine --no-default-features --all-targets --locked -- -D warnings
npm run check
npm run desktop:build -- --debug --no-bundle
```

Resultados locales correctos:

| Comprobación                        | Evidencia                                                                                                                                                  |
| ----------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Suite focal SampleBuffer            | 4 pruebas correctas.                                                                                                                                       |
| Clippy daw-engine --all-targets     | Correcto, warnings como errores.                                                                                                                           |
| npm run check                       | 13 tests frontend, 50 Rust headless, 55 Rust workspace y nueve escenarios CLI; TypeScript, ESLint, formato, bindings y build frontend correctos.           |
| Audio instrumentado dentro de check | 14 pruebas correctas, 1 de hardware ignorada; Clippy del CLI diagnóstico correcto. Las suites comparten pruebas: no sumar cantidades como cobertura única. |
| Desktop debug sin instalador        | Build correcto después de check, ejecutable target/debug/rust-daw-desktop.exe.                                                                             |
| Documentación/workflow              | Enlaces locales existentes, formato y actionlint correctos; git diff --check sin errores. Workflow/scripts sin cambios.                                    |
| Reproducibilidad                    | Cargo.lock, package-lock.json y api.ts mantienen hashes idénticos antes/después de check/build y sin diff respecto a la base.                              |

SHA-256 conservados:

| Archivo                         | Hash                                                             |
| ------------------------------- | ---------------------------------------------------------------- |
| Cargo.lock                      | 1A3F4C34AC94A2040057545B28DA8243864CC4E69FA639728C483D00D8AE9C58 |
| package-lock.json               | ABCCDCEAE11E9A00DCF184D6039B2E55FC8397CDF4204C883DDEB2B61BA4CFBD |
| src/shared/api/generated/api.ts | CD72C52812C3AD12D3845A127EB9B16C8B90A41640F2118A6C34B583573E26FE |

9.1 queda completada localmente; la etapa 9 completa sigue abierta. Las nuevas
pruebas verifican recursos, no reproducción, ausencia de asignaciones/liberaciones
en voces ni tiempos del sampler. Los checks existentes y ambos jobs descubren
la suite nueva al seleccionar daw-engine, sin cambiar workflow o scripts.

## Documentación y continuidad

Se actualizan README, arquitectura, contrato realtime, desarrollo, roadmap, guía
de pruebas, checklist, plan, detalles y continuidad. La UI sigue ofreciendo
silencio/senoide de etapa 8. Sus capturas, mediciones y reportes históricos se
conservan. No se repiten mediciones físicas para un recurso que aún no se renderiza.
No hay cambios de dependencias, Cargo.lock, package-lock.json, DTO o TS generado.

Los resultados remotos deben consultarse para el SHA publicado en
[Actions](https://github.com/YoZaiKK/AudioMOTORR/actions/workflows/ci.yml), aparte
del resultado de la base y de las comprobaciones locales. La publicación de un
contrato de 9.1 no equivale a completar la etapa 9.

Siguiente incremento: **9.2, banco fijo y kernel de 16 voces**. Usar estos recursos,
probar muestras exactas, superposición, agotamiento/reutilización, vida útil y
particiones 1/127/256/4096. Después siguen 9.3 preparación/carga fuera del callback,
9.4 integración nativa/IPC/UI y 9.5 mediciones/checks/CI del sampler. Ver
[roadmap](../docs/roadmap.md) y [plan](plan-etapas-7-13.md).
