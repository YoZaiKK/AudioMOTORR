# Escuchar y controlar el mezclador

La integración desktop 10.3.c y su
[aceptación local 10.3.d](../progress/cierre-integracion-mezclador-10.3d.md)
están completadas. La comprobación
física y la audibilidad del mezclador quedaron registradas en el
[cierre de etapa 10](../progress/cierre-etapa-10.md), dentro de su alcance; ver el
[estado](../progress/estado-y-siguiente-etapa.md) y el
[contrato desktop](../docs/mixer-desktop.md).

1. Ejecuta `npm run desktop:dev`. El navegador solo muestra una vista deshabilitada.
2. Con audio cerrado, usa las cuatro tiras para preparar ganancia (0–1), pan
   (−1 izquierda / +1 derecha) y mute. Pulsa **Aplicar canal** para seleccionar
   cada ajuste. Abre **Rutas de los 16 slots** para elegir sus destinos.
3. Carga un banco WAV con el procedimiento del [sampler](06-sampler.md).
   Cargar y editar ajustes no abren la salida ni activan Play.
4. Selecciona la salida del sampler y pulsa **Inicializar con mezcla**.
   La inicialización consume el banco preparado, incluso si falla. Si falla,
   revisa el error y vuelve a preparar el banco antes de intentarlo otra vez.
5. Pulsa **Play** y **Disparar sample** en un slot cargado. Ajusta una tira y
   pulsa **Aplicar canal** para escuchar el cambio. Las rutas nuevas afectan
   a los siguientes disparos; las voces que ya suenan conservan su canal.
6. Compara **Seleccionado**, el recibo del último cambio y **Observado**.
   «Aceptado» significa encolado; «procesado» todavía puede mostrar frames de
   rampa pendientes. Los medidores muestran la última ventana de audio observada,
   con su secuencia y número de frames. Mono muestra un nivel, estéreo L/R.
7. **Stop** conserva el banco. **Cerrar audio** libera la sesión y conserva los
   ajustes seleccionados en memoria para la siguiente inicialización. Guardar
   esos ajustes en disco corresponde a la etapa 12.

Si hay cola llena o timeout, conserva el borrador y consulta el error. Usa
**Consultar ajustes de mezcla** para recuperar la selección del documento y
compara el estado observado antes de reintentar. Un cambio sin confirmación al
cerrar/perder la sesión se muestra como desconocido. El sampler original sigue
disponible con **Inicializar sampler**; para usar las tiras, abre una sesión con
mezcla explícitamente.
