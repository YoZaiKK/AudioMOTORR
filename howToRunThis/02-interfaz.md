# Walkthrough de la interfaz

## 1. Navegador y escritorio

```powershell
npm run dev
```

Abrir http://127.0.0.1:1420. Debe mostrarse «Vista de interfaz: requiere Tauri para
audio», con operaciones nativas deshabilitadas. No hay dispositivos simulados.
Detener Vite con Ctrl+C antes del siguiente comando, que usa el mismo puerto:

```powershell
npm run desktop:dev
```

La ventana muestra el proyecto, dispositivos y transporte. Inicialmente el servicio
está sin inicializar y Play está deshabilitado. El montaje de React no abre audio.

![Servicio sin inicializar](screenshots/01-sin-inicializar.png)

## 2. Error de inicialización y recuperación

Si no hay salida disponible, inicializar debe mostrar un error y mantener el
transporte deshabilitado. El recorrido automatizado usa un ID inexistente mediante
IPC para probar esta ruta de manera reproducible; no hay un botón que fabrique
errores en la interfaz ni es necesario desconectar físicamente el dispositivo.

![Error de dispositivo inexistente](screenshots/02-error-dispositivo.png)

Seleccionar salida predeterminada o un dispositivo enumerado y pulsar
**Inicializar audio**. Esperar servicio `ready`. Un contador de callbacks mayor
que cero confirma procesamiento; `ready` por sí solo confirma apertura del stream.

## 3. Play y BPM

Pulsar Play, esperar «Reproduciendo», escribir `137.5` en BPM y pulsar Aplicar BPM.
Esperar «Aplicado: 137.5 BPM». La posición y los frames del dispositivo avanzan.
Un comando aceptado puede aparecer como pendiente antes de aplicarse; no se cambia
optimistamente el estado observado. Con hardware rápido ese pendiente puede ser breve.

![Play y BPM aplicados](screenshots/03-reproduciendo.png)

## 4. Pause

Pulsar Pause y esperar «Pausado». La posición musical se conserva, mientras el
dispositivo sigue procesando silencio. Play reanuda desde esa posición.

![Pause conserva la posición](screenshots/04-pausado.png)

## 5. Stop

Pulsar Stop y esperar «Detenido». La posición vuelve a cero; el contador de frames
del dispositivo no se reinicia y el stream sigue abierto.

![Stop reinicia la posición musical](screenshots/05-detenido.png)

## 6. Cerrar y reiniciar el audio

Pulsar Cerrar audio: el servicio vuelve a `uninitialized`. Volver a inicializar:
el motor comienza detenido, con contadores nuevos y el tempo del documento
conservado. Para cambiar de dispositivo, cerrar primero la salida actual.

![Reinicio con tempo conservado y callbacks](screenshots/06-reiniciado.png)

Cerrar audio otra vez: Play, Pause, Stop y BPM quedan deshabilitados. Cerrar la
ventana termina el publicador y el servicio; no se guarda el proyecto en disco.

![Servicio cerrado](screenshots/07-audio-cerrado.png)

## Regenerar estas capturas

```powershell
npm run check
npm run desktop:build -- --debug --no-bundle
powershell -NoProfile -File scripts/desktop-smoke.ps1 -UpdateDocs
```

El modo UpdateDocs reemplaza explícitamente los PNG y metadata.json de esta guía.
Sin ese switch, el smoke solo guarda su captura local en target/desktop-stage4.png.
Requiere hardware, WebView2 y el puerto local 9223 libre; detecta el puerto ocupado
antes de abrir la aplicación. El script restaura sus variables de entorno y cierra
la ventana. No ejecutar comandos Cargo entre build y smoke: pueden sustituir el
ejecutable empaquetado por uno que espera el servidor de desarrollo.
