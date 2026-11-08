import { AudioPanel } from "../features/audio/AudioPanel";
import { useDawSession } from "../features/audio/useDawSession";
import { TransportBar } from "../features/transport/TransportBar";
import { EngineStatusPanel } from "../features/engine-status/EngineStatusPanel";

export function App() {
  const session = useDawSession();
  return (
    <main className="workspace">
      <header>
        <span className="eyebrow">AudioMOTORR · Etapa 8</span>
        <h1>
          {session.info?.name ?? "Rust DAW"}{" "}
          <small>{session.info?.version}</small>
        </h1>
        <p>{session.project?.title ?? "Proyecto"}</p>
        <p className="notice">
          {session.snapshot?.engine?.sineFrequencyHz != null
            ? `Senoide: ${session.snapshot.engine.sineFrequencyHz} Hz · rampas de 5 ms`
            : "Salida de silencio · selecciona senoide para generar señal"}
        </p>
      </header>
      {!session.native && (
        <p className="browser-notice">
          Vista de interfaz: requiere Tauri para audio
        </p>
      )}
      {session.native && !session.connected && !session.error && (
        <p role="status">Conectando con el servicio…</p>
      )}
      {session.error && (
        <div className="error" role="alert">
          <strong>{session.error.code}</strong>: {session.error.message}
        </div>
      )}
      <AudioPanel session={session} />
      <TransportBar session={session} />
      <EngineStatusPanel snapshot={session.snapshot} />
    </main>
  );
}
