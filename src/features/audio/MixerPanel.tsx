import { useState } from "react";
import type {
  MixerChannelDto,
  MixerChannelSnapshotDto,
  MixerPeakDto,
} from "../../shared/api/generated/api";
import type { DawSession } from "./useDawSession";

function Peaks({ name, peak }: { name: string; peak?: MixerPeakDto }) {
  if (!peak) return <p className="muted">Sin ventana medida</p>;
  return (
    <div className="mixer-peaks">
      {(
        [
          ["L", peak.left],
          ["R", peak.right],
        ] as const
      ).map(([side, value]) =>
        value == null ? null : (
          <label key={side}>
            {peak.right == null ? "Mono" : side}
            <meter
              aria-label={`${name} ${peak.right == null ? "mono" : side}`}
              min={0}
              max={1}
              value={value}
            />
            <span>{value.toFixed(5)}</span>
          </label>
        ),
      )}
    </div>
  );
}

function ChannelStrip({
  index,
  selected,
  applied,
  peak,
  disabled,
  session,
}: {
  index: number;
  selected?: MixerChannelDto;
  applied?: MixerChannelSnapshotDto;
  peak?: MixerPeakDto;
  disabled: boolean;
  session: DawSession;
}) {
  const [gain, setGain] = useState(selected?.gain.toString() ?? "");
  const [pan, setPan] = useState(selected?.pan.toString() ?? "");
  const [muted, setMuted] = useState(selected?.muted ?? false);
  const valid =
    gain.trim() !== "" &&
    pan.trim() !== "" &&
    Number.isFinite(Number(gain)) &&
    Number(gain) >= 0 &&
    Number(gain) <= 1 &&
    Number.isFinite(Number(pan)) &&
    Number(pan) >= -1 &&
    Number(pan) <= 1;
  const changed =
    !!selected &&
    (Number(gain) !== selected.gain ||
      Number(pan) !== selected.pan ||
      muted !== selected.muted);
  const name = `Canal ${index + 1}`;
  return (
    <article className="mixer-strip" aria-label={name}>
      <h3>{name}</h3>
      <form
        onSubmit={(event) => {
          event.preventDefault();
          if (!disabled && valid && changed)
            void session.setMixerChannel(
              index,
              Number(gain),
              Number(pan),
              muted,
            );
        }}
      >
        <fieldset disabled={disabled}>
          <legend>Borrador</legend>
          <label htmlFor={`mixer-gain-${index}`}>Ganancia · {name}</label>
          <input
            id={`mixer-gain-${index}`}
            disabled={disabled}
            type="number"
            min="0"
            max="1"
            step="any"
            value={gain}
            onChange={(event) => setGain(event.target.value)}
          />
          <label htmlFor={`mixer-pan-${index}`}>Pan · {name}</label>
          <input
            id={`mixer-pan-${index}`}
            disabled={disabled}
            type="number"
            min="-1"
            max="1"
            step="any"
            value={pan}
            onChange={(event) => setPan(event.target.value)}
          />
          <label className="mixer-mute">
            <input
              type="checkbox"
              disabled={disabled}
              checked={muted}
              onChange={(event) => setMuted(event.target.checked)}
            />
            Mute · {name}
          </label>
          <button disabled={disabled || !valid || !changed}>
            Aplicar canal {index + 1}
          </button>
        </fieldset>
      </form>
      <p data-testid={`mixer-selected-${index}`} className="muted">
        {selected
          ? `Seleccionado: G ${selected.gain} · P ${selected.pan} · ${selected.muted ? "mute" : "activo"}`
          : "Ajustes sin consultar"}
      </p>
      <p data-testid={`mixer-applied-${index}`}>
        {applied
          ? `Observado: G ${applied.effectiveGain.toFixed(3)} · L ${applied.left.toFixed(3)} / R ${applied.right.toFixed(3)} · rampa G ${applied.gainRemainingFrames} / P ${applied.panRemainingFrames} frames`
          : "Sin mezcla aplicada"}
      </p>
      <Peaks name={name} peak={peak} />
    </article>
  );
}

export function MixerPanel({ session }: { session: DawSession }) {
  const selected = session.mixerSettings;
  const applied =
    session.snapshot?.state === "ready" ? session.snapshot.mixer : null;
  const disabled =
    !session.native ||
    !session.connected ||
    session.busy ||
    !selected ||
    !(session.snapshot?.state === "uninitialized" || applied);
  const measured =
    applied && applied.meters.sequence !== "0" && applied.meters.frames !== "0";
  const edit = session.mixerEdit;
  return (
    <section className="panel" aria-labelledby="mixer-heading">
      <h2 id="mixer-heading">Mezclador · cuatro canales</h2>
      <p className="muted">
        Prepara los ajustes con audio cerrado o aplícalos durante una sesión
        mezclada. Pan −1: izquierda; +1: derecha. En salida mono el pan se
        conserva sin modificar el sonido.
      </p>
      <button
        disabled={!session.native || !session.connected || session.busy}
        onClick={() => void session.refreshMixer()}
      >
        Consultar ajustes de mezcla
      </button>
      {session.ready && !applied && (
        <p className="notice">
          Cierra audio e inicializa el sampler con mezcla para usar estos
          controles.
        </p>
      )}
      {edit && (
        <p role="status" data-testid="mixer-outcome">
          {edit === "offline"
            ? "Selección de mezcla actualizada sin enviar comandos de audio."
            : `Cambio ${edit.commandId}: ${edit.status === "pending" ? "aceptado; esperando procesamiento" : edit.status === "processed" ? "procesado; consulta valores y rampas observados" : "aplicación desconocida; sesión cerrada o perdida"}.`}
        </p>
      )}
      <div className="mixer-grid">
        {Array.from({ length: 4 }, (_, index) => (
          <ChannelStrip
            key={`${index}:${JSON.stringify(selected?.channels[index])}`}
            index={index}
            selected={selected?.channels[index]}
            applied={applied?.channels[index]}
            peak={measured ? applied.meters.channels[index] : undefined}
            disabled={disabled}
            session={session}
          />
        ))}
      </div>
      <div className="mixer-master">
        <h3>Master</h3>
        <Peaks
          name="Master"
          peak={measured ? applied.meters.master : undefined}
        />
        <p className="muted">
          {measured
            ? `Última ventana observada: ${applied.meters.frames} frames · secuencia ${applied.meters.sequence}.`
            : "Aún no hay una ventana de audio medida."}{" "}
          Los niveles de canal se miden antes del master. La vista puede omitir
          ventanas entre actualizaciones.
        </p>
      </div>
      <details>
        <summary>Rutas de los 16 slots</summary>
        <p className="muted">
          Los cambios de ruta afectan a los próximos disparos; las voces activas
          conservan su canal.
        </p>
        <div className="mixer-routes">
          {Array.from({ length: 16 }, (_, slot) => (
            <div key={slot}>
              <label htmlFor={`mixer-route-${slot}`}>Ruta · slot {slot}</label>
              <select
                id={`mixer-route-${slot}`}
                disabled={disabled}
                value={selected?.routes[slot] ?? ""}
                onChange={(event) =>
                  void session.setMixerRoute(slot, Number(event.target.value))
                }
              >
                {!selected && <option value="">Sin consultar</option>}
                {[0, 1, 2, 3].map((channel) => (
                  <option key={channel} value={channel}>
                    Canal {channel + 1}
                  </option>
                ))}
              </select>
              <small>
                {applied
                  ? `Observada: canal ${applied.settings.routes[slot] + 1}`
                  : "Sin ruta aplicada"}
              </small>
            </div>
          ))}
        </div>
      </details>
    </section>
  );
}
