import type { FormEvent } from "react";
import type { DawSession } from "../audio/useDawSession";
export function TransportBar({ session }: { session: DawSession }) {
  const disabled = !session.ready || session.busy || session.pending !== null;
  const engine = session.snapshot?.engine;
  const appliedTempo = engine?.tempo ?? session.project?.tempo ?? 120;
  const labels = {
    stopped: "Detenido",
    playing: "Reproduciendo",
    paused: "Pausado",
  };
  function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    void session.setTempo(Number(new FormData(event.currentTarget).get("bpm")));
  }
  function gainSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    void session.setGain(Number(new FormData(event.currentTarget).get("gain")));
  }
  return (
    <section className="panel" aria-labelledby="transport-heading">
      <h2 id="transport-heading">Transporte</h2>
      <p>
        Estado aplicado:{" "}
        <strong data-testid="transport-state">
          {engine ? labels[engine.transport] : "Sin motor"}
        </strong>
      </p>
      <div className="actions">
        <button
          className="primary"
          onClick={() => void session.play()}
          disabled={disabled}
        >
          Play
        </button>
        <button onClick={() => void session.pause()} disabled={disabled}>
          Pause
        </button>
        <button onClick={() => void session.stop()} disabled={disabled}>
          Stop
        </button>
      </div>
      {engine && engine.requestedTransport !== engine.transport && (
        <p role="status">
          Transición a {labels[engine.requestedTransport]} · rampa pendiente
        </p>
      )}
      {engine?.sineFrequencyHz != null && (
        <form onSubmit={gainSubmit} className="tempo-form">
          <label htmlFor="signal-gain">Volumen (0–1)</label>
          <input
            key={engine.targetGain}
            id="signal-gain"
            name="gain"
            type="number"
            min="0"
            max="1"
            step="0.01"
            required
            defaultValue={engine.targetGain}
            disabled={disabled}
          />
          <button type="submit" disabled={disabled}>
            Aplicar volumen
          </button>
          <span className="muted">
            Ganancia observada: {engine.gain.toFixed(3)}
          </span>
        </form>
      )}
      <form onSubmit={submit} className="tempo-form">
        <label htmlFor="tempo">BPM</label>
        <input
          key={appliedTempo}
          id="tempo"
          name="bpm"
          type="number"
          min="20"
          max="300"
          step="any"
          required
          defaultValue={appliedTempo}
          disabled={disabled}
        />
        <button type="submit" disabled={disabled}>
          Aplicar BPM
        </button>
        <span className="muted">
          Aplicado: {engine ? `${engine.tempo} BPM` : "—"}
        </span>
      </form>
      <p role="status" aria-live="polite">
        {session.busy
          ? "Procesando solicitud…"
          : session.pending
            ? `Comando ${session.pending} aceptado. Esperando aplicación…`
            : "Sin comandos pendientes"}
      </p>
    </section>
  );
}
