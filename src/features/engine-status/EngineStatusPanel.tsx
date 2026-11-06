import type { ServiceSnapshotDto } from "../../shared/api/generated/api";
export function EngineStatusPanel({
  snapshot,
}: {
  snapshot: ServiceSnapshotDto | null;
}) {
  const engine = snapshot?.engine;
  const rows = [
    ["Servicio", snapshot?.state ?? "Sin conexión"],
    ["Backend", snapshot?.backend ?? "—"],
    ["Salida", snapshot?.output?.name ?? "—"],
    ["Formato", snapshot?.output?.sampleFormat ?? "—"],
    ["Sample rate", engine ? `${engine.sampleRate} Hz` : "—"],
    ["Canales", engine?.channels ?? "—"],
    ["Callbacks", snapshot?.callbacks ?? "—"],
    ["Frames del dispositivo", engine?.deviceFrames ?? "—"],
    ["Posición (frames)", engine?.transportFrames ?? "—"],
    ["Último comando aplicado", snapshot?.lastAppliedCommand ?? "—"],
    ["Secuencia observada", snapshot?.sequence ?? "—"],
  ];
  return (
    <section className="panel technical" aria-labelledby="engine-heading">
      <h2 id="engine-heading">Estado técnico</h2>
      <dl>
        {rows.map(([label, value]) => (
          <div key={label}>
            <dt>{label}</dt>
            <dd>{value}</dd>
          </div>
        ))}
      </dl>
    </section>
  );
}
