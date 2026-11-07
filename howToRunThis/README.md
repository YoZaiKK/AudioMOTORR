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

Inicializar audio abre una salida; Play inicia el transporte. **La salida desktop
contiene silencio**. No hay instrumentos en la UI, secuenciador ni guardar/abrir
proyectos. No se espera un tono al pulsar Play. La CLI sí permite exportar una
senoide a WAV offline; ver [comando export-wav](03-cli.md).

Los [casos JSON](test-files/cli-cases.json) son datos para el ejecutor de pruebas,
no proyectos que la aplicación pueda importar. La [salida de referencia](test-files/expected-output.txt)
es ilustrativa; el ejecutor imprime los resultados reales de cada ejecución.

Las siete capturas son de WebView2 real y un dispositivo real. Sus fechas y
configuración están en [metadata.json](screenshots/metadata.json). Los contadores
varían entre ejecuciones; las capturas no demuestran continuidad audible ni ausencia
de dropouts. Linux nativo/macOS siguen sin verificación; Ubuntu headless dispone
de evidencia remota de la base publicada, indicada más abajo.

Estado y evidencia de etapa 5: [registro de progreso](../progress/detalles.md).

La etapa 6 añade documentación técnica en inglés: [arquitectura](../docs/architecture.md),
[contrato de tiempo real](../docs/realtime-contract.md), [decisiones](../docs/adr/0001-core-and-desktop-boundaries.md),
[desarrollo y CI](../docs/development.md) y [roadmap](../docs/roadmap.md).
Ver [cierre de etapa 6](../progress/cierre-etapa-6.md) para la auditoría final y
los resultados locales, y [continuidad actual](../progress/continuidad-etapa-6.md)
para el relevo. Workflow validado localmente no significa CI remota satisfactoria;
consultar los [resultados de Actions](https://github.com/YoZaiKK/AudioMOTORR/actions/workflows/ci.yml)
de la revisión publicada.

Plan del siguiente desarrollo (2026-11-07): [etapas 7–13](../progress/plan-etapas-7-13.md).
El oscilador 7.2 y la exportación CLI 7.3 ya están implementados localmente.
Desktop y smoke conservan silencio; export-wav escribe señal a archivo. La
[evidencia del incremento](../progress/etapa-7-oscilador.md) confirma la base CI
en Ubuntu headless y Windows desktop, sin validar Linux nativo/macOS.
Esta guía describe las funciones implementadas; sus nueve fixtures Node siguen
siendo de silencio y los tests Rust cubren WAV. Ver [reporte WAV](../progress/etapa-7-exportacion.md).
La reproducción nativa de señal llegará en etapa 8.
