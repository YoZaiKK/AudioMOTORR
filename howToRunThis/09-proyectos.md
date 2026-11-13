# Guardar, abrir y exportar un beat

La etapa 12 añade proyectos JSON v1 y exportación del beat completo. Consulta
el [contrato](../docs/project-format.md) y el
[reporte](../progress/etapa-12-persistencia-exportacion.md). Estas operaciones
requieren la aplicación de escritorio; el modo navegador es una vista de interfaz.

## Crear y guardar

1. Carga tus WAV en los slots del sampler y edita las cuatro filas de pasos.
   El proyecto conserva el patrón, BPM seleccionado, gain/pan/mute de cuatro
   canales y rutas de los 16 slots. Los archivos originales deben seguir
   accesibles al guardar; el PCM retenido para reproducir no es un archivo fuente.
2. En **Proyecto y exportación**, escribe el título y una ruta de archivo,
   por ejemplo `C:\Music\mi-beat.json`. La carpeta padre debe existir.
3. Pulsa **Guardar proyecto**. Se crea una carpeta de samples junto al JSON y
   se guardan referencias relativas. Puedes mover la carpeta del proyecto
   completa a otra ubicación. El título se confirma junto con el guardado.
4. Guardar de nuevo sobre el proyecto actual lo actualiza. Para sobrescribir
   otro archivo debes marcar **Reemplazar otro archivo existente**. Cambiar la
   ruta permite guardar una copia. Se conservan carpetas de assets anteriores
   para no romper otros documentos que las referencien; no se borran automáticamente.

Guardar puede hacerse durante la reproducción y toma el documento seleccionado,
incluidas ediciones aceptadas aún pendientes de aplicar al motor. No cambia el
audio en curso. **Consultar proyecto** actualiza el estado sin borrar tus
borradores de título/ruta. El indicador distingue cambios pendientes y guardado.

## Abrir y volver a reproducir

1. Cierra audio y cancela/descarta cualquier preparación pendiente.
2. Indica la ruta del JSON y pulsa **Abrir proyecto**. Se valida todo antes de
   sustituir el documento. Versiones desconocidas, archivos inválidos y samples
   ausentes muestran un error y conservan la selección anterior.
3. Revisa los samples en el desplegable del proyecto. Elige frecuencia y canales
   compatibles con sus WAV y pulsa **Preparar samples del proyecto**.
4. Cuando el banco esté listo, selecciona **Inicializar secuenciador** en el
   sampler y después **Play**. Abrir y preparar no abren audio automáticamente.

El proyecto no guarda dispositivo, posición de reproducción, voces, medidores,
IDs de comandos ni volumen master transitorio. Los samples no se remuestrean;
su frecuencia debe coincidir con la salida. Mono se admite en salida estéreo;
una fuente estéreo no se convierte a salida mono.

## Exportar el beat

Indica un WAV nuevo, compases, frecuencia/canales y el final deseado. Pulsa
**Exportar beat**. La operación usa un motor independiente, con el patrón,
mezcla y BPM seleccionados, y mantiene la reproducción actual.

- **Completar colas de samples:** termina los compases, deja de programar notas
  y completa los samples activos. La duración puede superar los compases.
- **Cortar al final de los compases:** escribe exactamente los frames musicales,
  sin añadir cola ni fundido automático.

El resultado es float32 y no sobrescribe un WAV existente. Los frames musicales,
de cola y totales se muestran con los contadores de notas. Si se llenan las 16
voces se aplica la misma política de rechazo de notas nuevas que en reproducción.
Un sample ausente utilizado por el patrón impide crear el WAV. Los errores de
escritura/finalización informan si puede quedar un archivo parcial.

Desde PowerShell, sin abrir audio ni crear un servicio vivo:

```powershell
cargo run -p daw-cli --no-default-features --locked -- project-check --project C:\Music\mi-beat.json
cargo run -p daw-cli --no-default-features --locked -- export-beat --project C:\Music\mi-beat.json --output C:\Music\mi-beat.wav --bars 4 --tail finish --block-size 127 --sample-rate 48000 --channels 2
```

Las [capturas normal](../docs/screenshots/stage12-project-wide.png) y
[estrecha](../docs/screenshots/stage12-project-narrow.png) documentan el flujo
real de WebView2. La prueba guardó, abrió y exportó sin inicializar audio;
no es una observación auditiva.
