# Etapa 14.1 — Modelo de arreglo en línea de tiempo

Fecha: 2026-11-16 (México). Base: `f5f7ded`; árbol limpio al iniciar.
**Implementada localmente.** El usuario pidió continuar; se inicia el siguiente
hito del roadmap sin convertir esa indicación en una confirmación de escucha.
13.4.b conserva la observación auditiva pendiente y 13 su cierre formal abierto.

## Entrega

- `ClipRegion`: clip de la misma fila, inicio y longitud en compases completos;
  intervalo semiabierto, longitud positiva y fin máximo de 4096 compases.
- `Arrangement`: cuatro filas × 32 identidades de región estables. Rechaza
  solapamientos dentro de una fila; permite adyacencia y coincidencias entre filas.
  Ediciones y errores sin cambios parciales; duración derivada del último fin.
- `resolve`: revalida referencias y produce `ResolvedArrangement` con clips
  propios. Ediciones/eliminaciones posteriores no alteran una copia resuelta.
- Consultas puras de región por paso global y paso local repetido, con huecos,
  fronteras exactas y posiciones fuera del horizonte, incluido `u64::MAX`.
- [Contrato](../docs/arrangement-contract.md), [plan 14](plan-etapa-14.md), README,
  roadmap y registros vigentes actualizados con el siguiente trabajo concreto.

El modelo es independiente: aún no forma parte de `Project`, JSON v2, DTOs,
sesiones ni UI. Se integrará junto con un formato versionado capaz de guardarlo.
No se agregan dependencias ni lockfiles. La resolución valida referencias del
arreglo actual; `ClipLauncher` independiente no puede impedir por sí solo borrar
un clip que otro documento referencia. La aplicación deberá comprobar ambos
grafos antes de eliminar fuentes al integrar persistencia.

## Verificación

`cargo test -p daw-domain --release --locked`: **26 pruebas correctas**, ocho
nuevas y 18 anteriores. Cobertura: las 128 celdas, orden temporal independiente
de identidad, 4096 compases, entradas extremas, todas las formas de solapamiento,
errores atómicos, adyacencia, referencias duplicadas/ausentes y snapshots propios.
Una referencia de ubicaciones y notas comprueba 772 posiciones distribuidas en
cuatro filas, con huecos y repeticiones. El último compás y el límite `u64` tienen
comprobaciones separadas. Estas consultas no prueban scheduling ni salida PCM.

`npm run check`: **código 0**. Incluye bindings, tipos, ESLint, 72 pruebas
frontend, formato, build web, nueve escenarios CLI, suites Rust
headless/workspace/audio instrumentado, diagnósticos, memoria y Clippy.
Las suites se solapan y sus ejecuciones no se suman como pruebas distintas.
También verifica las 26 pruebas de dominio en debug. El primer intento se detuvo
en una advertencia de Clippy del test nuevo; corregida con `is_multiple_of`,
se repitieron release y el check completo, ambos correctos.

Logs locales: `target/stage14-1-20261116/check.log` (intento inicial) y
`target/stage14-1-20261116/check-final.log` (correcto), con tiempos/códigos en
sus JSON asociados. `evidencia-modelo-arreglo-14.1.json` registra base Git,
hashes de las tres fuentes y diez documentos, comandos y enlaces locales.
De las 69 fuentes/configuraciones medidas en 13, solo cambia `daw-domain/src/lib.rs`;
las otras 68 conservan sus hashes. Las dos fuentes nuevas son el módulo y su suite.

## Siguiente incremento y límites

**14.2: reproducción determinista de arreglos en `daw-engine`.** Usar el reloj
musical exacto existente; resolver inicio/fin y huecos, transporte y tempo,
colas de samples y fin del arreglo. Comparar eventos/PCM con un oráculo
independiente, distintas particiones y medición de memoria del procesamiento.
14.3 conectará sesiones/aplicación, proyecto/exportación versionados y editor;
14.4 recogerá aceptación y evidencia física/CI propias.

No se abrió hardware ni se cambió la interfaz en 14.1. Las capturas del README
siguen describiendo el launcher entregado. No se atribuye CI de revisiones
anteriores al cambio local ni se presenta la línea de tiempo como reproducible
desde la app en esta entrega. Sin commit/push en este incremento.
