import { useState } from "react";
import type { DawSession } from "./useDawSession";
export function AudioPanel({ session }: { session: DawSession }) {
  const [device, setDevice] = useState("");
  const active = session.snapshot?.state === "ready";
  const disabled = !session.native || session.busy;
  const selected = session.devices.find((output) => output.id === device);
  return (
    <section className="panel" aria-labelledby="audio-heading">
      <h2 id="audio-heading">Salida de audio</h2>
      <label htmlFor="output-device">Dispositivo</label>
      <select
        id="output-device"
        value={device}
        onChange={(event) => setDevice(event.target.value)}
        disabled={disabled || active}
      >
        <option value="">Salida predeterminada</option>
        {session.devices.map((output) => (
          <option key={output.id} value={output.id}>
            {output.name}
            {output.isDefault ? " · predeterminado" : ""}
          </option>
        ))}
      </select>
      {selected && (
        <p className="muted">
          {selected.configs.length} configuraciones anunciadas. Se usará la
          configuración predeterminada del dispositivo.
        </p>
      )}
      <div className="actions">
        <button onClick={() => void session.refresh()} disabled={disabled}>
          Actualizar dispositivos
        </button>
        <button
          className="primary"
          onClick={() => void session.initialize(device || null)}
          disabled={disabled || !session.connected || active}
        >
          Inicializar audio
        </button>
        <button
          onClick={() => void session.shutdown()}
          disabled={
            disabled ||
            !session.connected ||
            session.snapshot?.state === "uninitialized"
          }
        >
          Cerrar audio
        </button>
      </div>
      <p className="muted">
        Inicializar abre el dispositivo. Play y Stop controlan el transporte.
      </p>
    </section>
  );
}
