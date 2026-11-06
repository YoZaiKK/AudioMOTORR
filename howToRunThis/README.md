# Cómo ejecutar y comprobar AudioMOTORR

Guía en español para la base actual: motor de silencio, transporte y servicio de
audio. Los comandos se ejecutan desde la raíz del repositorio en PowerShell.
El README principal se mantiene en inglés.

| Quiero…                                             | Guía                                         |
| --------------------------------------------------- | -------------------------------------------- |
| Preparar Windows y las herramientas                 | [Preparación](01-preparacion.md)             |
| Recorrer los controles con capturas                 | [Walkthrough de la interfaz](02-interfaz.md) |
| Procesar frames sin dispositivo o probar audio real | [CLI](03-cli.md)                             |
| Ejecutar tests, bindings y comprobaciones           | [Pruebas](04-pruebas.md)                     |
| Resolver errores del entorno                        | [Problemas conocidos](05-problemas.md)       |

Ruta rápida:

```powershell
npm ci
npm run doctor
node howToRunThis/run-cli-cases.mjs
npm run desktop:dev
```

Inicializar audio abre una salida; Play inicia el transporte. **La salida contiene
silencio**. No hay todavía instrumentos, secuenciador, guardar/abrir proyectos ni
exportación WAV. No se espera escuchar un tono al pulsar Play.

Los [casos JSON](test-files/cli-cases.json) son datos para el ejecutor de pruebas,
no proyectos que la aplicación pueda importar. La [salida de referencia](test-files/expected-output.txt)
es ilustrativa; el ejecutor imprime los resultados reales de cada ejecución.

Las siete capturas son de WebView2 real y un dispositivo real. Sus fechas y
configuración están en [metadata.json](screenshots/metadata.json). Los contadores
varían entre ejecuciones; las capturas no demuestran continuidad audible ni ausencia
de dropouts. Las plataformas Linux/macOS siguen sin verificación local.

Estado y evidencia de etapa 5: [registro de progreso](../progress/detalles.md).

La etapa 6 añade documentación técnica en inglés: [arquitectura](../docs/architecture.md),
[contrato de tiempo real](../docs/realtime-contract.md), [decisiones](../docs/adr/0001-core-and-desktop-boundaries.md),
[desarrollo y CI](../docs/development.md) y [roadmap](../docs/roadmap.md).
Ver [cierre de etapa 6](../progress/cierre-etapa-6.md) para la auditoría final y
los resultados locales, y [continuidad actual](../progress/continuidad-etapa-6.md)
para el relevo. Workflow validado localmente no significa CI remota satisfactoria;
consultar los [resultados de Actions](https://github.com/YoZaiKK/AudioMOTORR/actions/workflows/ci.yml)
de la revisión publicada.
