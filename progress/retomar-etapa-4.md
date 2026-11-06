# Continuidad — Etapa 4

Registro histórico del relevo de etapa 4. El estado de Git descrito aquí es el
de ese momento; consultar [continuidad vigente](continuidad-etapa-6.md) para la
entrega de las etapas 4–6 y el siguiente trabajo.

La pausa documentada el 2026-11-06 fue levantada por el usuario. La etapa 4 está
completada y verificada. El check completo y el smoke real pasan. El registro histórico de la pausa y los
resultados de reanudación se conservan en [detalles.md](detalles.md).

## Qué cambió desde la pausa

- ts-rs 12.0.1 descargado y fijado en Cargo.lock; compatible con Rust 1.90.
- IPC conectado a Tauri: diez comandos, errores estructurados y estado de aplicación.
- DTO TypeScript generado en src/shared/api/generated/api.ts; scripts
  bindings:generate/check disponibles, sin duplicación manual de tipos.
- Publicación engine-snapshot cada 67 ms como mínimo, con cierre y unión del hilo.
- UI React funcional: audio explícito, dispositivo, transporte, BPM y estado técnico.
- Estado aplicado separado de aceptación; descarte de secuencias antiguas con BigInt.
- Listeners limpiados bajo StrictMode y registros asíncronos tardíos.
- Permisos de aplicación y eventos concedidos solo a main local; CSP conservada.
- Tests frontend, handler IPC, precisión de contadores, ACL y publicador.
- Smoke opcional real de WebView2/hardware, con captura y salida normal de la app.

Verificación final: once tests frontend, 22 tests headless y 26 tests Rust del
workspace correctos, Clippy sin warnings, bindings regenerados sin cambios,
compilación desktop debug y smoke real con error de apertura/recuperación,
transporte, 137.5 BPM, reapertura y cierre normal.

Los borradores y bloqueos descritos en la pausa anterior ya fueron resueltos.

## Estado de Git

La base publicada sigue siendo main / origin/main, commit
69467ef97698b291fa764acee3d7cfd52d5a412e. Los cambios de etapa 4 y documentación
se conservan localmente, sin commit/push adicional en esta reanudación.

No descartar cambios locales ni atribuir al commit publicado los avances nuevos.
Consultar git status antes de seguir.

## Comandos disponibles

```powershell
npm run bindings:generate
npm run bindings:check
npm run test:frontend
npm run check
npm run desktop:build -- --debug --no-bundle
powershell -NoProfile -File scripts/desktop-smoke.ps1
```

El check completo incluye desktop y necesita sus prerrequisitos. El mock runtime
puede requerir autorización fuera del sandbox para acceder a sus rutas de datos.
El smoke real requiere Windows, WebView2, puerto local 9223 disponible y hardware;
abre y cierra la aplicación, y no pertenece a la suite automática. La captura local
se guarda en target/desktop-stage4.png y no se versiona.

Ejecutar primero check, luego desktop:build y finalmente el smoke. Cargo test
puede sustituir el binario principal por uno de desarrollo que espera devUrl;
reconstruir con Tauri CLI si se ejecutaron comandos Cargo entre build y smoke.

## Siguiente etapa

Actualización de continuidad: etapas 5 y 6 completadas localmente el 2026-11-06.
Consultar [continuidad tras etapa 6](continuidad-etapa-6.md) y
[cierre y auditoría](cierre-etapa-6.md) para el estado vigente. Se conserva debajo
el relevo histórico hacia etapas 5/6; ya no describe trabajo pendiente actual.

Etapa 5 completada y verificada: ver ../howToRunThis/README.md para guías, escenarios y capturas. Continuar con etapa 6. Las etapas 4 y 5 están cerradas en checklist.md.
Los contratos básicos ya tienen pruebas; ampliar únicamente donde falte evidencia:
comprobación reproducible de lockfiles/bindings, validaciones integradas restantes
y registro preciso de pruebas dependientes de hardware. La etapa 6 completará la
documentación de arquitectura y CI headless Linux / desktop Windows.

El README se mantiene en inglés. Progress conserva el seguimiento en español.
La salida sigue siendo silencio; oscilador, sampler y secuenciador quedan fuera
del alcance de estas etapas base.
