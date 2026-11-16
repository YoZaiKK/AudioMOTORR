import { useState } from "react";
import type { DawSession } from "./useDawSession";
import type {
  ClipDto,
  LaunchActionDto,
  QuantizationDto,
  SceneActionDto,
} from "../../shared/api/generated/api";

const indices = Array.from({ length: 16 }, (_, i) => i);
function actionLabel(action: LaunchActionDto) {
  return action.action === "launch"
    ? `Clip ${action.slot}`
    : action.action === "stop"
      ? "Detener"
      : "Conservar";
}
export function LauncherPanel({ session }: { session: DawSession }) {
  const [row, setRow] = useState(0);
  const [slot, setSlot] = useState(0);
  const [scene, setScene] = useState(0);
  const selected = session.launcherSettings;
  const observed =
    session.snapshot?.state === "ready" ? session.snapshot.launcher : null;
  const unavailable =
    !session.native || !session.connected || session.busy || !selected;
  const closed = session.snapshot?.state === "uninitialized";
  const editing = !unavailable && (closed || !!observed);
  const launching = !unavailable && !!observed && selected?.mode === "launcher";
  const clip = selected?.clips[row][slot];
  const receipt = session.launcherLaunch;
  return (
    <section
      className="panel launcher-panel"
      aria-labelledby="launcher-heading"
    >
      <span className="eyebrow">4 filas · 16 clips · 16 escenas</span>
      <h2 id="launcher-heading">Clips y escenas</h2>
      <div className="project-fields">
        <label>
          Modo del proyecto
          <select
            value={selected?.mode ?? "pattern"}
            disabled={unavailable || !closed}
            onChange={(e) =>
              void session.setPlaybackMode(
                e.target.value as "pattern" | "launcher",
              )
            }
          >
            <option value="pattern">Patrón</option>
            <option value="launcher">Clips y escenas</option>
          </select>
        </label>
        <label>
          Cuantización de próximos lanzamientos
          <select
            value={selected?.quantization ?? "bar"}
            disabled={!editing}
            onChange={(e) =>
              void session.editLauncher({
                edit: "quantization",
                quantization: e.target.value as QuantizationDto,
              })
            }
          >
            <option value="step">Paso</option>
            <option value="beat">Pulso</option>
            <option value="bar">Compás</option>
          </select>
        </label>
      </div>
      <p className="muted">
        Cambia de modo con el audio cerrado. Editar conserva el contenido que ya
        fue lanzado; vuelve a lanzarlo para escuchar los cambios.
      </p>
      <div
        className="launcher-grid-scroll"
        tabIndex={0}
        aria-label="Matriz de clips, desplazamiento horizontal"
      >
        <table className="launcher-grid">
          <caption>
            Selecciona un clip para editar. Activo y en cola provienen del
            motor.
          </caption>
          <thead>
            <tr>
              <th>Fila / clip</th>
              {indices.map((s) => (
                <th key={s}>{s}</th>
              ))}
            </tr>
          </thead>
          <tbody>
            {[0, 1, 2, 3].map((r) => (
              <tr key={r}>
                <th>Fila {r}</th>
                {indices.map((s) => {
                  const active = observed?.active[r]?.slot === s;
                  const pending = observed?.pending[r]?.action;
                  const queued =
                    pending?.action === "launch" && pending.slot === s;
                  return (
                    <td key={s}>
                      <button
                        type="button"
                        aria-label={`Editar fila ${r} clip ${s}`}
                        aria-pressed={row === r && slot === s}
                        className={`${active ? "clip-active" : ""} ${queued ? "clip-queued" : ""}`}
                        onClick={() => {
                          setRow(r);
                          setSlot(s);
                        }}
                      >
                        {selected?.clips[r][s] ? "●" : "·"}
                        <span className="clip-indicator">
                          {active ? "A" : ""}
                          {queued ? "Q" : ""}
                        </span>
                      </button>
                    </td>
                  );
                })}
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      <p className="muted">
        A = activo observado · Q = lanzamiento en cola · borde claro = selección
        de edición
      </p>
      <h3>
        Editor · fila {row}, clip {slot}
      </h3>
      <div className="actions">
        <button
          disabled={!editing || !!clip}
          onClick={() =>
            void session.editLauncher({
              edit: "clip",
              row,
              slot,
              clip: {
                steps: indices.map(() => false) as ClipDto["steps"],
                sampleSlot: row,
              },
            })
          }
        >
          Crear clip vacío
        </button>
        <button
          disabled={!editing || !clip}
          onClick={() =>
            void session.editLauncher({ edit: "removeClip", row, slot })
          }
        >
          Eliminar clip
        </button>
        <button
          className="primary"
          disabled={!launching || !clip}
          onClick={() =>
            void session.launchLauncher({ target: "clip", row, slot })
          }
        >
          Lanzar clip seleccionado
        </button>
        <button
          disabled={!launching}
          onClick={() =>
            void session.launchLauncher({ target: "stopRow", row })
          }
        >
          Detener fila {row}
        </button>
      </div>
      <label className="launcher-sample">
        Sample del clip
        <select
          value={clip?.sampleSlot ?? row}
          disabled={!editing || !clip}
          onChange={(e) =>
            void session.editLauncher({
              edit: "sample",
              row,
              slot,
              sample: Number(e.target.value),
            })
          }
        >
          {indices.map((i) => (
            <option key={i} value={i}>
              Slot {i}
              {(session.snapshot?.sampler?.occupiedSlots ??
                session.preparation?.occupiedSlots ??
                0) &
              (1 << i)
                ? " · cargado"
                : " · sin banco cargado"}
            </option>
          ))}
        </select>
      </label>
      <div className="clip-steps" aria-label="Pasos del clip seleccionado">
        {indices.map((step) => (
          <button
            key={step}
            aria-label={`Clip fila ${row} slot ${slot} paso ${step + 1}`}
            aria-pressed={clip?.steps[step] ?? false}
            disabled={!editing || !clip}
            onClick={() =>
              void session.editLauncher({
                edit: "step",
                row,
                slot,
                step,
                enabled: !clip?.steps[step],
              })
            }
          >
            {step + 1}
          </button>
        ))}
      </div>
      <h3>Escenas</h3>
      <div className="project-fields">
        <label>
          Escena a editar y lanzar
          <select
            value={scene}
            onChange={(e) => setScene(Number(e.target.value))}
          >
            {indices.map((i) => (
              <option key={i} value={i}>
                Escena {i}
              </option>
            ))}
          </select>
        </label>
        <label>
          Escena inicial guardada / exportación
          <select
            value={selected?.initialScene ?? 0}
            disabled={!editing}
            onChange={(e) =>
              void session.editLauncher({
                edit: "initialScene",
                index: Number(e.target.value),
              })
            }
          >
            {indices.map((i) => (
              <option key={i} value={i}>
                Escena {i}
              </option>
            ))}
          </select>
        </label>
      </div>
      <div className="project-fields">
        {[0, 1, 2, 3].map((r) => {
          const action = selected?.scenes[scene][r];
          return (
            <label key={r}>
              Escena {scene} · fila {r}
              <select
                disabled={!editing}
                value={
                  action?.action === "launch"
                    ? String(action.slot)
                    : (action?.action ?? "keep")
                }
                onChange={(e) => {
                  const value = e.target.value;
                  const action: SceneActionDto =
                    value === "keep"
                      ? { action: "keep" }
                      : value === "stop"
                        ? { action: "stop" }
                        : { action: "launch", slot: Number(value) };
                  void session.editLauncher({
                    edit: "sceneAction",
                    index: scene,
                    row: r,
                    action,
                  });
                }}
              >
                <option value="keep">Conservar (Keep)</option>
                <option value="stop">Detener (Stop)</option>
                {indices.map((i) => (
                  <option key={i} value={i} disabled={!selected?.clips[r][i]}>
                    Clip {i}
                    {!selected?.clips[r][i] ? " · vacío" : ""}
                  </option>
                ))}
              </select>
            </label>
          );
        })}
      </div>
      <div className="actions">
        <button
          className="primary"
          disabled={!launching}
          onClick={() =>
            void session.launchLauncher({ target: "scene", index: scene })
          }
        >
          Lanzar escena {scene}
        </button>
        <button
          disabled={!launching}
          onClick={() => void session.launchLauncher({ target: "stopAll" })}
        >
          Detener todas las filas
        </button>
        <button
          disabled={unavailable}
          onClick={() => void session.refreshLauncher()}
        >
          Consultar estado de clips
        </button>
      </div>
      <p className="muted">
        Lanzar no cambia la escena inicial. Inicializar deja las filas vacías y
        el transporte detenido: lanza una escena y pulsa Play. La exportación
        usa la escena inicial en modo Clips.
      </p>
      {receipt && (
        <p role="status" data-testid="launcher-receipt">
          Solicitud {receipt.receipt.accepted.commandId}:{" "}
          {receipt.progress.status === "accepted"
            ? "aceptada; esperando procesamiento"
            : receipt.progress.status === "processed"
              ? `procesada (motor ${receipt.progress.requestId}); consulta las transiciones observadas`
              : receipt.progress.status === "rejected"
                ? `rechazada: ${receipt.progress.reason}`
                : "resultado desconocido; consulta el estado antes de volver a lanzar"}
          .
        </p>
      )}
      {observed ? (
        <div aria-label="Estado observado de clips">
          <h3>Observado en el motor</h3>
          {[0, 1, 2, 3].map((r) => {
            const active = observed.active[r],
              pending = observed.pending[r],
              transition = observed.lastTransition[r];
            return (
              <p key={r} data-testid={`launcher-row-${r}`}>
                Fila {r}:{" "}
                {active
                  ? `clip ${active.slot} activo · origen ${active.originStep} · motor ${active.requestId}`
                  : "sin clip activo"}
                {pending
                  ? ` · en cola: ${actionLabel(pending.action)} al paso global ${pending.targetStep} (motor ${pending.requestId})`
                  : " · sin transición pendiente"}
                {observed.rowSteps[r] !== null
                  ? ` · posición al siguiente frame: paso ${observed.rowSteps[r]! + 1}`
                  : ""}
                {transition
                  ? ` · última transición: ${actionLabel(transition.action)} al paso ${transition.targetStep} (motor ${transition.requestId})`
                  : ""}
              </p>
            );
          })}
          <p>
            Transiciones: {observed.transitions} · notas iniciadas:{" "}
            {observed.counters.started} · sample ausente:{" "}
            {observed.counters.missingAsset} · pool lleno:{" "}
            {observed.counters.poolFull}
          </p>
          <p>
            Último límite observado:{" "}
            {observed.lastBoundary
              ? `vuelta ${observed.lastBoundary.loopIndex}, paso ${observed.lastBoundary.step + 1}, frame ${observed.lastBoundary.musicalFrame}`
              : "ninguno"}
          </p>
          {observed.lastEvent && (
            <p>
              Última nota observada: fila {observed.lastEvent.row}, clip{" "}
              {observed.lastEvent.clipSlot}, paso{" "}
              {observed.lastEvent.clipStep + 1} ·{" "}
              {observed.lastEvent.rejection ??
                `voz ${observed.lastEvent.voice}`}
            </p>
          )}
        </div>
      ) : (
        <p className="muted">
          Sin sesión de clips observada. Prepara el banco WAV e inicializa el
          proyecto en modo Clips.
        </p>
      )}
    </section>
  );
}
