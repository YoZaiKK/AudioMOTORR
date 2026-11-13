# Etapa 11.4.a — Diagnósticos del secuenciador y protocolo nativo

Fecha: 2026-11-12. Base Git `2d4cc285434383b451116925f4227219d4bc5171`, más
11.3.b/c/d locales. **Herramientas implementadas y verificadas localmente.**
La etapa 11 sigue abierta: faltan medición/escucha física 11.4.b y auditoría,
entrega y CI del SHA correspondiente en 11.4.c.

## Entrega

- `sequencer-bench`: secuenciador explícito con patrones vacío/musical/denso,
  BPM exactos y bloques 1/127/256/4096, salidas mono/estéreo a 48 kHz. Un oráculo
  racional independiente comprueba cada muestra, fase/eventos/rechazos, rutas
  y picos de ventana; el patrón denso con colas largas agota las 16 voces.
- Todas las reservas preceden al bucle medido. Se miden solo render y su llamada;
  la validación queda fuera del cronómetro y dentro del contador de memoria.
  Cero asignaciones/liberaciones y bytes durante ese bucle; destrucción positiva
  de los cuatro buffers PCM en el propietario, fuera del procesamiento.
- `sequencer-native-check`: exige directorio con cuatro WAV y dispositivo
  explícito, instala mediante aplicación y valida el timeline de compases,
  ausencia de contaminación de disparos manuales, slot vacío, pausa/ediciones,
  mute con notas programadas, reanudación, Stop y cierre/reapertura. Registra
  métricas acumuladas por sesión y fases con IDs/frames/contadores exactos.
- Generador `scripts/sequencer-fixtures.mjs`: kick/snare/hat/accent breves en
  float32, mono o estéreo, manifiesto SHA256 y creación exclusiva sin sobrescritura.
  No es exportación de beats/proyectos.
- Gates de release/features, presets/opciones acotados y errores antes de I/O.
  Sin soporte nativo, argumentos válidos devuelven NativeAudioDisabled antes
  de leer el directorio. Los comandos de sampler/mezclador conservan sus caminos.
- [Contrato de diagnósticos](../docs/sequencer-diagnostics.md) y
  [protocolo nativo](../docs/sequencer-native-verification.md), con seis casos
  físicos propuestos y escucha separada de las observaciones automáticas.

Los nuevos presets admiten 20/95.5/127.25/300 BPM para poder contrastar fase por
fórmula racional. Esto no reduce los BPM admitidos por la aplicación normal.
La medición avanza al menos los compases solicitados tras 64 callbacks de warmup;
el inicio depende del bloque. Los contadores medidos son deltas de esa ventana,
no una comparación de particiones con distinta fase inicial.

## Pruebas y mediciones

Seis casos nuevos únicos: dos unitarios de benchmark/parser, lifecycle manual
con WAV y reapertura, gate native-disabled y dos contratos CLI release. Algunas
features/perfiles excluyen casos; no se suman ejecuciones repetidas como casos
distintos. También se amplían help/gates existentes.

El lifecycle real de la aplicación se ejecuta con audio manual para las salidas
mono/estéreo y los tres patrones iniciales: seis combinaciones. Comprueba fase
congelada, patrones/rutas y voces retenidos, eventos en mute, errores por slot
vacío y nueva sesión sin ACK viejo. No abre CPAL ni atribuye tiempos nativos a
estos fixtures. Los contratos CLI release ejecutan las 72 combinaciones y el
preset adicional 20 BPM; validan opciones antes de gates/I/O.

Matriz offline final: dos salidas × cuatro bloques × tres patrones × tres BPM,
**72 mediciones**, dos compases por ventana. CPU Intel i7-1185G7, Windows
10.0.26200 x64. El binario offline se conserva separado del nativo, ambos con
hashes. En cada caso pasan PCM/eventos/medidores y memoria cero. Cada destrucción
libera cuatro buffers: 240000 bytes mono, 480000 estéreo.

| Salida  | Casos | Rango de p99 por caso | Mayor tiempo individual | Excesos nominales |
| ------- | ----- | --------------------- | ----------------------- | ----------------- |
| Mono    | 36    | 200–251700 ns         | 251700 ns               | 14 en siete casos |
| Estéreo | 36    | 200–381200 ns         | 381200 ns               | 9 en cinco casos  |

Los **23 excesos** corresponden a bloques de un frame: presupuesto nominal de
20833 ns. El mayor ratio nominal fue 7.4352 en ese tamaño. Se conservan los
resultados, sin repetir para obtener números favorables. Son observaciones
offline del motor/cronómetro/sistema, no fallos medidos de callbacks ni promesa
de tiempo real duro. Los casos densos a 300 BPM observan rechazos PoolFull
contrastados por el oráculo; la ventana depende de la fase posterior al warmup.

Los ocho WAV generados se inspeccionan: headers, tamaños/hashes, muestras finitas
y no nulas, extremos cero, rechazo de directorios existentes sin cambiar hashes
y argumentos inválidos antes de crear archivos. La inspección no abre audio.

## Verificación y alcance

| Comando                                                                                                             | Resultado                                                                                                                                           |
| ------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------- |
| `cargo test -p daw-cli --release --features offline-diagnostics --locked --bin daw-cli --test diagnostics_contract` | Código 0: 15 casos de binario y cinco de CLI.                                                                                                       |
| `cargo test -p daw-cli --release --features rt-diagnostics --locked --bin daw-cli --test diagnostics_contract`      | Código 0: 12 casos de binario y cinco de CLI; el runner se prueba con backend manual.                                                               |
| `npm run check`                                                                                                     | Código 0: bindings, tipos/lint, 54 casos frontend, formato/build web, nueve escenarios CLI, suites Rust headless/workspace/instrumentadas y Clippy. |
| `npm run desktop:build -- --debug --no-bundle`                                                                      | Build Windows debug con assets, sin instalador. Resultado exacto en manifiesto.                                                                     |

El [manifiesto](evidencia-diagnosticos-secuenciador-11.4a.json) conserva fechas,
comandos, hashes, matriz completa y vínculo al manifiesto anterior. Logs,
binarios y fixtures locales en `target/stage11-4a-20261112/`.

Solo cambian CLI/diagnósticos/pruebas y el generador: tres fuentes anteriores
modificadas y tres nuevas; la huella pasa de 153 a 156 archivos. Motor, servicio,
aplicación, desktop, dependencias/locks, IPC/ACL/bindings y estilos conservan sus
hashes de 11.3.d. El check general vuelve a ejecutar sus regresiones. La captura
WebView2 anterior se conserva como histórica; no se abre otra ventana.

## Siguiente paso

**11.4.b: ejecutar la matriz física y obtener una observación de escucha.**
Usar el [protocolo](../docs/sequencer-native-verification.md) sobre un dispositivo
enumerado, guardar comando/entrada/binario/configuración y logs de los seis casos,
incluidas sus reaperturas. Registrar la escucha humana del secuenciador por
separado. Después 11.4.c auditará la revisión entregada y su CI antes de cerrar.
Este incremento no ejecuta audio físico, escucha, commit/push ni CI hospedada.
Loopback, desconexión física, estrés prolongado y otras plataformas conservan
sus límites. Persistencia/exportación del beat y clips siguen en etapas 12–13.
