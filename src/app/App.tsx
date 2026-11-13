import { AudioPanel } from "../features/audio/AudioPanel";
import { useDawSession } from "../features/audio/useDawSession";
import { TransportBar } from "../features/transport/TransportBar";
import { EngineStatusPanel } from "../features/engine-status/EngineStatusPanel";
import { SamplerPanel } from "../features/audio/SamplerPanel";
import { MixerPanel } from "../features/audio/MixerPanel";
import { SequencerPanel } from "../features/audio/SequencerPanel";
import { ProjectPanel } from "../features/project/ProjectPanel";

export function App() {
  const session = useDawSession();
  return (
    <main className="workspace">
      <header>
        <span className="eyebrow">AudioMOTORR · Etapa 12</span>
        <h1>
          {session.info?.name ?? "Rust DAW"}{" "}
          <small>{session.info?.version}</small>
        </h1>
        <p>{session.project?.title ?? "Proyecto"}</p>
        <p className="notice">
          {session.snapshot?.state === "uninitialized"
            ? "Crea un patrón y carga un banco WAV para comenzar"
            : session.snapshot?.sequencer != null
              ? "Secuenciador · 4 filas × 16 pasos · mezcla de cuatro canales"
              : session.snapshot?.mixer != null
                ? "Sampler · mezcla de cuatro canales · 16 voces"
                : session.snapshot?.sampler != null
                  ? "Sampler one-shot · 16 voces · rampas de 5 ms"
                  : session.snapshot?.engine?.sineFrequencyHz != null
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
      <SequencerPanel session={session} />
      <ProjectPanel key={session.documentVersion} session={session} />
      <TransportBar session={session} />
      <AudioPanel session={session} />
      <SamplerPanel session={session} />
      <MixerPanel session={session} />
      <EngineStatusPanel snapshot={session.snapshot} />
    </main>
  );
}
