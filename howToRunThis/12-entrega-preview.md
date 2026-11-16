# Entrega local de la vista previa de etapa 13

La revisión publicada que contiene la implementación y evidencia nativa es
`ca37f7480d93ca5651facf048ab1ffb978d499c1`. Los cambios de producto están en
`0305598`; `ca37f74` añade la medición y sus límites. Consultar el
[reporte de entrega/CI](../progress/etapa-13-entrega-ci.md) para el estado actualizado
de Windows/Linux y la confirmación de escucha. La etapa no se cierra por tener
señal en los medidores.

El paquete preparado en esta sesión está disponible localmente en:

```text
target/stage13-4c-20261116/AudioMOTORR-stage13-preview-win-x64-ca37f74.zip
```

No es un instalador ni un archivo adjunto a un GitHub Release. Extraer la carpeta
completa conserva las rutas relativas de sus recursos. Contiene:

| Archivo                           | Uso                                                                  |
| --------------------------------- | -------------------------------------------------------------------- |
| `AudioMOTORR.exe`                 | App Windows x64 en perfil debug, con los recursos web embebidos.     |
| `daw-cli.exe`                     | CLI release con `rt-diagnostics`, el mismo binario medido en 13.4.b. |
| `example/launcher.json`           | Proyecto v2, cuatro samples, patrón y escenas 0/7/8.                 |
| `example/assets/`                 | WAV de 48 kHz estéreo y referencias relativas.                       |
| `example/scene-7.wav`             | Exportación offline de una escena, sin abrir dispositivo.            |
| `MANIFEST.json`, `SHA256SUMS.txt` | Revisión, perfiles y hashes de los archivos.                         |
| `images/`                         | Capturas reales históricas del editor y exportación.                 |

La app compilada necesita Windows x64 con WebView2 Runtime y runtime MSVC
compatibles. El paquete no instala prerrequisitos. Las herramientas Rust/Node/npm
son necesarias para reconstruir desde el repositorio; sus versiones están fijadas.

## Probar el proyecto incluido

1. Abrir `AudioMOTORR.exe`, indicar la ruta completa a `example/launcher.json` en
   Archivo de proyecto y pulsar Abrir proyecto.
2. Mantener 48000 Hz y estéreo. Preparar samples del proyecto, seleccionar una
   salida compatible y pulsar Inicializar proyecto (clips).
3. Elegir escena 7, pulsar Lanzar escena 7 y Play. Abrir/inicializar no lanza
   automáticamente esa escena inicial.
4. Escena 8 cambia dos filas, detiene otra y conserva una. Escena 0 detiene todas
   respetando la cuantización de compás. Stop reinicia el transporte; Cerrar audio
   libera la sesión.
5. Para escuchar ediciones de un clip ya activo, lanzarlo de nuevo. La escena
   inicial guardada sigue siendo 7 y determina la exportación, sin depender del
   último lanzamiento en vivo.

La CLI permite comprobar o exportar el mismo ejemplo sin abrir audio:

```powershell
.\daw-cli.exe project-check --project .\example\launcher.json
.\daw-cli.exe export-beat --project .\example\launcher.json --scene 7 --output .\otra-exportacion.wav --bars 1 --tail finish --block-size 127 --sample-rate 48000 --channels 2
```

Se validó el ZIP tras extraerlo a otra carpeta, con rutas que contienen espacios:
los hashes coinciden y el render con bloque 4096 es idéntico al de bloque 127.
La escena incluye 18 notas, 90531 frames musicales y 5862 de cola, total 96393;
todas sus muestras son finitas y hay señal no nula. Esto comprueba el archivo
exportado, sin sustituir la confirmación de escucha nativa.

Guardar o exportar a un destino ya existente requiere la acción explícita
correspondiente; la CLI no sobrescribe WAV. Para más detalles, ver
[clips/escenas](10-clips-escenas.md), [proyectos](09-proyectos.md) y
[protocolo nativo](../docs/launcher-native-verification.md).
