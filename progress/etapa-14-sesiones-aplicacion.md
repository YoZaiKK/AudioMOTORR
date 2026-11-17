# Etapa 14.3.a — Sesiones y aplicación de arreglos

Fecha: 2026-11-17 (México). **Implementada localmente** sobre `081f91b`, con
árbol limpio al iniciar. Se continúa según el plan vigente; la escucha de 13
conserva su estado pendiente, sin detener esta integración.

## Entrega

- Sesiones explícitas `initialize_manual_arranged_sampler` y
  `initialize_native_arranged_sampler` en `AudioHandle`. Instalan banco/copia
  validada en el control, con tempo/mezcla exactos, detenidas en cero y sin Play.
- `ServiceSnapshot.arrangement` y publicación Copy en la cola de observaciones.
  Solo incluye estado aplicado del motor; el documento/PCM no cruza el FIFO.
- Controles existentes compartidos: 128 comandos, presupuesto 32 por callback,
  64 observaciones. Rechazos de modo/cola llena sin consumir IDs, pérdida de
  observaciones sin repetir controles y cierre/fallo con estado descartado.
- Selección nativa con rate/layout del banco, conversión por bloques y medidores
  del callback completo; retorno del procesador al propietario antes de liberar
  PCM si el backend destruye el callback en otro hilo.
- Aplicación: `initialize_manual_arrangement`, `initialize_arrangement` y
  `get_arrangement_status`. Resuelve clips actuales antes de consumir preparación,
  usa tempo/mezcla al instalar y conserva contexto de contenido propio.

El arreglo se entrega explícitamente al inicializador; el contexto instalado es
transitorio y no forma parte del documento seleccionado. Se prueban los modos
Project Pattern y Launcher existentes sin añadir todavía un modo al JSON v2.
El proyecto y dirty state no cambian al instalar. Tras un timeout posterior a una
instalación real puede existir observación con `installed == None`; se consulta
o cierra, sin inventar contenido, compensar ni reintentar automáticamente.

Referencias inválidas/IDs obsoletos dejan listo el banco para corregir/reintentar.
Carga/cancelación conservan propiedad y permiso del worker. Una vez tomado el
banco, inicialización correcta o fallida lo consume. Cerrar/reabrir exige nueva
preparación explícita y captura de contenido; los comandos antiguos no sobreviven.
El [contrato](../docs/arrangement-service.md) especifica API, estados y límites.

## Verificación

**Release:** suite completa `daw-audio` con `native-audio,rt-diagnostics`,
**90 pruebas correctas**; cuatro casos públicos de aplicación y dos casos
internos de arreglo también correctos. Clippy de audio/aplicación con features,
todos los targets y `-D warnings`: correcto. Se corrigió el aviso de complejidad
de la tupla inicial mediante el alias `InitialState` antes de estas verificaciones.

La matriz nativa sin dispositivo compara 32 configuraciones (mono/estéreo,
f32/f64/i16/u16, scratch 1/127/256/513), siete callbacks cada una, contra callbacks
sin particionar. PCM convertido, estado de arreglo/motor, controles, voces y
medidores completos coinciden. Cero asignaciones/liberaciones en procesamiento.
La referencia independiente musical/PCM del kernel sigue siendo la de 14.2;
esta matriz comprueba el adaptador alrededor del mismo motor.

Otras pruebas cubren FIFO/presupuesto, cola llena, pérdida/recuperación de estado,
fin/colas, buffers vacíos/inválidos, overflow previo a chunks, fallo/reinicio,
instalación expirada y liberación fuera del procesamiento. La matriz realtime
instrumentada cubre seis combinaciones 1/800/48000 Hz × mono/estéreo, controles,
publicación y liberación positiva de los dos PCM en el propietario.

La aplicación carga WAV reales y verifica selección tardía de clips/tempo/mezcla,
contexto inmutable, documento intacto, rechazo de edición de otro modo, IDs
obsoletos, referencias ausentes, cancelación, Busy, fallo nativo sin feature,
rollback con cola llena y pérdida determinista de la respuesta de inicialización.
Un worker bloqueado comprueba Loading/Cancelling y una instalación nueva posterior.

`npm run check`: **código 0**. Bindings, tipos, ESLint, 72 pruebas frontend,
formato, build web, nueve escenarios CLI, suites Rust headless/workspace,
audio instrumentado, diagnósticos, memoria y Clippy correctos. Las suites se
solapan; no se suman sus ejecuciones como pruebas distintas.

Logs y tiempos en `target/stage14-3a-20261117/`; el manifiesto
`evidencia-sesiones-arreglo-14.3a.json` identifica fuentes, resultados y límites.
Las once fuentes de dominio/motor/pruebas/check heredadas de 14.1–14.2 conservan
el contenido de `081f91b`. Se añade únicamente `daw-domain` como dependencia de desarrollo de
`daw-audio`, ya presente en el workspace; Cargo.lock refleja esa relación.
No se agregan paquetes externos ni dependencias de producción. El fixture IPC
existente añade `arrangement: None`; no se exponen DTOs ni se cambian bindings/UI.

## Siguiente paso

**14.3.b: proyecto versionado y exportación de arreglo.** Integrar documento y
modo con esquema nuevo, migrar v1/v2, validar referencias antes de reemplazar el
proyecto y renderizar/exportar con el mismo motor y política de colas explícita.
Después vendrán editor/IPC y aceptación integrada. [Plan](plan-etapa-14.md).

No se abrió hardware ni se midió latencia/tiempo de callback físico. La compilación
nativa y las simulaciones no establecen audibilidad. JSON v2, guardado y exportación
de patrón/escena conservan su alcance; todavía no guardan/exportan el arreglo.
Los controles desktop y capturas existentes no cambian. Sin commit/push aquí;
no se atribuye CI remota previa a esta revisión local.
