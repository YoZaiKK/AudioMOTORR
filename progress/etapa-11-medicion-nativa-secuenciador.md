# Etapa 11.4.b — Medición física del secuenciador

Se ejecutaron seis casos con sus seis reaperturas usando el binario release
verificado de 11.4.a, antes de modificar fuentes para etapa 12. Salida Speakers
(2- Realtek(R) Audio), WASAPI, 48000 Hz estéreo; bancos mono/estéreo y patrones
empty/beat/dense a 127.25 BPM. Todos devolvieron código 0.

Se observaron timeline racional, slot vacío, pausa/ediciones, notas en mute,
reanudación, Stop y reapertura. Las doce sesiones registran cero asignaciones,
liberaciones, muestras no finitas y deadlines incumplidos medidos. Las métricas
son acumuladas por sesión, incluidas fases de control; no son percentiles
aislados del patrón inicial. P50/p99 son cotas superiores del histograma.

Binario, dispositivo, entradas, logs, fases, contadores y métricas quedan en
[evidencia](evidencia-nativa-secuenciador-11.4b.json), con artefactos locales en
`target/stage11-4b-20261112/`. Se conserva la huella de 156 fuentes de 11.4.a.

**Escucha pendiente:** se preguntó al usuario después de la reproducción real.
Un código 0 y las métricas no confirman audibilidad. No se atribuye al secuenciador
la confirmación auditiva de etapas previas. La etapa 11 todavía no está cerrada.
Siguen auditoría/entrega/CI 11.4.c y, en paralelo mientras llega esa observación,
persistencia/exportación de etapa 12. Ver [continuidad de esta ejecución](ejecucion-11.4b-12.md).

Sin loopback acústico, unplug físico, estrés prolongado ni otros dispositivos,
formatos/plataformas. Los samples breves del caso denso no garantizan agotar voces;
esa política está contrastada con colas largas en el benchmark offline.
