import { useState } from "react";
import type { DawSession } from "../audio/useDawSession";
import type { BeatTailDto } from "../../shared/api/generated/api";

export function ProjectPanel({ session }: { session: DawSession }) {
  const doc = session.document;
  const [path, setPath] = useState(doc?.path ?? "beat.json");
  const [title, setTitle] = useState(
    doc?.summary.title ?? "Proyecto sin título",
  );
  const [replace, setReplace] = useState(false);
  const [output, setOutput] = useState("beat.wav");
  const [rate, setRate] = useState("48000");
  const [channels, setChannels] = useState("2");
  const [bars, setBars] = useState("1");
  const [tail, setTail] = useState<BeatTailDto>("finish");
  const disabled = !session.native || !session.connected || session.busy;
  const held =
    session.preparation != null &&
    ["loading", "cancelling", "ready"].includes(session.preparation.state);
  const canOpen =
    !disabled && session.snapshot?.state === "uninitialized" && !held;
  const validRate =
    Number.isInteger(Number(rate)) &&
    Number(rate) >= 100 &&
    Number(rate) <= 192000;
  const validBars =
    Number.isInteger(Number(bars)) && Number(bars) >= 1 && Number(bars) <= 4096;
  const assets =
    doc?.assets.flatMap((asset, slot) => (asset ? [{ slot, asset }] : [])) ??
    [];
  return (
    <section className="panel project-panel" aria-labelledby="project-heading">
      <h2 id="project-heading">Proyecto y exportación</h2>
      <p className="muted">
        Guardar copia los samples junto al proyecto para poder moverlo. Abrir
        requiere audio cerrado y ninguna preparación pendiente.
      </p>
      <p role="status" data-testid="project-file-state">
        {!doc
          ? "Proyecto sin consultar"
          : doc.dirty || title.trim() !== doc.summary.title
            ? "Cambios sin guardar"
            : "Proyecto guardado"}
      </p>
      <div className="project-fields">
        <label>
          Título del proyecto
          <input
            value={title}
            disabled={disabled}
            onChange={(event) => setTitle(event.target.value)}
          />
        </label>
        <label>
          Archivo de proyecto
          <input
            value={path}
            disabled={disabled}
            onChange={(event) => setPath(event.target.value)}
          />
        </label>
      </div>
      <div className="actions">
        <button
          disabled={disabled || !doc || !path.trim() || !title.trim()}
          onClick={() =>
            void (async () => {
              const saved = await session.saveProject(
                path,
                replace || doc?.path === path,
                title,
              );
              if (saved) {
                setPath(saved.path ?? path);
                setTitle(saved.summary.title);
              }
            })()
          }
        >
          Guardar proyecto
        </button>
        <button
          disabled={!canOpen || !path.trim()}
          onClick={() => void session.openProject(path)}
        >
          Abrir proyecto
        </button>
        <button
          disabled={disabled}
          onClick={() => void session.refreshDocument()}
        >
          Consultar proyecto
        </button>
        <label>
          <input
            type="checkbox"
            checked={replace}
            disabled={disabled}
            onChange={(event) => setReplace(event.target.checked)}
          />{" "}
          Reemplazar otro archivo existente
        </label>
      </div>
      <details>
        <summary>Samples del proyecto ({assets.length})</summary>
        {assets.length === 0 ? (
          <p className="muted">
            Carga WAV en el sampler para asignarlos al proyecto.
          </p>
        ) : (
          <ul>
            {assets.map(({ slot, asset }) => (
              <li key={slot}>
                Slot {slot}: {asset}
              </li>
            ))}
          </ul>
        )}
      </details>
      <div className="project-fields">
        <label>
          Frecuencia para preparar/exportar (Hz)
          <input
            type="number"
            min="100"
            max="192000"
            value={rate}
            disabled={disabled}
            onChange={(event) => setRate(event.target.value)}
          />
        </label>
        <label>
          Canales para preparar/exportar
          <select
            value={channels}
            disabled={disabled}
            onChange={(event) => setChannels(event.target.value)}
          >
            <option value="1">Mono</option>
            <option value="2">Estéreo</option>
          </select>
        </label>
      </div>
      <div className="actions">
        <button
          disabled={!canOpen || !validRate || assets.length === 0}
          onClick={() =>
            void session.prepareProjectSamples(Number(rate), Number(channels))
          }
        >
          Preparar samples del proyecto
        </button>
      </div>
      <p className="muted">
        Preparar no abre audio. Después, inicializa el secuenciador desde el
        sampler. Exportar usa el patrón y la mezcla seleccionados y conserva la
        reproducción actual.
      </p>
      <div className="project-fields">
        <label>
          Archivo WAV del beat
          <input
            value={output}
            disabled={disabled}
            onChange={(event) => setOutput(event.target.value)}
          />
        </label>
        <label>
          Compases a exportar
          <input
            type="number"
            min="1"
            max="4096"
            value={bars}
            disabled={disabled}
            onChange={(event) => setBars(event.target.value)}
          />
        </label>
        <label>
          Final del beat
          <select
            value={tail}
            disabled={disabled}
            onChange={(event) => setTail(event.target.value as BeatTailDto)}
          >
            <option value="finish">Completar colas de samples</option>
            <option value="cut">Cortar al final de los compases</option>
          </select>
        </label>
      </div>
      <div className="actions">
        <button
          disabled={
            disabled || !doc || !output.trim() || !validRate || !validBars
          }
          onClick={() =>
            void session.exportBeat({
              outputPath: output,
              sampleRate: Number(rate),
              channels: Number(channels),
              bars: Number(bars),
              blockSize: 256,
              tail,
            })
          }
        >
          Exportar beat
        </button>
      </div>
      {session.beatExport && (
        <p role="status" data-testid="beat-export-result">
          Última exportación: {session.beatExport.path} ·{" "}
          {session.beatExport.report.totalFrames} frames (
          {session.beatExport.report.musicalFrames} musicales +{" "}
          {session.beatExport.report.tailFrames} de cola) ·{" "}
          {session.beatExport.report.counters.started} notas iniciadas ·{" "}
          {session.beatExport.report.counters.poolFull} rechazadas por voces
          llenas.
        </p>
      )}
    </section>
  );
}
