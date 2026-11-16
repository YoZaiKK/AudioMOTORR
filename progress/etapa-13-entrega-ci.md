# Etapa 13.4.c — Entrega, revisión y CI del launcher

Estado de entrega: **CI Linux/Windows correcta** para `ca37f74`;
paquete local preparado y verificado. **Cierre de etapa 13 pendiente de
confirmación explícita de escucha** por parte del usuario.

## Revisión de entrega

- Commit de implementación: `03055988a37faf68af7d1cd0bd3b02315af6799b`, 111 archivos.
- Commit de evidencia nativa: `ca37f7480d93ca5651facf048ab1ffb978d499c1`, documentación/mediciones.
- El remoto main se observó en ca37f74 al comenzar esta etapa. No fue necesario
  volver a publicar esos commits; no se atribuye la publicación a una operación
  que no ejecutó este proceso.
- Las 69 fuentes coinciden con los hashes crudos medidos y los blobs del commit.
  Cargo.toml de desktop solo presenta diferencia CRLF/LF entre copia y Git.
  No se cambió código, dependencia, lockfile, binding ni callback.
- Las suites locales de 13.4.a y la matriz física de 13.4.b corresponden a ese
  mismo contenido. Los manifiestos previos conservan su estado histórico.

## CI de la revisión exacta

[Run 43975724679](https://github.com/YoZaiKK/AudioMOTORR/actions/runs/43975724679) verificado mediante la API oficial de GitHub,
con head_sha igual a ca37f7480d93ca5651facf048ab1ffb978d499c1, evento push.
Última consulta registrada: 2026-11-16T23:14:05.568Z.

| Job                                                                                                 | Estado    | Conclusión |
| --------------------------------------------------------------------------------------------------- | --------- | ---------- |
| [headless-linux](https://github.com/YoZaiKK/AudioMOTORR/actions/runs/43975724679/job/133679062950)  | completed | success    |
| [desktop-windows](https://github.com/YoZaiKK/AudioMOTORR/actions/runs/43975724679/job/133679063340) | completed | success    |

Linux cubre los cinco paquetes headless, formato, Clippy y nueve escenarios CLI.
Windows comprueba bindings deterministas, tipos/lint/frontend, contratos mock IPC,
workspace, audio instrumentado, diagnósticos/memoria, Clippy y build desktop debug.
La CI comprueba que locks y bindings no cambien. No abre hardware ni confirma escucha.
Ambos jobs de esta revisión finalizaron correctamente.

## Paquete local revisable

`target/stage13-4c-20261116/AudioMOTORR-stage13-preview-win-x64-ca37f74.zip`

- Tamaño: 6096253 bytes. SHA256:
  `e292548d2239c28ec815e74826bf82775f9829bbd34a9e4e0d6b546bb32af830`.
- App Windows x64 en **debug**, recursos web embebidos, sin instalador.
- CLI **release con rt-diagnostics**, mismo binario que la medición nativa.
- Proyecto portátil v2 con cuatro WAV, escenas 0/7/8, patrón y mezcla; escena
  inicial 7. Abrir/inicializar conserva filas vacías hasta un lanzamiento explícito.
- Dos capturas reales, instrucciones, manifiesto y checksums. No se generaron
  imágenes simuladas ni se presenta el paquete como un release público.
- Trece archivos inventariados; doce entradas en SHA256SUMS, excluyendo al propio
  listado para evitar autorreferencia.

Se comprimió y extrajo en otra carpeta, con espacios en la ruta; todos los hashes
coinciden. La CLI extraída valida el proyecto y renderiza la escena 7 con bloque
4096, idéntica byte a byte al WAV generado antes con bloque 127. RIFF float32
estéreo a 48 kHz, 18 notas, 90531 frames musicales + 5862 de cola = 96393 frames;
muestras finitas y señal no nula. No se abrió audio durante estas verificaciones.

[Guía del paquete](../howToRunThis/12-entrega-preview.md) y
[evidencia de entrega](evidencia-entrega-clips-13.4c.json).

## Alcance y pendiente

La [matriz nativa](etapa-13-medicion-nativa-clips.md) tiene seis casos y seis
reaperturas correctos en Realtek/WASAPI, 48 kHz estéreo f32; cero memoria/no finitos/
deadline misses, máximo 145.9 µs y carga nominal 1.459 %. La escucha sigue
pendiente; picos, permisos para avanzar y silencio en una conversación no la prueban.
No se declara cerrada 13.4.b ni toda la etapa 13 sin esa observación.

No se midieron latencia física, continuidad acústica, loopback, unplug, estrés ni
otros dispositivos/plataformas. No hay grabación, MIDI externo, plugins,
resampling/time stretching, timeline, piano roll ni automatización.

Pendiente inmediato: registrar la observación de escucha para cerrar 13.4.b y la etapa 13 dentro del alcance probado.
