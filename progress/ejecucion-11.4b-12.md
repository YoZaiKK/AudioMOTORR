# Continuidad de ejecución — 11.4.b, 11.4.c y etapa 12

Petición del usuario: ejecutar los siguientes tres puntos del plan y documentar
la continuidad en `progress`. Alcance acordado: medición/escucha 11.4.b,
auditoría/entrega/CI 11.4.c y persistencia/exportación de etapa 12.

## Punto de partida

- Base Git `2d4cc28`; 11.3.b/c/d y 11.4.a implementados localmente sin publicar.
- Última huella verificada: 156 fuentes,
  `b2cfa93451ef73188557aecee552affc79f7b5c524981303ac1007af95b676cc`.
- Check completo, pruebas release offline/nativas y build desktop correctos.
- Benchmark secuenciado: 72 casos, cero asignaciones/liberaciones durante el
  procesamiento; 23 excesos nominales de bloques de un frame registrados.
- Dispositivo enumerado: Speakers (2- Realtek(R) Audio), WASAPI, salida estéreo
  con 48000 Hz f32 anunciado. Aún no se atribuye escucha al secuenciador.

## Orden de trabajo

1. Conservar binario/entradas/logs y ejecutar matriz física mono/estéreo ×
   empty/beat/dense, con controles y reaperturas. Registrar fallos sin descartarlos.
2. Solicitar observación auditiva después de reproducir. Mientras se espera,
   avanzar tareas independientes; no cerrar escucha por inferencia.
3. Auditar fuentes/evidencia y preparar la entrega 11.4.c; publicar la revisión
   autorizada y verificar sus dos jobs de CI. Si falta escucha, conservar ese
   criterio pendiente aunque el código esté publicado.
4. Etapa 12: contrato versionado, referencias de assets relativas, guardar/abrir,
   render/exportación del beat completo y flujo desktop/CLI; pruebas y entrega.

## Estado actual

Matriz nativa completada: seis casos y seis reaperturas correctos; máximo
180200 ns, ratio 0.01802; memoria/non-finite/deadlines medidos en cero.
[Reporte](etapa-11-medicion-nativa-secuenciador.md) y
[evidencia](evidencia-nativa-secuenciador-11.4b.json) conservan el binario y las
156 fuentes anteriores a etapa 12. Escucha pendiente de respuesta asíncrona.

Etapa 12 en curso: [contrato](../docs/project-format.md), modelo de referencias
en Project, `daw-app/project_file.rs` (v1/copia relativa/commit de archivo/abrir)
y `project_render.rs` (motor independiente, colas naturales y WAV) implementados.
Compilación y Clippy daw-app/daw-cli correctos. Seis casos de persistencia/render
pasan; dos de CLI real (`project-check`, `export-beat`) y dos de IPC con archivos
reales/ACL también pasan. La UI está conectada por cinco comandos nuevos (30
totales), con bindings regenerados. Frontend pasó 60 de 61 casos; se corrigió
un nombre de fixture de prueba y falta repetir esa suite. Typecheck correcto.
Cargo.lock cambió por serde/serde_json/hound en daw-app; versiones ya presentes
y fijadas, sin descarga.

Frontend final: 63 casos correctos. Check completo inicial correcto (62 casos
frontend antes de la última regresión); release de proyectos correcto (6 app +
2 CLI). Build desktop final correcto. WebView2 real completó guardar/copia de
assets/abrir/exportar con 197520 frames y 36 notas, audio sin inicializar; capturas
completas normal/estrecha y fuente de 164 archivos con huella
`9a1cdc442eb36256a03bef4a4d4417dadc4739db368812f29cc52ca8e823df39`.

Una repetición adicional de la matriz nativa con el ejecutable optimizado actual
no inició casos: lectura EPERM, copia archivada no disponible y EPERM posterior
en el output estándar. Compilación nativa código 0. Consulta Windows de detecciones
solo lectura sin coincidencias; causa no confirmada, ninguna configuración de
seguridad cambiada. Incidencia: `target/stage12-20261112/native-current/incident.json`.
Se conserva la matriz física anterior, vinculada exactamente a sus 156 fuentes;
no se atribuye a las 164 nuevas. Procesamiento engine/audio sin cambios.

Check completo final correcto, 01:40:52–01:44:08 UTC, con 63 casos frontend y
10 IPC entre sus suites; source/RT/Clippy correctos. Pendientes: manifiesto 12,
auditoría/publicación/CI. Escucha del usuario continúa pendiente.

Este archivo se actualizará en cada hito con resultados, cambios, comandos y
tareas pendientes. Los manifiestos anteriores conservan sus capturas históricas.
