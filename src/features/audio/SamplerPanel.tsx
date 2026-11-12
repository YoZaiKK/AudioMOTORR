import { useState } from "react";
import type { DawSession } from "./useDawSession";

const preparationLabels = {
  idle: "Sin banco preparado",
  loading: "Cargando WAV…",
  cancelling: "Cancelando carga…",
  ready: "Banco listo",
  cancelled: "Carga cancelada",
  error: "Falló la carga",
};
export function SamplerPanel({ session }: { session: DawSession }) {
  const [paths, setPaths] = useState([""]);
  const [rate, setRate] = useState("48000");
  const [channels, setChannels] = useState("2");
  const [device, setDevice] = useState("");
  const [slot, setSlot] = useState("0");
  const [gain, setGain] = useState("1");
  const prepared = session.preparation;
  const sampler = session.snapshot?.sampler;
  const disabled = !session.native || !session.connected || session.busy;
  const held =
    !!prepared && ["loading", "cancelling", "ready"].includes(prepared.state);
  const canLoad =
    !disabled && session.snapshot?.state === "uninitialized" && !held;
  const files = paths.flatMap((path, slot) =>
    path.trim() ? [{ slot, path }] : [],
  );
  const validRate =
    Number.isInteger(Number(rate)) &&
    Number(rate) >= 1 &&
    Number(rate) <= 4294967295;
  const occupied = sampler?.occupiedSlots ?? 0;
  const trigger = session.trigger;
  return (
    <section className="panel" aria-labelledby="sampler-heading">
      <h2 id="sampler-heading">Sampler · WAV one-shot</h2>
      <p className="muted">
        Cierra audio antes de cargar. Indica rutas locales WAV PCM16 o float32,
        mono/estéreo y con la misma frecuencia que la salida. Cargar no abre
        audio ni inicia Play.
      </p>
      <div className="actions">
        <label htmlFor="sample-rate">Frecuencia de salida (Hz)</label>
        <input
          id="sample-rate"
          type="number"
          value={rate}
          disabled={!canLoad}
          onChange={(event) => setRate(event.target.value)}
        />
        <label htmlFor="sample-channels">Canales del sampler</label>
        <select
          id="sample-channels"
          value={channels}
          disabled={!canLoad}
          onChange={(event) => setChannels(event.target.value)}
        >
          <option value="1">Mono</option>
          <option value="2">Estéreo</option>
        </select>
      </div>
      {paths.map((path, index) => (
        <label
          className="sample-path"
          key={index}
          htmlFor={`sample-path-${index}`}
        >
          Ruta WAV · slot {index}
          <input
            id={`sample-path-${index}`}
            type="text"
            value={path}
            disabled={!canLoad}
            onChange={(event) =>
              setPaths(
                paths.map((old, slot) =>
                  slot === index ? event.target.value : old,
                ),
              )
            }
          />
        </label>
      ))}
      <div className="actions">
        <button
          disabled={!canLoad || paths.length >= 16}
          onClick={() => setPaths([...paths, ""])}
        >
          Añadir slot
        </button>
        <button
          disabled={!canLoad || files.length === 0 || !validRate}
          onClick={() =>
            void session.prepareSamples(Number(rate), Number(channels), files)
          }
        >
          Cargar banco WAV
        </button>
        <button
          disabled={disabled || !held || prepared?.state === "cancelling"}
          onClick={() => void session.cancelPreparation()}
        >
          Cancelar / descartar banco
        </button>
      </div>
      <p role="status">
        {prepared
          ? preparationLabels[prepared.state]
          : "Consultando preparación…"}
        {prepared?.state === "ready" &&
          ` · ${prepared.retainedPcmBytes} bytes PCM · ${prepared.sampleRate} Hz / ${prepared.channels} canales`}
      </p>
      {prepared?.error && (
        <p role="alert" className="error">
          {prepared.error}
        </p>
      )}
      <label htmlFor="sampler-device">Salida del sampler</label>
      <select
        id="sampler-device"
        value={device}
        disabled={disabled || session.ready}
        onChange={(event) => setDevice(event.target.value)}
      >
        <option value="">Salida predeterminada</option>
        {session.devices.map((output) => (
          <option key={output.id} value={output.id}>
            {output.name}
          </option>
        ))}
      </select>
      <button
        disabled={
          disabled ||
          session.snapshot?.state !== "uninitialized" ||
          prepared?.state !== "ready"
        }
        onClick={() => void session.initializeSampler(device || null)}
      >
        Inicializar sampler
      </button>
      <button
        disabled={
          disabled ||
          !session.mixerSettings ||
          session.snapshot?.state !== "uninitialized" ||
          prepared?.state !== "ready"
        }
        onClick={() => void session.initializeMixedSampler(device || null)}
      >
        Inicializar con mezcla
      </button>
      <p className="muted">
        Inicializar abre la salida con el formato exacto del banco y lo consume,
        incluso si falla. Después pulsa Play y dispara un slot. Stop conserva el
        banco; Cerrar audio lo libera.
      </p>
      <div className="actions">
        <label htmlFor="trigger-slot">Slot a disparar</label>
        <select
          id="trigger-slot"
          value={slot}
          disabled={!sampler || disabled}
          onChange={(event) => setSlot(event.target.value)}
        >
          {Array.from({ length: 16 }, (_, slot) => (
            <option
              key={slot}
              value={slot}
              disabled={!(occupied & (1 << slot))}
            >
              Slot {slot}
              {occupied & (1 << slot) ? " · cargado" : " · vacío"}
            </option>
          ))}
        </select>
        <label htmlFor="trigger-gain">Ganancia del disparo (0–1)</label>
        <input
          id="trigger-gain"
          type="number"
          min="0"
          max="1"
          step="0.01"
          value={gain}
          disabled={!sampler || disabled}
          onChange={(event) => setGain(event.target.value)}
        />
        <button
          disabled={
            disabled ||
            !session.ready ||
            !sampler ||
            !(occupied & (1 << Number(slot))) ||
            trigger?.status === "pending" ||
            gain.trim() === "" ||
            !Number.isFinite(Number(gain)) ||
            Number(gain) < 0 ||
            Number(gain) > 1
          }
          onClick={() => void session.triggerSample(Number(slot), Number(gain))}
        >
          Disparar sample
        </button>
      </div>
      {sampler && (
        <p>
          Voces observadas: {sampler.activeVoices}/16 · iniciados:{" "}
          {sampler.started} · rechazados: {sampler.rejected}
        </p>
      )}
      {trigger && (
        <p role="status" data-testid="trigger-outcome">
          Disparo {trigger.commandId}:{" "}
          {trigger.status === "pending"
            ? "aceptado; esperando procesamiento"
            : trigger.status === "started"
              ? `procesado; voz ${trigger.voice} iniciada`
              : trigger.status === "rejected"
                ? `procesado; rechazado (${trigger.reason})`
                : "resultado desconocido; observación perdida o sesión cerrada"}
          .
        </p>
      )}
    </section>
  );
}
