# Etapa 9.4.c — Application, DTO, IPC y UI de sampler

Fecha: 2026-11-09. Cambios locales sobre 1002efa, conservando 9.3 y 9.4.a–b.
Sin commit/push, publicación, CI remota nueva o hardware abierto. Los manifiestos
anteriores se conservan como snapshots históricos; no describen este nuevo código.

Registro fechado de 9.4.c. La [auditoría posterior 9.4.d](cierre-integracion-9.4.md)
cierra integración local y corrige recuperación de dispositivos/polling; su nuevo
manifiesto identifica el código actual. Los resultados siguientes se conservan.

## Implementación

- Application posee coordinador, worker de preparación y resultado pendiente.
  Cerrar audio precede a cargar/instalar. El decoder corre fuera del mutex de
  Application y del controlador/callback; consulta/cancelación no esperan su I/O.
- Estados idle/loading/cancelling/ready/cancelled/error, secuencia e ID propios.
  IDs obsoletos no alteran el banco actual. Cancelar descarta incluso éxito aún
  no publicado; un worker pendiente impide otro hasta recogerlo. El cierre une
  el worker; una lectura OS bloqueada sigue sin plazo de interrupción.
- Inicialización manual para pruebas y nativa para desktop consumen un banco
  listo por ID, también si falla la apertura. No hay Play ni trigger automáticos.
  El documento conserva ID/título/BPM; rutas/bancos de sesión no son persistencia.
- Cinco comandos nuevos y ACL local main; dieciséis comandos totales. DTOs/TS
  generados exponen preparación y telemetría sampler. IDs, secuencias, bytes y
  contadores grandes son strings; rechazo de trigger es una unión tipada.
- UI explícita: hasta 16 rutas WAV, formato de salida, carga/cancelación/descarte,
  selección de dispositivo, inicialización, slot/ganancia y resultado por ID.
  El master gain también sirve al sampler. Browser continúa sin llamadas nativas.
- La UI distingue aceptado pendiente, procesado Started/rechazado y resultado
  desconocido por pérdida/cierre. No atribuye el último outcome a otro ID.
  Preparación se consulta cada 200 ms tras respuesta mientras carga/cancela,
  sin peticiones solapadas; secuencias y limpieza evitan respuestas obsoletas.

Contrato: [sampler desktop](../docs/sampler-desktop.md). Uso:
[guía práctica](../howToRunThis/06-sampler.md).

## Verificación

Pruebas focales de aplicación: WAV real → worker → banco → sesión manual →
rechazo NotPlaying → Play/trigger Started; cierre/reutilización, IDs obsoletos,
validación, fallo de archivo y cancelación antes/después de completar. Variante
headless verifica fallo NativeAudioDisabled sin fallback y consumo del banco.
Una prueba privada sincroniza un worker detenido: Application sigue accesible,
cancelación no recoge prematuramente PCM/permiso y Drop une el worker siguiente.

El test IPC existente ahora invoca los cinco comandos con runtime mock y una
sesión manual, sin abrir hardware; verifica campos/errores, resultados exactos,
strings >2^53/u64::MAX y denegación desde otra ventana. Frontend añade ocho casos
para flujo explícito, descarte/fallo de apertura, cuatro carreras outcome/respuesta,
QueueFull y respuesta de preparación posterior a cancelar; 21 pruebas totales.

Comandos focales:

```powershell
cargo test -p daw-app --no-default-features --locked --lib --test sampler_application_contract
cargo test -p rust-daw-desktop --locked --test ipc_contract
cargo clippy -p daw-app --no-default-features --all-targets --locked -- -D warnings
cargo clippy -p rust-daw-desktop --all-targets --locked -- -D warnings
npm run bindings:generate
npm run bindings:check
npm run test:frontend
npm run check
npm run desktop:build -- --debug --no-bundle
```

Resultados locales correctos:

- npm run check: 106 pruebas headless, 112 workspace, 26 audio instrumentadas,
  ocho memoria decoder/banco, 21 frontend y nueve escenarios CLI. Las suites se
  solapan y no se suman como pruebas únicas. La prueba física sigue ignorada.
- Bindings deterministas, TypeScript, ESLint, formato, build frontend, rustfmt
  y Clippy correctos dentro del check completo.
- npm run desktop:build -- --debug --no-bundle correcto después del check:
  target/debug/rust-daw-desktop.exe, sin instalador, firma o publicación.
- Cargo.lock, package-lock.json y workflow mantienen sus hashes de 9.4.b;
  api.ts cambia deliberadamente y permanece idéntico antes/después de check/build.

El [manifiesto actual](evidencia-desktop-9.4c.json) registra 116 archivos de
código/configuración con huella conjunta. Se conservan los manifiestos previos;
no se atribuye a este nuevo código su CI o sus mediciones. Logs locales no
versionados en target/stage9-4c/check.log y desktop-build.log; hashes de control
en hashes-before.json/hashes-after.json. La revisión final pasa formato,
461 destinos locales de enlaces/imágenes Markdown, diez límites del contrato,
actionlint y git diff --check; los 116 archivos coinciden con su manifiesto.
No implica audición física ni ejecución remota.

## Siguiente etapa: 9.4.d

Auditar criterios integrados de 9.4.a–c contra código y evidencia: propiedad de
PCM, cierre/preparación/cancelación, formato exacto, transporte/headroom, colas y
resultados por ID. Completar cualquier regresión que revele la auditoría, cerrar
integración local y dejar un protocolo reproducible de medición/audición para 9.5.
El check/build de esta entrega no completa esa auditoría ni las mediciones nativas.
9.5 mantiene pendientes rendimiento sampler, desconexión física, plataformas no
verificadas y CI del SHA que se publique. Etapas 10–13 no se adelantan.
