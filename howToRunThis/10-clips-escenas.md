# Clips y escenas

Ejecuta `npm run desktop:dev` o la aplicación compilada. En navegador puedes ver
los controles; las operaciones nativas requieren Tauri.

1. Cierra el audio y selecciona **Clips y escenas** en «Modo del proyecto».
2. Selecciona una celda de la matriz y pulsa **Crear clip vacío**. Los índices de
   fila/clip/sample empiezan en cero; los pasos del editor se muestran del 1 al 16.
3. Activa pasos y elige el sample. Usa el panel WAV para preparar un banco con ese
   slot, o abre un proyecto y prepara sus samples. Un clip puede referenciar un
   sample todavía no cargado; las notas lo reportarán como ausente.
4. Elige la escena que vas a editar. Cada fila puede Conservar, Detener o Lanzar
   un clip existente. Para eliminar un clip referenciado, cambia primero esas
   acciones a Conservar/Detener.
5. Selecciona la cuantización de los próximos lanzamientos: paso, pulso o compás.
   Los cambios no alteran solicitudes ya aceptadas.
6. Con el banco listo, pulsa **Inicializar proyecto (clips)**. Queda detenido y
   sin clips activos. Pulsa **Lanzar escena** o **Lanzar clip seleccionado** y Play.
7. Consulta las filas observadas: clip activo, acción en cola, paso global e ID
   de motor y última transición. «Procesada» confirma que se procesó la solicitud;
   la transición puede seguir esperando su límite cuantizado. La posición al
   siguiente frame es una proyección del snapshot, sin reloj animado de navegador.
8. Editar un clip activo cambia el documento. Lánzalo de nuevo para que el motor
   tome esa versión. Detener una fila o todas las filas respeta la cuantización
   y permite terminar las colas de las voces existentes. Stop del transporte y
   Cerrar audio conservan las reglas de transporte y liberación habituales.
9. Selecciona **Escena inicial guardada / exportación** y guarda el proyecto. Esta
   escena determina la exportación en modo Clips. Elegir o lanzar otra escena en
   vivo no cambia ese valor. En modo Patrón se exporta el patrón.

Abrir o reinicializar no reproduce el historial de lanzamientos. Un resultado
«desconocido» o un timeout requiere consultar el estado antes de volver a lanzar;
la interfaz no reintenta por su cuenta. Un banco de preparación se consume al
inicializar, incluso ante ciertos fallos: prepara de nuevo cuando corresponda.

Para volver al secuenciador de patrón, cierra audio y cambia el modo a Patrón.
El contenido de ambos modos se conserva en el archivo v2. También puedes cambiar
BPM con audio cerrado; se guarda como selección del proyecto.

Ver [proyectos y exportación](09-proyectos.md),
[contrato desktop](../docs/launcher-desktop.md) y
[avance de la etapa](../progress/etapa-13-desktop-clips.md).
