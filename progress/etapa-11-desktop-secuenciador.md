# Etapa 11.3.c — IPC y cuadrícula del secuenciador

Fecha: 2026-11-12. Implementado localmente sobre `2d4cc285434383b451116925f4227219d4bc5171`
más los cambios locales 11.3.b. Sin commit/push en este incremento. La etapa 11
sigue abierta; siguiente: **11.3.d, aceptación local integrada**.

## Entrega

- Cinco comandos nuevos, 25 en total, con permisos solo para la ventana main.
  DTOs y bindings generados: patrón fijo 4 × 16, selección/recibos y observación
  completa del secuenciador. IDs, frames, contadores, loops y fase u128 como strings.
- Cuadrícula accesible de 64 pasos, slots por fila, agrupación por negras,
  vaciado atómico, consulta explícita y posición del último paso observado.
  Selección, patrón aplicado y columna observada tienen indicadores distintos.
- Inicialización explícita del secuenciador desde un banco preparado, detenida
  y sin Play automático. Se conserva carga/cancelación, modos anteriores y mezcla.
- Respuestas/eventos en cualquier orden, IDs y secuencias con BigInt, recibos
  desconocidos tras pérdida de sesión y consultas iniciales obsoletas descartadas.
  Sin sobrescribir selección desde snapshots ni compensar timeouts inconclusos.
- Vista navegador deshabilitada; edición offline durante preparación y bloqueo
  en modos de audio incompatibles. Grid desplazable internamente en ancho estrecho.

Detalle: [contrato desktop](../docs/sequencer-desktop.md) y
[guía de uso](../howToRunThis/08-secuenciador.md).

## Verificación

18 casos nuevos: 14 de UI, uno del cliente IPC y tres de IPC Rust. Frontend pasa
54 casos totales; IPC pasa siete. Los casos nuevos cubren validación de forma,
permisos con argumentos válidos, precisión u64/u128, recibos/observación/FIFO,
cola llena, fases de preparación, errores, respuestas tardías, pérdida/reapertura
y posición observada sin reloj musical del navegador.

Build desktop Windows debug sin bundle correcto. Se inspeccionó WebView2 real:
16 celdas activadas desde botones y selección de slot confirmadas mediante IPC
Rust; 64 botones presentes. La sesión permaneció `uninitialized`, sin motor ni
callbacks. La primera vista recortaba el último paso; se ajustó el ancho y se
repitieron build/capturas. La versión final muestra los 16 pasos a 1000 px y
mantiene scroll interno a 360 px, sin desbordamiento de la página.

Capturas [normal](../docs/screenshots/stage11-sequencer-wide.png),
[estrecha](../docs/screenshots/stage11-sequencer-narrow.png) y
[metadatos](../docs/screenshots/stage11-sequencer.json). La vista estrecha se
configuró mediante CDP. El binario inspeccionado se conserva localmente en
`target/stage11-3c-20261112/desktop-preview.exe` con hash contrastado.
La ventana cerró normalmente. Estas capturas no prueban reproducción física,
continuidad acústica, tiempos de callback ni lectura fina de cada paso por oído.

Verificación final correcta:

- `npm run check`, código 0, 22:35:13–22:36:25 UTC: bindings, tipos, lint,
  54 casos frontend, formato, build web, nueve escenarios CLI, suites Rust
  headless/workspace/audio instrumentado/diagnósticos/memoria y Clippy.
- Build desktop Windows debug sin bundle, código 0, 22:22:37–22:22:47 UTC.
- Pruebas IPC: cuatro anteriores y tres nuevas, correctas dentro del check.
- WebView2 real: edición del documento y layout normal/estrecho correctos, sin
  inicializar audio; capturas inspeccionadas visualmente.

La huella de captura precede únicamente a un ajuste de espacios en App.test.tsx;
el manifiesto contrasta las 152 fuentes y verifica que la producción y el binario
inspeccionado conservan su identidad. Los casos de pruebas se solapan entre suites;
no se suman ejecuciones como casos diferentes.

Resultados finales y hashes en el
[manifiesto 11.3.c](evidencia-desktop-secuenciador-11.3c.json).
Logs locales: `target/stage11-3c-20261112/`. Los reportes y manifiestos anteriores
conservan sus capturas fechadas. Motor y servicio de audio no cambian en 11.3.c.

## Siguiente paso

11.3.d audita la integración completa: beat de referencia con WAV reales desde
documento/preparación hasta eventos/PCM, lifecycle/errores, transporte/ediciones,
orden/pérdida de observaciones y UI. Consolidar check/build y evidencia de la
revisión, sin atribuir resultados históricos a código nuevo.

11.4 agrega diagnósticos/verificación nativa, escucha y CI de la revisión entregada
antes de cerrar la etapa 11. La prueba visual actual abrió la aplicación, no una
sesión de audio. Persistencia y exportación de beats siguen en 12.
