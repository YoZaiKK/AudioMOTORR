# Resultado de la verificación de etapa 5

Reporte histórico del cierre de etapa 5. La implementación posterior de CI y la
revisión para publicación constan en [continuidad actual](../progress/continuidad-etapa-6.md)
y [detalles](../progress/detalles.md). Las notas sobre Git/CI de este reporte
describen el estado al tomar esta evidencia, no el estado de publicación actual.

Fecha: 2026-11-06. Windows, Rust 1.90.0, Node 22.23.4 y npm 10.9.9.
Se amplían pruebas y documentación, sin nuevas funciones del producto. El rótulo
Etapa 4 de las capturas corresponde a la interfaz que se está validando.

| Verificación ejecutada                       | Resultado                                                          |
| -------------------------------------------- | ------------------------------------------------------------------ |
| npm ci                                       | Correcto, 218 paquetes instalados.                                 |
| TypeScript, lint, formato, build Vite        | Correctos en npm run check.                                        |
| Tests frontend                               | 12 correctos.                                                      |
| Tests Rust headless                          | 25 correctos, sin Tauri/CPAL.                                      |
| Tests del workspace con configuración nativa | 29 correctos; test de hardware ignorado por defecto.               |
| Clippy headless/workspace y rustfmt          | Correctos; warnings tratados como errores.                         |
| Fixtures CLI                                 | Nueve casos correctos, bloques 1/127/256/4096 y errores esperados. |
| Regeneración y comprobación de bindings      | Correctas, hash idéntico.                                          |
| Lockfiles antes/después                      | Hashes idénticos.                                                  |
| Desktop debug sin instalador                 | Correcto, construido después del check.                            |
| WebView2 y audio real                        | Correcto; siete capturas y cierre normal.                          |

Headless y workspace comparten pruebas; no sumar sus cantidades como cobertura
independiente. El primer check detectó un import URL faltante en el ejecutor Node;
se corrigió y la secuencia completa posterior pasó.

## Matriz de evidencia

| Requisito                    | Pruebas y artefactos                                                                                                   |
| ---------------------------- | ---------------------------------------------------------------------------------------------------------------------- |
| Motor y particiones          | render_contract.rs: silencio, transporte y timeline con cambios en límites equivalentes.                               |
| Colas/lifecycle              | service_contract.rs y application_contract.rs: FIFO mixto, límite 32, cierre con acciones pendientes y tempo aceptado. |
| Errores y recuperación React | App.test.tsx y recorrido real DeviceAbsent.                                                                            |
| IPC y seguridad              | ipc_contract.rs: handler, argumentos, precisión JSON y ACL.                                                            |
| Publicación/cierre           | snapshot_publication.rs y salida normal de la ventana real.                                                            |
| CLI                          | test-files/cli-cases.json, ejecutor y tests Rust de smoke.                                                             |
| Reproducibilidad             | npm ci, --locked y hashes siguientes.                                                                                  |

## SHA-256 antes y después

| Archivo                         | Hash idéntico                                                    |
| ------------------------------- | ---------------------------------------------------------------- |
| Cargo.lock                      | 7749328186E04BA2E36DF54C1FCFF76F7055D76C83B2301BD93E7EFB315266F1 |
| package-lock.json               | ABCCDCEAE11E9A00DCF184D6039B2E55FC8397CDF4204C883DDEB2B61BA4CFBD |
| src/shared/api/generated/api.ts | 7D0EED2708401F1764F0C7FF33F704C995055666F8C406307F518D7F51C752E4 |

Estos valores corresponden a esta entrega. Actualizaciones deliberadas de DTO o
dependencias requerirán evidencia nueva.

## Hardware y capturas

Salida real: Speakers (2- Realtek(R) Audio), WASAPI, f32, 48000 Hz y dos canales.
Recorrido: apertura, error por ID inexistente y recuperación, Play, 137.5 BPM,
Pause, Stop, cierre y reinicio con callbacks confirmados y salida normal de ventana.
La observación previa al reinicio reportó 67 callbacks y último ID aplicado 4.

[metadata.json](screenshots/metadata.json) incluye fecha UTC, plataforma y snapshots
consultados después de cada PNG; sus contadores pueden ser ligeramente posteriores
a la captura. Se revisaron visualmente las siete imágenes.

En la última captura, «Aplicado: —» indica que no hay motor. El campo BPM
deshabilitado no confirma tempo aplicado. Al reabrir, el motor recupera el tempo
del documento, como muestra la captura de reinicio.

## Límites

- npm ci reportó diez avisos moderados. No se ejecutó audit fix ni auditoría nueva
  detallada, ni se cambiaron versiones para ocultarlos.
- Sin verificación Linux/macOS, desconexión física o formatos nativos distintos de
  los indicados. Sin medición de deadlines, allocator o continuidad audible.
- El smoke necesita hardware y queda fuera de las pruebas automáticas.
- CI pertenece a etapa 6. Sin nuevos commits ni push de estos cambios.
