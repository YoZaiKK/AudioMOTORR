# Etapa 13.3.d — Desktop de clips y escenas

Estado: **implementada y verificada localmente**, 2026-11-14 (México).
Entrada: [coordinación de aplicación 13.3.c](etapa-13-aplicacion-clips.md).
La etapa 13 continúa abierta.

## Entrega

- Ocho comandos de launcher/modo/tempo/inicialización, registrados en Tauri,
  manifiesto y permisos de la ventana local `main`. Entradas acotadas y
  validación de referencias; sin exponer despacho genérico de métodos.
- DTOs propios y bindings generados: selección/documento, recibo con copia de
  acciones, progreso y telemetría aplicada. IDs de servicio y de motor separados;
  `u64`/`u128`, orígenes, targets, contadores y fases como texto decimal exacto.
- Matriz 4 × 16, creación/eliminación y editor de 16 pasos/sample, 16 escenas
  con Keep/Stop/Launch por fila, modo, cuantización y escena inicial de exportación.
  Seleccionar/editar/lanzar una escena no cambia la escena inicial guardada.
- Lanzamientos de escena/clip y Stop por fila/total explícitos; inicialización
  según modo con PCM preparado, vacía y detenida. No auto escena/Play ni replay
  al reabrir. BPM editable con audio cerrado. Patrón conserva su contenido y
  deshabilita edición/inicialización incompatible al seleccionar Clips.
- Contenido seleccionado independiente del motor; A/Q de activo/en cola,
  orígenes, pasos objetivo, última transición y notas/contadores reales.
  `rowSteps` proyecta la posición al siguiente frame desde el snapshot exacto;
  se distingue de límites/notas ya observados. No reloj musical JavaScript.
- Recibos Accepted/Processed/Rejected/Unknown por ID exacto, sin inferir
  transición desde aceptación/procesamiento. Se ignoran eventos viejos;
  cierre invalida respuestas tardías. Errores no confirman ediciones y no hay
  reintentos/compensación automáticos. Consulta explícita recupera el estado.
- Modo navegador sin invocaciones nativas. Matriz con desplazamiento horizontal
  dentro del panel y controles adaptables a pantalla estrecha.

[Contrato desktop](../docs/launcher-desktop.md) y
[guía de uso](../howToRunThis/10-clips-escenas.md).

## Verificación

Ocho casos nuevos: cuatro de interfaz, uno del cliente y tres de IPC. El total
frontend pasa de 63 a **68**; no se suman ejecuciones solapadas como casos únicos.

- Interfaz: edición offline sin lanzar; escena inicial independiente; aceptación
  tardía después del snapshot; procesamiento exacto frente a transición pendiente;
  evento viejo ignorado; reporte perdido → Unknown; cierre/reapertura descarta
  acknowledgement tardío; error conserva selección; inicializador por modo sin Play.
- Cliente: nombres/argumentos tipados, ID de preparación máximo intacto y rechazo
  en navegador antes de invocar Tauri.
- IPC con aplicación real y backend manual: shapes/índices/grafo/unknown fields,
  preparación canónica, modo cerrado, inicialización vacía, snapshot seleccionado
  frente a contenido activo, solicitud procesada todavía pendiente, cola llena
  sin commit de operación compuesta ni reemplazo del recibo, cierre sin replay.
- Precisión: `u64::MAX` y `u128::MAX` en contadores, origen, target, IDs y fracción;
  permisos denegados para los ocho comandos desde otra ventana con argumentos válidos.
- `npm run check`: código **0**. Bindings, TypeScript, ESLint, 68 frontend,
  Prettier, build web, nueve escenarios CLI, formato Rust, pruebas headless/workspace,
  audio instrumentado, diagnósticos/memoria y Clippy con `-D warnings`.
- `npm run desktop:build -- --debug --no-bundle`: código **0**, ejecutable Windows
  compilado con assets actuales. No se generó instalador.
- Ventana real **Tauri/WebView2**: cambio a Launcher, clip de fila 0/slot 15,
  pasos 1/5/9/13, sample 15, escena 7 y cuantización pulso, verificados por IPC Rust.
  Cambiar la escena de edición a 3 conserva inicial 7. Lanzar permanece bloqueado
  offline; estado Uninitialized, cero callbacks y ninguna sesión de clips.
- Capturas revisadas a **1000 y 360 CSS px**: 64 celdas, cuatro pasos seleccionados,
  sin desbordamiento de página; matriz completa en ancho y desplazable en estrecho.
  Sin excepciones WebView; proceso cerrado normalmente.

Capturas: [ancha](../docs/screenshots/stage13-launcher-1000.png),
[estrecha](../docs/screenshots/stage13-launcher-360.png) y
[metadatos](../docs/screenshots/stage13-launcher.json).
El comprobador gráfico necesitó ajustar su espera inicial hasta que React montara
los controles; la ejecución registrada después del ajuste pasó completa.
La captura final usa un build desktop posterior al check: las pruebas de workspace
también recompilan el binario debug y pueden reemplazar la variante con assets
embebidos si se ejecutan en paralelo con ese build.

ts-rs emite tres avisos de macro porque no interpreta `deny_unknown_fields` en
enums; Serde sí lo aplica. Los contratos IPC prueban ese rechazo, incluidos Keep y
variantes sin contenido. Los tipos generados y su verificación pasan.

Logs en `target/stage13-3d-20261114/`: `check.log`, `desktop-build.log`,
`preview.log` y `preview.json`. El [manifiesto de fuentes y evidencia](evidencia-desktop-clips-13.3d.json)
registra hashes crudos, fuentes heredadas, configuración y capturas de esta entrega.
Son 22 fuentes/configuraciones de desktop y 62 acumuladas; las 40 heredadas
conservan sus hashes. Se comprobaron 515 enlaces locales en doce documentos.

## Continuidad

Siguiente: **13.3.e, aceptación integrada local** con WAV reales, archivos y
exportación, lifecycle/orden IPC/UI y sus verificaciones antes de diagnósticos 13.4.

Dominio, motor, servicio, aplicación y formato conservan sus fuentes de 13.3.c.
La dependencia workspace `daw-engine` del adaptador pasa de dev a normal para
convertir telemetría; no hay nuevas dependencias externas ni cambios de lockfiles.
No se abrió hardware ni se midió latencia o audibilidad. Sin commit/push ni CI nueva;
los manifiestos anteriores conservan su carácter histórico.
