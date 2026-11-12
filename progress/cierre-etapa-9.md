# Cierre de etapa 9 — Sampler

Fecha: 2026-11-11. **Etapa 9 completada dentro del alcance verificado.**
Implementación publicada: `4dc2c6c2e0882b8f131b68088feffa8def84ba83`.
La documentación de cierre y el contrato posterior 10.1 son cambios locales;
no se les atribuye una revisión publicada o CI propia.

## Resultado

Carga/cancelación WAV fuera del procesamiento, banco fijo de 16 slots y pool
global de 16 voces, transporte/rampas, sesiones manuales/CPAL, aplicación,
IPC/DTO/UI, resultados por ID y diagnósticos implementados y verificados.
La [auditoría 9.5.c](auditoria-cierre-9.5c.md) detalla aceptación, hashes y límites;
el [manifiesto de cierre](evidencia-cierre-9.5c.json) registra la confirmación humana.

## Evidencia de cierre

| Área                    | Resultado                                                                                                                                                                                                          |
| ----------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Código/configuración    | Los 120 archivos locales coinciden con la huella medida. Respecto a Git, cuatro difieren solo por finales de línea, con equivalencia de contenido comprobada.                                                      |
| Pruebas/build           | Logs originales de check, build y release verificados por hash. CI de la implementación inspeccionada: ambos jobs correctos en [run 42896571891](https://github.com/YoZaiKK/AudioMOTORR/actions/runs/42896571891). |
| Matriz nativa original  | Seis casos Realtek/WASAPI, entradas mono/estéreo y 0/1/16 voces; doce sesiones inicial/reapertura. Cero asignaciones/liberaciones/non-finite/excesos nominales medidos.                                            |
| Repeticiones de escucha | Tres pares mono/estéreo de 16 voces y ventanas de cinco segundos; primero altavoces y después dos pares por audífonos. Todos finalizan con código 0 y aserciones correctas.                                        |
| Escucha humana          | Tras el último par, el usuario confirma: «Si se escucha». Se registra audibilidad; no se amplía la observación a separación de canales, ausencia general de artefactos o continuidad acústica medida.              |

El primer par dirigido a audífonos no se oyó. El usuario aclaró que no estaban
bien conectados y solicitó repetir. La confirmación corresponde al último par,
iniciado el 2026-11-11 a las 20:59:47 UTC, dirigido explícitamente a
Headphones (2- Realtek(R) Audio), device ID
`wasapi:{0.0.0.00000000}.{6ad7cba4-f52f-49bb-aea1-e43eee86e121}`.
No se atribuye esa confirmación retroactivamente a la primera reproducción.

Logs/reportes nuevos conservados bajo `target/stage9-listening-*`;
el manifiesto versionable conserva comandos, fechas, hashes y métricas de los
casos. Los logs de las tres repeticiones y los manifiestos originales se preservan.
El código de producto no cambió durante el cierre; no se repitieron suites/build
ya cubiertos por logs y CI del mismo código. Sí se repitió audio a petición del usuario.

## Límites

La escucha confirma sonido audible. No sustituye loopback acústico ni acredita
continuidad de salida, ausencia de dropouts del driver o estrés prolongado.
Quedan sin verificar desconexión física, banco máximo nativo, otros formatos
físicos y audio nativo Linux/macOS. Los percentiles son cotas de bins; las métricas
de procesamiento son acumuladas del lifecycle y pueden omitir el último callback.

## Siguiente incremento

**10.1 especificado:** [contrato del mezclador](../docs/mixer-contract.md).
**10.2 implementado posteriormente:** [modelo y mezcla del motor](etapa-10-mezclador-motor.md),
con fixtures y comprobación de memoria. **Siguiente integración: 10.3**,
sesiones/comandos, ventanas de medidores por callback e IPC/UI.
La etapa 10 completa permanece pendiente; su evidencia nueva conserva huella propia.
