# Cierre local de 9.3 — Preparación de bancos WAV

Reporte histórico del cierre de preparación. El incremento posterior 9.4.a
modifica el motor y pruebas; ver [reporte actual](etapa-9-integracion-motor.md).
El manifiesto de 9.3 identifica su snapshot anterior, no los nuevos archivos.

Fecha: 2026-11-09. Se completa 9.3.f y se cierra **la preparación 9.3 en su
alcance local**. La etapa 9 completa sigue abierta: integración del sampler
en 9.4 y verificación nativa/cierre en 9.5. Esta auditoría añade documentación y
evidencia, sin modificar producción, pruebas, dependencias, DTO o UI.

## Revisión de aceptación

Se releyeron implementación, suites, contrato y reportes de 9.3.a–e. La
[matriz técnica](../docs/sample-loading-acceptance.md) conserva el detalle por
requisito y sus limitaciones; el [contrato](../docs/sample-loading-contract.md)
sigue siendo la referencia normativa.

| Criterio de 9.3                               | Implementación y evidencia                                                                                                                                                                           | Resultado / límite                                                                                                                                                                              |
| --------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Formatos, conversión y compatibilidad         | Lector en daw-app, round-trip del CLI real y cuatro pruebas WAV → banco → Sampler. 18 combinaciones de encoding/header/canales, 72 timelines, todos los códigos PCM16 a 44.1 kHz.                    | Cumplido localmente. PCM16/float32 clásico/extensible; sin remuestreo o downmix. No es compatibilidad universal con WAV.                                                                        |
| Contenedor y validación previa a PCM          | Preflight RIFF completo: tamaños, aritmética, padding, orden/duplicados/trailers, metadata por seek, precisión/GUID/máscaras, rate y canales.                                                        | Cumplido. Archivos reales en límites exactos y un exceso, además de fixtures malformados/unsupported. Reescrituras de igual longitud no se detectan universalmente.                             |
| Presupuestos de recurso, banco y coexistencia | Reserva exacta, scratch estructural acotado, límites comprobados antes de reservar; contabilidad conservadora y ocho pruebas instrumentadas.                                                         | Cumplido dentro del contrato. 128 MiB/banco, 136 MiB + 64 KiB preparación, 264 MiB + 64 KiB con activo. Payload/buffers, no RSS ni límite global entre coordinadores independientes.            |
| Banco completo y permisos                     | Validación de toda selección antes de I/O, 0–16 slots únicos, orden ascendente, un lector a la vez, paths repetidos con PCM independiente, PreparedBank y permiso compartido hasta entrega/descarte. | Cumplido. Resultado inmutable propietario; install transfiere a un holder del llamador. La instalación en servicio/stream corresponde a 9.4.                                                    |
| Errores, cancelación y cierre                 | Errores tipados con slot/fuente, 17 casos del segundo asset, interleavings sincronizados de retorno/Read, cierre en trabajador y saldo de heap en trabajador/receptor.                               | Cumplido con evidencia representativa. OOM/open/metadata incluyen inyección simulada; no se promete interrumpir I/O bloqueado, ordenar todas las races o instrumentar cada error con allocator. |
| Checks, build y documentación                 | Reporte 9.3.e: check completo, release focal y build Windows correctos; documentación y manifiesto de este cierre.                                                                                   | Cumplido localmente. No se afirma publicación/CI nueva o verificación nativa del sampler.                                                                                                       |

La fila antigua del roadmap que hablaba de mover el banco a una “session” se
precisa como preparación/entrega de propiedad. El contrato siempre reserva
instalación en servicio y cierre/recreación del stream para 9.4. El kernel raw
continúa separado de AudioEngine; no se presenta como sampler ya utilizable en UI.

## Evidencia identificable y verificaciones

Base Git: 1002efabfc70d59abb5526a62f07ef50f4f355d0, cierre documental anterior;
código de kernel publicado: 7acc1db. 9.3 permanece en el worktree sin commit nuevo.
El [manifiesto de evidencia](evidencia-preparacion-9.3.json) registra 110 archivos
de código, pruebas y configuración, su SHA256 individual y huella conjunta:

```text
46fffcf951567447ac4d2f286639061d2a5f90f36cccb4d2b9fbed78666675e9
```

La huella se captura en este cierre sobre los archivos que no cambian en 9.3.f;
identifica bytes locales, incluido fin de línea. No es SHA de commit, firma de
logs ni ejecución remota. Excluye documentación/progress, target y dependencias
instaladas; los archivos/cobertura exactos están declarados en el JSON.

Se reutilizan los resultados del [reporte 9.3.e](etapa-9-fixtures-integracion.md),
sin volver a ejecutar pruebas/build al no cambiar código o configuración:

- npm run check correcto: 85 pruebas headless, 90 workspace, 15 audio
  instrumentadas, ocho memoria decoder/banco, 13 frontend y nueve escenarios CLI.
  Suites solapadas; no sumarlas como pruebas únicas. Incluye bindings, TypeScript,
  ESLint, formato, build frontend, rustfmt y Clippy.
- Cuatro pruebas nuevas de carga/reproducción correctas en debug y release;
  las mediciones release anteriores mantienen sus reportes fechados de 9.3.b–d.
- npm run desktop:build -- --debug --no-bundle correcto, ejecutable local
  target/debug/rust-daw-desktop.exe. Logs no versionados bajo target/stage9-3e.
- Rust/cargo 1.90.0, Node 22.23.4 y npm 10.9.9 comprobados en este cierre.
  Cargo.lock, package-lock.json y api.ts generado mantienen los hashes registrados.

Para comprobar los archivos del manifiesto desde la raíz:

```powershell
$evidencia = Get-Content -Raw -Encoding UTF8 progress/evidencia-preparacion-9.3.json | ConvertFrom-Json
foreach ($archivo in $evidencia.files) {
  if ((Get-FileHash -LiteralPath $archivo.path -Algorithm SHA256).Hash -ne $archivo.sha256) {
    throw "La evidencia ya no coincide: $($archivo.path)"
  }
}
```

Cambios posteriores de código/configuración requieren actualizar la evidencia
y ejecutar las comprobaciones afectadas. Resultado de validaciones de este cierre:
110 archivos de código/configuración y huella conjunta coincidentes; ningún
archivo registrado tiene escritura posterior al log de check 9.3.e. Prettier,
375 destinos locales Markdown/imagen, diez límites numéricos del contrato,
actionlint y git diff --check correctos. No se repiten pruebas/build sin cambio
que lo justifique; este cierre reutiliza expresamente sus resultados anteriores.

## Pendientes separados del cierre local

- Publicar esta revisión e inspeccionar su CI exacta cuando se solicite. No hay
  commit/push nuevo ni evidencia remota propia; el éxito histórico de 7acc1db
  verifica ese kernel, no este loader.
- 9.4: AudioEngine con transporte/headroom/rampas de sampler, sesión manual/CPAL,
  outcomes acotados de triggers y aplicación/DTO/IPC/UI explícitos.
- 9.5: allocator/deallocations, tiempos/carga/continuidad y hardware identificado
  del sampler nativo, más CI de la revisión publicada y cierre de etapa 9.
- RSS total, fallos físicos de disco/OOM, unplug físico, loopback acústico,
  estrés prolongado, otros dispositivos/formatos y Linux nativo/macOS sin verificar.

Estos pendientes no se marcan como validados por pruebas offline, mock IPC o
mediciones previas de senoide. Guardar/exportar beats sigue en 12; mezclador y
secuenciador continúan en 10–11, después de completar sampler.

## Próximo cambio: 9.4.a

Integrar un banco preparado en un modo sampler de AudioEngine, sin dispositivos
ni UI todavía. Aplicar headroom fijo 1/16 después de la suma raw y reutilizar la
envolvente master de 5 ms. Definir resultado de disparo tipado/Copy para distinguir
procesado de voz iniciada/rechazada, con las reglas de transporte existentes.

Aceptar cuando las pruebas demuestren Play/Trigger y orden inverso, rechazo fuera
de Playing o durante release, Pause/reanudación de cursores, Stop con release y
limpieza sin liberar PCM, gain/BPM, errores sin avance y particiones exactas.
Las regresiones de silencio/senoide y la restricción de no asignar/liberar PCM
en procesamiento forman parte de la aceptación. Después 9.4.b conecta sesión y
outcomes del servicio; 9.4.c integra aplicación/IPC/UI. El
[roadmap](../docs/roadmap.md) y [plan](plan-etapas-7-13.md) mantienen esa secuencia.
