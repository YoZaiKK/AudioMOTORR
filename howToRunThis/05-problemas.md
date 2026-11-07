# Problemas conocidos

| Síntoma                                       | Acción                                                                                                                                                         |
| --------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Vite indica puerto 1420 ocupado               | Detener el npm run dev anterior antes de desktop:dev. No se cambia el puerto silenciosamente.                                                                  |
| Puerto 9223 ocupado                           | Cerrar la sesión de depuración anterior antes del smoke. El script no debe conectarse a otra aplicación.                                                       |
| No se encuentra el destino WebView2 del smoke | Volver a compilar con desktop:build inmediatamente antes del smoke; no basta un binario generado por cargo test.                                               |
| Falta MSVC, SDK o WebView2                    | Ejecutar doctor y completar los prerrequisitos de Windows; no confundir un build frontend correcto con un build desktop.                                       |
| DeviceAbsent                                  | Actualizar dispositivos, seleccionar una salida disponible y reintentar. El transporte debe seguir deshabilitado mientras el servicio no esté ready.           |
| QueueFull o Busy                              | No se ha confirmado la acción rechazada. Esperar y consultar el estado antes de reintentar.                                                                    |
| Comando aceptado pero pendiente               | Esperar el snapshot aplicado; si no hay callbacks, cerrar y reiniciar el audio. No asumir que Play ya se aplicó.                                               |
| No se oye sonido                              | Es lo esperado: el motor actual escribe silencio. Verificar callbacks y frames en el panel técnico.                                                            |
| Access is denied en tests Tauri               | El mock runtime puede necesitar rutas de datos fuera del workspace. En entornos restringidos solicitar ejecución autorizada; no quitar las comprobaciones ACL. |
| Cargo falla con Schannel SEC_E_NO_CREDENTIALS | Usar el mecanismo de aprobación del entorno para la descarga. No desactivar TLS.                                                                               |
| npm falla con EPERM sobre su caché            | La caché está fuera del workspace; solicitar acceso mediante el entorno.                                                                                       |
| Bindings desactualizados                      | Revisar DTO Rust, regenerar explícitamente y comprobar el diff; no editar api.ts a mano.                                                                       |
| Test de hardware aparece ignored              | Ejecutarlo explícitamente solo en un host con audio. La suite ordinaria no depende del dispositivo.                                                            |

export-wav no sobrescribe archivos: si el destino existe, elegir otro nombre.
Si falla después de crearlo puede quedar un WAV parcial; la ruta figura en el
error. Revisarlo/quitarlo explícitamente antes de repetir. El padre debe existir.
Un fallo de límite RIFF o byte rate se resuelve reduciendo frames/sample rate,
no actualizando lockfiles ni abriendo un dispositivo.

Los tests de integración Windows usan manifiesto Common Controls v6. Si reaparece
STATUS_ENTRYPOINT_NOT_FOUND, revisar el build.rs y el manifiesto de los tests;
no copiar DLL de sitios externos ni deshabilitar pruebas.

El screenshot de un contador no prueba ausencia de dropouts. La continuidad se
medirá con una señal en un hito posterior. No hay persistencia: cerrar la app
descarta el documento actual; cerrar/reabrir solo el servicio conserva su tempo.
