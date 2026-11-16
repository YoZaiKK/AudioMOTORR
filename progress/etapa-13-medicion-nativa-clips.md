# Etapa 13.4.b — Medición nativa de clips y escenas

Fecha: 2026-11-16 (México). **Matriz física y reaperturas correctas;
confirmación de escucha pendiente.** La etapa 13 sigue abierta.

El commit solicitado, `03055988a37faf68af7d1cd0bd3b02315af6799b`, contiene
la implementación acumulada hasta 13.4.a: 111 archivos, incluidos código,
contratos, pruebas, diagnósticos y capturas reales. No se hizo push ni se ejecutó
CI de ese SHA. Los manifiestos anteriores conservan sus estados históricos.

## Identificación y alcance

- Release con `rt-diagnostics`; binario verificado de 13.4.a copiado antes de medir.
- Windows 10.0.26200 x64, Intel Core i7-1185G7.
- WASAPI, **Speakers (2- Realtek(R) Audio)**, salida exacta **48000 Hz / 2 canales / f32**,
  sin fallback. El dispositivo enumera salida estéreo; mono en la matriz significa
  fuentes WAV mono, no salida física mono.
- Driver Realtek Audio 6.0.9954.2, fecha reportada 2026-02-11; componentes Universal
  Service 1.0.960.0 y Effects 13.5209.2429.786. La consulta WMI necesitó salir del
  sandbox por acceso denegado; quedó registrada antes de completar la matriz.
- Ocho WAV generados: kick/snare/hat/accent en mono y estéreo. SHA256 y metadatos
  verificados antes y después de ejecutar; amplitudes del generador sin cambios.

Las 69 fuentes/configuraciones de la evidencia 13.4.a coinciden con el working
tree y con el contenido del commit. Solo Cargo.toml de desktop presenta diferencia
CRLF/LF entre bytes del working tree y blob Git; el contenido es idéntico.
No se cambió fuente de producto, dependencia, lockfile, binding ni callback.

## Matriz física

Cada caso inicializa y reabre el mismo dispositivo, con IDs de servicio crecientes,
IDs de motor reiniciados y filas vacías. Comprueba warmup detenido, timeline,
pausa/ediciones, escena parcial, Keep/supersesión, cambio de BPM, sample ausente,
mute con notas en marcha, StopAll/colas naturales, Stop y cierre a Uninitialized.
Play en la reapertura permanece sin clips hasta un lanzamiento explícito.

| WAV fuente | Cuantización | Clips iniciales | BPM    | Máximo inicial | Máximo reapertura | Mayor carga nominal |
| ---------- | ------------ | --------------- | ------ | -------------- | ----------------- | ------------------- |
| Mono       | Step         | Empty           | 95.5   | 68000 ns       | 60900 ns          | 0.680 %             |
| Mono       | Beat         | Beat            | 127.25 | 100000 ns      | 79200 ns          | 1.000 %             |
| Mono       | Bar          | Dense           | 300    | 122200 ns      | 82900 ns          | 1.222 %             |
| Estéreo    | Step         | Dense           | 300    | 99300 ns       | 85300 ns          | 0.993 %             |
| Estéreo    | Beat         | Empty           | 95.5   | 89900 ns       | 77600 ns          | 0.899 %             |
| Estéreo    | Bar          | Beat            | 127.25 | 145900 ns      | 96200 ns          | 1.459 %             |

**Seis casos con código 0 y seis reaperturas verificadas**, doce sesiones:
2283 callbacks y 1102752 frames medidos, callbacks observados de 480..1056 frames.
Máximo de procesamiento: **145900 ns (145.9 µs)**; mayor ratio nominal: **1.459 %**.
Mayor cota superior p99 de histograma: 131071 ns. Cero deadline misses,
asignaciones, bytes asignados, liberaciones, bytes liberados y muestras no finitas.
Todas las sesiones completas observaron señal; picos entre 0.0011054969 y 0.0062017236.

Los tiempos cubren procesamiento completo del callback, conversión y publicación
según la instrumentación existente. Son acumulados de cada sesión, incluido el
warmup silencioso del runner y los controles posteriores; no son tiempos de una
sola escena ni incluyen una medida de latencia física del dispositivo.

Se corrigió una descripción de 13.4.a: **no se excluyen 64 callbacks de las métricas
nativas**. El runner espera 66 como estabilización; la instrumentación acumula
todos. Publicar el snapshot ocurre antes de registrar la duración del callback
actual, por lo que sus métricas de procesamiento pueden ir un callback detrás
del contador publicado. Esta diferencia no significa pérdida de callbacks.

## Procedencia y límites

Una primera ejecución de Mono/Step/Empty terminó con marcador OK y dos líneas
de métricas, pero el recolector falló al buscar la metadata WMI aún no disponible.
Se conserva su log íntegro; su código de salida y hora de inicio no quedaron
persistidos y se registran como desconocidos. Se repitió ese caso una vez para
obtener procedencia completa, no para elegir tiempos favorables. Sus dos sesiones
iniciales se conservan aparte y no se suman a las doce sesiones de la matriz.
Después del ajuste, cada ejecución guarda primero su resultado bruto y luego
actualiza el agregado, sin sobrescribir ensayos anteriores.

La escucha se solicitó al usuario y sigue pendiente de una respuesta explícita.
Los picos no prueban que el usuario oyera audio. No se midieron loopback,
continuidad acústica, latencia del driver, estrés externo ni desconexiones; tampoco
otras plataformas/dispositivos o salida física mono. No se marca 13.4.b completa
ni se inicia el cierre 13.4.c hasta registrar la observación de escucha.

El [manifiesto nativo](evidencia-nativa-clips-13.4b.json) conserva el SHA del commit,
huella de fuentes y binario, driver/formato, WAVs, argumentos, fases, métricas y
logs. Archivos locales en `target/stage13-4b-20261116/`: `matrix.json`,
`summary.json`, `driver.json`, `devices.log`, logs individuales y resultados brutos.
Se verifican los hashes, enlaces/documentación y formato. Las suites/build de
13.4.a siguen correspondiendo a estas mismas fuentes; no se repiten pruebas de
producto por cambios solo documentales.

Pendiente inmediato: **confirmar escucha de estas secuencias** o registrar una
repetición si no pudieron escucharse. Después: **13.4.c, entrega/CI y cierre**.
