# Sampler: cargar WAV y disparar samples

Actualizado 2026-11-09, etapa 9.4.c. Flujo implementado con pruebas de Application,
IPC mock y frontend; esta entrega no realizó audición física ni creó capturas.
Las imágenes anteriores conservan su fecha y muestran versiones anteriores.

La [auditoría 9.4.d](../progress/cierre-integracion-9.4.md) añade recuperación de
consulta del banco cuando falla enumerar dispositivos y evita polls automáticos
solapados al cancelar/recargar. No añade mediciones físicas. El
[protocolo nativo](../docs/sampler-native-verification.md) describe la verificación
física 9.5.b; los [diagnósticos 9.5.a](../docs/sampler-diagnostics.md) ya están
implementados y probados sin dispositivo.

1. Ejecuta npm run desktop:dev. Si hay audio inicializado, pulsa **Cerrar audio**.
   Cargar un banco requiere servicio sin inicializar; si falló el stream, ciérralo.
2. En **Sampler · WAV one-shot**, elige frecuencia de salida y mono/estéreo.
   Usa una configuración anunciada por tu dispositivo. El sampler exige ese
   rate y número de canales exactos al abrirlo; no convierte frecuencia.
3. Escribe una ruta local WAV en slot 0. **Añadir slot** permite hasta 16 rutas;
   las vacías se omiten. Admite PCM16 o float32, clásico/extensible compatible,
   mono/estéreo, hasta 1048576 frames y 16 MiB de archivo por recurso.
   Mono puede salir por estéreo; estéreo no se reduce a mono. Ver
   [formatos y límites completos](../docs/sample-loading-contract.md).
4. Pulsa **Cargar banco WAV**. No abre el dispositivo ni reproduce sonido.
   **Cancelar / descartar banco** cancela carga o libera un resultado listo.
   Cancelar espera la siguiente comprobación del worker: no interrumpe un read
   bloqueado en el sistema operativo.
5. Cuando indique **Banco listo**, selecciona **Salida del sampler** y pulsa
   **Inicializar sampler**. Consume el banco incluso si falla la apertura;
   en ese caso vuelve a cargar. No hay cambio automático a otro formato/dispositivo.
6. Pulsa **Play**, selecciona un slot cargado y su ganancia (0–1), y pulsa
   **Disparar sample**. Puedes probar un disparo antes de Play: se acepta en la
   cola, pero al procesarlo se rechaza como notPlaying. No empieza a sonar después.
7. Observa el resultado por ID: pendiente, voz iniciada, rechazado o desconocido
   por observación perdida/cierre. El contador de procesados no confirma que un
   disparo empezó. Voz iniciada tampoco demuestra sonido físico audible.
8. **Aplicar volumen** controla el master con rampa de 5 ms. Hay 16 voces;
   con todas ocupadas el siguiente disparo se rechaza (poolFull). Headroom fijo
   1/16 evita clipping de la suma en el contrato actual.
9. Pause congela voces tras release; Play continúa. Stop limpia voces tras release
   y conserva PCM. **Cerrar audio** libera el banco y permite cargar otro.

No hay guardado de rutas/bancos, secuenciador, importación de proyectos o exportación
del sampler. La CLI exporta senoide. En navegador los controles nativos están
deshabilitados. Las etapas 10–13 incorporarán mezclador, secuenciador,
persistencia/exportación del beat y clips/escenas, en ese orden.

Ver [contrato desktop](../docs/sampler-desktop.md),
[pruebas](04-pruebas.md) y [reporte de implementación](../progress/etapa-9-aplicacion-sampler.md).
