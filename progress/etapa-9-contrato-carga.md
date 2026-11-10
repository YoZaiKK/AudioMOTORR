# Etapa 9.3.a — Contrato de carga y presupuesto de memoria

Reporte histórico de especificación. El lector/decoder individual de 9.3.b se
registra en el [reporte de implementación](etapa-9-lectura-wav.md); el banco
completo y sus presupuestos/permisos globales todavía están pendientes.

Fecha: 2026-11-09. Código base 7acc1db; HEAD documental 1002efa. Se conservan los
cambios locales previos del plan/auditoría. **9.3.a especificada; loader todavía
no implementado.** No se modifican Rust, UI, dependencias, lockfiles o bindings.

El [contrato principal de carga](../docs/sample-loading-contract.md) fija:

- WAV PCM16 y float32, mono/estéreo y rate objetivo exacto. Sin resampling,
  downmix, normalización de pico o clipping. PCM16 se divide por 32768; float32
  exige finito y dentro de -1..=1.
- Cabeceras clásicas y extensibles equivalentes, incluyendo los WAV float32 que
  ya produce export-wav. Se inspeccionó el escritor local hound 3.5.1: máscara
  mono 1 y estéreo 3. La política acepta mono 0/1/4 y estéreo 0/3.
- Preflight seekable antes de reservar PCM: tamaño físico/RIFF coherente,
  un fmt antes de un data, offsets/padding comprobados y hasta 4096 chunks.
  Metadata ignorada por seek, sin guardar cuerpos ni cargar el archivo completo.
- Entrada de 0–16 slots únicos, validada antes de I/O; carga secuencial por slot,
  un lector/decoder a la vez y banco completo como único resultado satisfactorio.

## Límites decididos

| Concepto                                    | Límite                                                                  |
| ------------------------------------------- | ----------------------------------------------------------------------- |
| Archivo codificado por entrada              | 16777216 bytes (16 MiB).                                                |
| Recurso PCM                                 | 1048576 frames; hasta 4194304 bytes mono u 8388608 estéreo.             |
| PCM retenido por banco                      | 134217728 bytes (128 MiB), 16 slots.                                    |
| Buffers conjuntos del lector/parser/decoder | 65536 bytes (64 KiB).                                                   |
| Extra PCM para posible Vec→Box              | 8388608 bytes (8 MiB).                                                  |
| Preparación de banco nuevo                  | 142671872 bytes (136 MiB + 64 KiB) de payload/buffers controlados.      |
| Banco activo + preparación                  | 276889600 bytes (264 MiB + 64 KiB) del mismo ámbito.                    |
| Cancelación durante decode                  | Comprobar como máximo cada 256 frames, además de los otros checkpoints. |

Se prepara un banco cada vez. Un resultado terminado conserva el permiso hasta
consumirse/descartarse; otra solicitud devuelve Busy. Esto evita acumular bancos
preparados sin control. El presupuesto contempla el posible realojamiento de
SampleBuffer::new al convertir Vec a Box. No incluye overhead del allocator,
paths/errores, stacks, caché del SO, frontend o driver: no es un límite de RSS
de toda la app, ni una garantía ya medida por un loader inexistente.

## Fallo, cancelación y propiedad

Se distinguen errores de solicitud, I/O, WAV inválido, formato no admitido,
incompatibilidad, límites/overflow, muestra inválida y reserva fallida, además
de Busy/Cancelled. Son categorías del contrato; no nuevos DTO/códigos IPC.

Cancelación cooperativa antes de archivos/reservas, entre cabeceras de chunks,
cada 256 frames, al volver de I/O y antes de entregar el banco. Si se observa
antes de entregar, se descarta el banco nuevo, se cierran entradas y se libera
el permiso en preparación. La entrega del banco completo fija el resultado;
cancelación posterior no revoca el objeto ya transferido. No se promete interrumpir
una llamada I/O bloqueada. Sesión/documento/recursos anteriores se conservan.

## Verificación y continuidad

Se comprobó la aritmética de bytes con Node y los extremos PCM16: -32768→-1,
32767→0.999969482421875. Se consultaron referencias primarias de Microsoft sobre
RIFF/padding y WAVEFORMATEXTENSIBLE, enlazadas en el contrato, y el escritor
versionado local para exigir compatibilidad con la exportación actual.
Estas comprobaciones validan decisiones/aritmética/documentación, no implementación
o mediciones de memoria de un decoder. Verificación final de esta actualización:

- Checker local extrae los diez límites numéricos del contrato y contrasta frames,
  slots y bytes con las constantes PCM existentes; suma preparación/coexistencia
  y extremos PCM16 correctos. Checkpoint de 256 frames cabe en el buffer definido.
- npm run format:check correcto; 291 destinos Markdown locales existentes.
- git diff --check sin errores. Rust, frontend, scripts, manifests, lockfiles y
  bindings sin diff; se conservan las ediciones documentales previas.
- No se repiten tests de runtime/hardware para una especificación documental.

Se actualizan roadmap, plan, checklist, auditoría, README y continuidad:
9.3.a completada como contrato; siguientes 9.3.b–e para preflight/decoder,
coordinación y pruebas de fallo/cancelación. 9.3.f verificará la implementación
antes de cerrar 9.3; después 9.4 conectará sampler nativo/UI y 9.5 cerrará etapa 9.
No se atribuye a estos cambios una nueva CI: son documentación local, sobre la
evidencia histórica de código registrada en [9.2](etapa-9-voces.md).
