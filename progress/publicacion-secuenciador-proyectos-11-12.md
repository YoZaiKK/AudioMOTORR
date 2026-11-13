# Publicación — secuenciador y proyectos

La implementación se publicó en main mediante avance normal:
[`6c8c52e820081249a62ab5b664f94853568a4af6`](https://github.com/YoZaiKK/AudioMOTORR/commit/6c8c52e820081249a62ab5b664f94853568a4af6).
Título: `feat(daw): add sequencer desktop, portable projects and beat export`.
82 archivos inventariados y revisados; fuentes con huella
`9a1cdc442eb36256a03bef4a4d4417dadc4739db368812f29cc52ca8e823df39`.

La publicación incluye 11.3.b/c/d, diagnósticos 11.4.a, medición 11.4.b y
persistencia/exportación 12, sobre la entrega 11.3.a anterior. El índice estaba
vacío al preparar la entrega; se verificaron rama/base, remoto, alcance, hashes,
patch y enlaces. Commit y push devolvieron código 0, sin reset ni force push.
Inventario y recibo locales: `target/stage12-20261112/delivery.json` y
`publication.json`.

CI del SHA exacto inspeccionada y correcta en
[run 43348424541](https://github.com/YoZaiKK/AudioMOTORR/actions/runs/43348424541):
`headless-linux` y `desktop-windows` completados con `success`. El remoto main
confirmó `6c8c52e` después de publicar. Se comprobó equivalencia de las 164
fuentes: 160 idénticas y cuatro con diferencias solo de finales de línea.
El [manifiesto de implementación](evidencia-persistencia-exportacion-12.json)
conserva su estado anterior a publicar. Este reporte registra la evolución posterior.
En esta captura inicial la escucha estaba pendiente. Después, el usuario confirmó
«Si se escucha»; ver [cierre de 11](cierre-etapa-11.md).

[Evidencia de entrega](evidencia-publicacion-11-12.json) y
[cierre de etapa 12](cierre-etapa-12.md). La auditoría/publicación/CI de 11.4.c
está completada; etapa 11 quedó cerrada tras la confirmación posterior. El seguimiento
solo documental posterior identifica esta implementación por su SHA, sin atribuir
sus resultados a código diferente.

## Corrección posterior de precisión

La revisión [`4f126dc`](https://github.com/YoZaiKK/AudioMOTORR/commit/4f126dc0d8c56b17c77160b4da93a4d251bcb0e4)
activa float_roundtrip y añade la regresión de persistencia exacta.
[Verificación local](correccion-12-precision-json.md): check/release/build y
flujo real WebView2 correctos. CI Linux/Windows del SHA exacto correcta en [run 43362943622](https://github.com/YoZaiKK/AudioMOTORR/actions/runs/43362943622); etapa 12 cerrada dentro de su alcance.
[Evidencia de corrección y entrega](evidencia-publicacion-precision-12.json).
La CI inicial de `6c8c52e` conserva su alcance histórico.
