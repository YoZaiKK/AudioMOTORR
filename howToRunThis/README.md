# Cómo ejecutar y comprobar AudioMOTORR

Actualización 2026-11-10: 9.3.a fija el
[contrato de carga WAV](../docs/sample-loading-contract.md) y los presupuestos;
9.3.b ya carga un recurso individual desde Rust con preflight/decodificación
acotados y cancelación. 9.3.c prepara bancos con permiso compartido y presupuesto
agregado; 9.3.d verifica fallos/cancelación y cierre entre hilos.
9.3.e ya verifica fixtures WAV → banco → Sampler offline y 9.3.f cierra
preparación localmente. 9.4.a integra sampler en AudioEngine, verificado offline.
9.4.b conecta sesiones/outcomes y 9.4.c añade Application, DTO/IPC y UI de sampler,
con carga/cancelación en worker y pruebas mock. 9.4.d cierra la integración
local y corrige recuperación de dispositivos/polls solapados. 9.5.a implementa
diagnósticos sampler y benchmark sin CPAL; siguiente verificación física 9.5.b. Ver
[reporte actual](../progress/etapa-9-diagnosticos-sampler.md) y
[guía del sampler](06-sampler.md); no se añade evidencia física en esta entrega.

Guía en español para la app actual: silencio, senoide nativa explícita con rampas,
sampler WAV one-shot, transporte, servicio de audio y exportación de senoide WAV offline. Los comandos se ejecutan
desde la raíz del repositorio en PowerShell.
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

Inicializar audio abre una salida; Play inicia el transporte. **Silencio es la
fuente predeterminada**. Seleccionar Senoide antes de inicializar permite reproducir
un tono, ajustar frecuencia y aplicar volumen con rampas de 5 ms. Ver
[walkthrough](02-interfaz.md). El [sampler](06-sampler.md) permite cargar un banco
y dispararlo explícitamente. No hay secuenciador ni guardar/abrir proyectos. La CLI permite exportar una senoide offline; ver [export-wav](03-cli.md).

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

Plan vigente (actualizado 2026-11-08): [etapas 7–13](../progress/plan-etapas-7-13.md).
La etapa 7 está cerrada: oscilador y exportación CLI publicados en c6cf627,
con CI Linux/Windows correcta. Ver [cierre](../progress/cierre-etapa-7.md).
Los smoke ordinarios conservan silencio; export-wav escribe señal a archivo. La
[evidencia del incremento](../progress/etapa-7-oscilador.md) confirma la base CI
en Ubuntu headless y Windows desktop, sin validar Linux nativo/macOS.
Esta guía describe las funciones implementadas; sus nueve fixtures Node siguen
siendo de silencio y los tests Rust cubren WAV. Ver [reporte WAV](../progress/etapa-7-exportacion.md).
Etapa 8 completada: reproducción nativa, rampas y mediciones release publicadas en
1e6bca8, con CI Linux/Windows correcta para ese SHA en run 42524752652.
Ver [reporte de etapa 8](../progress/etapa-8.md) y
[contrato técnico](../docs/native-audition.md), con estado de CI y límites explícitos.

Etapa 9 en curso: 9.1 aporta recursos PCM y 9.2 reproduce one-shots con un banco
de 16 slots y 16 voces en la biblioteca del motor. Todavía no hay carga de archivos
ni reproducción de samples en la app de escritorio. Ver
[contrato del sampler](../docs/sampler-contract.md) y
[progreso del kernel](../progress/etapa-9-voces.md). 9.3.b–c ya decodifican WAV
y preparan bancos completos fuera del procesamiento. El siguiente incremento
es 9.4.c: Application/DTO/IPC/UI, tras conectar sesiones en 9.4.b; ver
[reporte actual](../progress/etapa-9-sesiones-sampler.md).

9.2 está publicada en 7acc1db y pasó
[CI Linux/Windows, run 42530699397](https://github.com/YoZaiKK/AudioMOTORR/actions/runs/42530699397).
La etapa 9 completa sigue abierta; esa CI comprueba el kernel y el escritorio
actual sin abrir hardware. El reporte separa sus resultados de las pruebas release locales.
