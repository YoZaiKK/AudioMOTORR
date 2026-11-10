# Revisión de README, Git ignore y configuración

Fecha: 2026-11-10. Revisión sobre 0c346d55907ac4a94a4f25662990002213143b56,
Phase 9.5.a. Al empezar, main y la referencia local origin/main apuntan a ese
commit y el árbol está limpio. Esto identifica el estado local; no se consultó
el servidor ni se inspeccionaron los jobs remotos para ese SHA.

## Cambios

- README en inglés: estado actual resumido, commit vigente, evidencia local
  diferenciada de CI, etapas 9.5.b/9.5.c y 10–13 con aceptación, y tabla de
  configuraciones efectivas. El siguiente trabajo sigue siendo 9.5.b.
- .gitignore incorpora .env/.env._, conserva .env.example/.env._.example y
  excluye coverage y *.tsbuildinfo. Mantiene dependencias/builds/logs y archivos
  generados Tauri fuera de Git. No se ocultan WAV, imágenes o evidencias en general.
- Prettier y ESLint también excluyen coverage; Prettier omite caché TS. Bindings
  generados se conservan en Git y siguen fuera de Prettier para regeneración exacta.
- Continuidad/estado actual identifican la revisión 0c346d5. Los reportes y
  manifiestos de implementación se conservan como evidencia fechada.

No se cambia motor, UI, IPC, dependencias, lockfiles, workflow o configuración
de ejecución. No se abre hardware ni se hace commit/push. El manifiesto 9.5.a
de 119 archivos deja de coincidir solo en .gitignore, .prettierignore y
eslint.config.js: los otros 116 archivos conservan sus hashes. No se modifica
ese manifiesto para atribuirle una ejecución o configuración posteriores.

## Configuraciones verificadas

| Área             | Resultado                                                                                                                                                                                     |
| ---------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Herramientas     | Rust/Cargo 1.90.0, Node 22.23.4 y npm 10.9.9 instalados y coherentes con los pines/rangos. MSVC x64, Windows SDK y WebView2 detectados por doctor.                                            |
| Dependencias npm | Versiones exactas, manifest/lock coincidentes; engine-strict y save-exact activos.                                                                                                            |
| Cargo            | Metadata --locked --offline válida: seis paquetes, cinco defaults sin desktop, edición 2024, MSRV compatible, unsafe_code forbid. No es necesario añadir .cargo/config.toml.                  |
| Frontend/desktop | Vite 127.0.0.1:1420/strictPort coincide con devUrl de Tauri; dist apunta al build raíz. TypeScript strict/noEmit y Vitest jsdom. Versiones de producto/paquetes coinciden.                    |
| CSP/ACL          | Ventana main local, CSP de producción sin unsafe-inline/eval; 16 comandos registrados coinciden con handler/ACL, más dos permisos de eventos.                                                 |
| CI               | Cuatro usos de acciones oficiales con SHA completo; toolchains coherentes; Ubuntu 24.04 y Windows 2022. actionlint valida sintaxis/expresiones; no certifica ejecución remota.                |
| Git/format/lint  | 27 casos verificados con git check-ignore --no-index. Ningún archivo ya versionado queda cubierto por ignores. ESLint/Prettier ignoran coverage y Prettier conserva su excepción de bindings. |

Comprobaciones de coherencia e ignores ejecutadas desde un script local no
versionado en target/repository-review/verify-config.mjs; no crea archivos .env
ni lee valores de secretos. Doctor es de solo lectura.

Validación de esta revisión: doctor, coherencia/27 casos Git, formato, ESLint,
TypeScript/build Vite, bindings:check, rustfmt, actionlint y 572 destinos locales
de enlaces/imágenes Markdown correctos. git diff --check correcto. El build y las
pruebas DSP completas de 9.5.a conservan su evidencia en su reporte fechado;
esta entrega revisa documentación/configuración y no produce medición física.

## Siguientes pasos

1. **9.5.b:** ejecutar sampler-native-check release con WAV y dispositivo/config
   explícitos, 0/1/16 voces; registrar hashes, IDs, métricas, transporte y reapertura.
   Separar audición física de probes y loopback.
2. **9.5.c:** revisar evidencia/fallos/límites y CI de la revisión publicada antes
   de cerrar sampler. Las comprobaciones anteriores de kernel/senoide no bastan.
3. **10–11:** mezclador de cuatro canales y secuenciador de 16 pasos.
4. **12–13:** persistencia/exportación del beat; después clips y escenas.

Ver [roadmap](../docs/roadmap.md), [protocolo](../docs/sampler-native-verification.md)
y [diagnósticos](../docs/sampler-diagnostics.md). Esta revisión no avanza 9.5.b.
