# Auditoría de entrega del mezclador — 10.4.c

Actualización posterior: [implementación e7f543f entregada y CI Linux/Windows correcta](publicacion-mezclador-10.4c.md).
Sigue pendiente la confirmación auditiva real. La auditoría siguiente conserva
el estado anterior a esa publicación.

Fecha: 2026-11-11, hora de México. **Auditoría local completada; etapa 10 abierta.**
Faltan confirmación auditiva del mezclador, entrega de una revisión propia e
inspección de su CI. El mensaje «Continua» no se registra como confirmación de
que el usuario haya escuchado las pruebas.

## Integridad comprobada

Se verificaron de nuevo los [diagnósticos](evidencia-diagnosticos-mezclador-10.4a.json)
y la [matriz nativa](evidencia-nativa-mezclador-10.4b.json), sin cambiar ni repetir
las mediciones. El [manifiesto de auditoría](evidencia-auditoria-mezclador-10.4c.json)
registra los resultados y la revisión de referencia.

| Evidencia              | Resultado                                                                                                                                                   |
| ---------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Fuentes/configuración  | Los 136 hashes y su huella agregada coinciden con 10.4.a y 10.4.b. No cambió la implementación después de check/build/medición.                             |
| Artefactos             | Coinciden 21 archivos distintos: binarios release offline/nativo y desktop debug, WAV, logs y metadatos guardados.                                          |
| Matriz nativa          | Seis casos, doce sesiones inicial/reapertura, exit status correctos y cierre a Uninitialized.                                                               |
| Fases de mezcla        | Se contrastaron siete fases por caso: voces, coeficientes/rampas, captura de rutas, mute con voces retenidas, pausa, final natural y señal tras reapertura. |
| Métricas               | Los doce registros coinciden con sus logs e histogramas; cero asignaciones/liberaciones/nonfinite/deadline misses medidos. Máximo 206300 ns, ratio 0.02063. |
| Benchmarks             | Los 24 casos conservan oráculo, memoria cero, cuatro frees PCM y sus bytes esperados en el propietario.                                                     |
| Verificaciones locales | Check, build y suites CLI release offline/nativo mantienen logs y hashes correctos. Se reutilizan porque no cambió su código.                               |

Huella de las fuentes medidas:
`cb7b56011b1d2010a6092414801b78115aebd6272030a74565aadb2772fc9112`.
Los agregados de tiempo/memoria tienen un callback de retraso; los probes de
muestras incluyen el callback actual. No se convierten en mediciones acústicas.

## Git y CI

HEAD y la referencia local `origin/main` siguen en
`4dc2c6c2e0882b8f131b68088feffa8def84ba83`. La API oficial de GitHub consultada
a las 02:59:17 UTC del 2026-11-12 confirma el mismo SHA en `main` remoto y
[run 42896571891](https://github.com/YoZaiKK/AudioMOTORR/actions/runs/42896571891)
`completed/success` para ese SHA. Esa revisión es del sampler y no contiene
la implementación local del mezclador.

Comparación de los 136 archivos con los blobs de ese HEAD: **88 idénticos en
bytes, cuatro equivalentes tras normalizar CRLF/LF, 28 con contenido nuevo y
16 rutas nuevas**. Se conservan las diferencias de finales de línea y los
manifiestos originales. La huella mide bytes del worktree, no es un SHA de Git
ni una certificación de CI.

El [workflow](../.github/workflows/ci.yml) ejecuta Linux headless y Windows desktop
al publicar en `main` o abrir un PR. Es necesario entregar una revisión que
contenga estos cambios e inspeccionar ambos jobs de ese SHA. Una ejecución
manual sobre el `main` actual volvería a verificar el sampler anterior.

## Entrega preparada y pendientes

Se prepara una [descripción de entrega](entrega-mezclador-etapa-10.md) y un patch
completo revisable bajo `target/stage10-4c-20261111/`, con cambios de archivos
existentes y archivos nuevos. Su inventario/hash se guardan en `delivery.json`.
Esta auditoría no modifica el índice ni realiza commit, push o publicación.

Para completar 10.4.c:

1. Registrar la respuesta auditiva real a la pregunta sobre Speakers Realtek.
   La confirmación anterior con audífonos pertenecía a etapa 9; no se reutiliza.
2. Entregar los cambios de implementación/documentación en una revisión propia
   y ejecutar/inspeccionar su CI Linux/Windows.
3. Asociar SHA, resultados y escucha a la evidencia, conservar límites y cerrar
   etapa 10 únicamente cuando se cumplan esos criterios.

Siguen sin verificarse loopback/continuidad acústica, comparación manual por
oído de pan/mute, desconexión, estrés prolongado, salida física mono, otras
configuraciones/dispositivos y audio nativo Linux/macOS. La medición actual se
limita a Speakers Realtek/WASAPI, 48000 Hz, estéreo f32. La implementación de
secuenciador, persistencia y clips permanece en 11–13.
