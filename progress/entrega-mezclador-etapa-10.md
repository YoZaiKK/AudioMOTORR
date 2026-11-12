# Entrega del mezclador de cuatro canales

Commit entregado en main (e7f543f): `feat(mixer): add four-channel mixing and native diagnostics`

## Descripción de la entrega

El sampler permite disparar 16 voces, pero no ofrecía canales musicales
independientes para controlar su mezcla. Esta entrega añade cuatro canales con
ganancia, balance y mute, y una ruta por cada uno de los 16 slots. Los disparos
capturan su destino; una ruta nueva afecta a los siguientes disparos y las
voces existentes conservan el canal. El pool sigue siendo global de 16 voces.

La mezcla es explícita y conserva las fuentes anteriores. Domain/engine implementan
ajustes validados, suma determinista, headroom 1/16, rampas de 5 ms y picos por
canal/master. El servicio usa la FIFO acotada existente y agrupa los medidores
sobre el callback completo. Application separa selección, aceptación y aplicación,
e instala los últimos ajustes al consumir el banco preparado.

Desktop añade cuatro tiras, rutas, niveles, DTOs/TypeScript generado y cuatro
comandos con permisos para `main`. IDs/frames/secuencias u64 siguen como cadenas;
mono expone un solo nivel. Se corrigen las respuestas de consulta inicial
obsoletas y los recibos offline que describían incorrectamente la sesión actual.

Los comandos `mixer-bench` y `mixer-native-check` ejercitan explícitamente el
camino mezclado. Se conservan contratos, guías, matrices de aceptación, reportes
y manifiestos con huellas y límites. El conjunto documental incluye la auditoría
de cierre de etapa 9 ya realizada durante esta continuidad.

## Validación

- Check completo y build Windows desktop debug correctos: 39 casos frontend,
  cuatro IPC, nueve escenarios CLI y suites Rust/Clippy/memoria instrumentada.
- Fixtures independientes de muestras, rampas, rutas y particiones; WAV reales
  eliminados antes del playback, lifecycle y pool global en mono/estéreo.
- CLI release: offline, 11 unitarias y tres de proceso; soporte nativo, nueve
  unitarias y tres de proceso. Las suites se solapan.
- 24 benchmarks offline con 2000 iteraciones; oráculo y memoria correctos,
  cuatro frees PCM fuera del render por caso.
- Seis casos nativos y seis reaperturas Speakers Realtek/WASAPI, 48000 Hz estéreo
  f32, fuentes mono/estéreo y 0/1/16 voces iniciales. Cero allocations/frees,
  nonfinite y deadline misses medidos; máximo 206300 ns y ratio 0.02063.

## Estado de aceptación

La implementación y auditoría están entregadas en **e7f543f**, con ambos jobs
CI correctos en run 43147510037. Ver [publicación](publicacion-mezclador-10.4c.md).
**Usuario confirmó «Se escucha»; [etapa 10 cerrada](cierre-etapa-10.md) dentro de su alcance.**
Los registros anteriores de CI de 4dc2c6c conservan su alcance de sampler.
Las métricas nativas son acumuladas de todo el lifecycle; no prueban continuidad
acústica. Loopback, unplug, estrés y otras configuraciones/plataformas conservan
sus límites. No hay persistencia de proyecto ni secuenciador en esta entrega.

Evidencia: [auditoría](auditoria-cierre-mezclador-10.4c.md),
[medición nativa](etapa-10-medicion-nativa-mezclador.md) y
[aceptación local](cierre-integracion-mezclador-10.3d.md).
