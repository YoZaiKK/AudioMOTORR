# Etapa 10.3.c — Mezclador en desktop

Fecha: 2026-11-11, hora de México. Base Git `4dc2c6c` con 10.2–10.3.b locales
preexistentes. **Integración DTO/IPC/ACL/UI implementada localmente.**
Siguiente: aceptación integrada 10.3.d; etapa 10 sigue abierta hasta 10.4.

## Entrega

- Cuatro comandos nuevos: consulta de selección, edición de canal, ruta por slot
  e inicialización explícita de sampler mezclado. La capability de `main` concede
  cada comando; otras ventanas siguen rechazadas. Total: 20 comandos.
- DTOs y TypeScript generado exponen selección, recibos, targets aplicados,
  coeficientes/rampas, destinos capturados y medidores de callback. IDs, secuencias
  y frames u64 conservan precisión como cadenas; el canal derecho mono es null.
- Cuatro tiras responsivas con borradores validados gain/pan/mute y botón Aplicar,
  selección separada de observación, rutas de 16 slots y niveles de canal/master.
- Consulta inicial independiente de enumeración de dispositivos; bloqueo hasta
  recuperar selección y opción de consultar de nuevo. Sin inicialización o Play
  implícitos. Se conserva la inicialización de sampler original.
- Recibos offline sin ID; recibos en vivo pendientes hasta watermark observado.
  Una rampa en curso permanece visible después de procesar el cambio. Secuencias
  antiguas se descartan usando BigInt; cierre/pérdida antes o después de recibir
  el recibo deja la aplicación desconocida, sin confirmarla desde otra sesión.
- Errores de cola/timeout conservan la selección y el borrador. Fallo de apertura
  refresca el banco consumido. Navegador y fuentes legacy mantienen controles
  deshabilitados. Los medidores solo aparecen con una ventana medida y distinguen
  mono/estéreo; no se inventan niveles ni se confunde nivel de canal con master.

Contrato: [desktop](../docs/mixer-desktop.md). Guía:
[mezclador](../howToRunThis/07-mezclador.md).
Domain/engine/audio/app, decoder, presupuestos RT y dependencias se mantienen
sin cambios frente al manifiesto 10.3.b. Cambian los bindings y ACL de forma
intencional; no hay nuevos permisos genéricos.

## Verificación y evidencia

Se incorporan **12 casos de interfaz y una prueba IPC integrada**. Las pruebas
existentes de serialización, permisos y cliente se amplían para cubrir mezcla.
La prueba IPC usa backend manual con WAV float conocido y verifica selección,
instalación sin Play, ruta capturada, pico derecho `1/32`, recibo pendiente, target
aplicado con 239 frames de rampa, retención tras cierre y rechazo legacy.
Los casos de UI cubren eventos anteriores al recibo, IDs mayores a 2^53, ventanas
mono/estéreo, cierre/error/reinicio, query lenta, fallo de dispositivos, cola/timeout
e inicialización explícita tanto correcta como fallida.

Los resultados finales, comandos, fechas y hashes de logs se registran en el
[manifiesto 10.3.c](evidencia-desktop-mezclador-10.3c.json), con huella nueva de
fuentes/configuración. Se conservan los manifiestos anteriores como históricos.
La huella cubre 133 archivos; 16 cambian frente a 10.3.b, incluidos dos nuevos.

| Verificación                                   | Resultado                                                                                                                                         |
| ---------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------- |
| `npm run check`                                | Correcto: bindings, TypeScript, lint, 36 casos frontend, formato, build web, nueve escenarios CLI, Rust headless/workspace, memoria y Clippy.     |
| `npm run desktop:build -- --debug --no-bundle` | Correcto: ejecutable debug de Windows, sin instalador.                                                                                            |
| Inspección visual headless                     | Correcta con datos IPC simulados a 1200/760/390 px: cuatro/dos/una columna, cuatro tiras y diez medidores estéreo, sin desbordamiento horizontal. |

Logs y capturas locales: `target/stage10-3c-20261111/`. La captura visual
representa un fixture, no una sesión de audio física. La ejecución visual
necesitó permiso fuera del sandbox después de que su proceso GPU no pudiera
arrancar dentro de él; la repetición autorizada terminó correctamente.
Se corrigieron avisos de formato de un test y del manifiesto; el check completo
se repitió correctamente sobre las fuentes finales.

## Límites y siguiente paso

La capa de control/serialización puede reservar memoria; el procesamiento RT
permanece en los crates previos y se vuelve a verificar con el check general.
El watermark confirma procesamiento FIFO, no coincidencia permanente del target
ni final de rampa. La UI conserva el último recibo, no un historial ilimitado.
Un cierre/reinicio externo completamente omitido por las observaciones no puede
identificarse sin identidad pública de sesión; lifecycle propio serializado.

Sin nueva escucha/medición física, commit/push o CI hospedada. Las verificaciones
del sampler y su escucha confirmada no se atribuyen a la mezcla nueva.
**10.3.d** auditará aceptación local de extremo a extremo y coherencia de evidencia;
**10.4** completará comprobación nativa de mezcla, evidencia, CI del SHA entregado
y cierre. Secuenciador, persistencia y clips siguen en 11–13.
