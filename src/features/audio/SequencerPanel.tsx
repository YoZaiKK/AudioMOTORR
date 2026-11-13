import type { DawSession } from "./useDawSession";

export function SequencerPanel({ session }: { session: DawSession }) {
  const selected = session.pattern;
  const observed =
    session.snapshot?.state === "ready" ? session.snapshot.sequencer : null;
  const disabled =
    !session.native ||
    !session.connected ||
    session.busy ||
    !selected ||
    !(session.snapshot?.state === "uninitialized" || observed);
  const boundary = observed?.lastBoundary;
  const edit = session.patternEdit;
  const occupied =
    session.snapshot?.sampler?.occupiedSlots ??
    session.preparation?.occupiedSlots ??
    0;
  const event = observed?.lastEvent;
  return (
    <section
      className="panel sequencer-panel"
      aria-labelledby="sequencer-heading"
    >
      <div className="sequencer-title">
        <div>
          <span className="eyebrow">Un compás · 4/4 · semicorcheas</span>
          <h2 id="sequencer-heading">Patrón · 16 pasos</h2>
        </div>
        <div className="actions">
          <button
            disabled={!session.native || !session.connected || session.busy}
            onClick={() => void session.refreshPattern()}
          >
            Consultar patrón
          </button>
          <button
            disabled={
              disabled || !selected?.steps.some((row) => row.some(Boolean))
            }
            onClick={() =>
              selected &&
              void session.setPattern({
                ...selected,
                steps: selected.steps.map((row) =>
                  row.map(() => false),
                ) as typeof selected.steps,
              })
            }
          >
            Vaciar patrón
          </button>
        </div>
      </div>
      <p className="muted">
        Activa pasos y elige un slot por fila. Carga un banco WAV, pulsa
        Inicializar secuenciador y después Play para escuchar el patrón en
        bucle.
      </p>
      {session.ready && !observed && (
        <p className="notice">
          Cierra audio e inicializa el secuenciador para editar este patrón
          durante la reproducción.
        </p>
      )}
      {edit && (
        <p role="status" data-testid="pattern-outcome">
          {edit === "offline"
            ? "Patrón seleccionado sin enviar comandos de audio."
            : `Cambio ${edit.commandId}: ${
                edit.status === "pending"
                  ? "aceptado; esperando procesamiento"
                  : edit.status === "processed"
                    ? "procesado; consulta el patrón observado"
                    : "aplicación desconocida; sesión cerrada o perdida"
              }.`}
        </p>
      )}
      <p data-testid="pattern-position" className="sequencer-position">
        {boundary
          ? `Último paso observado: ${boundary.step + 1} · vuelta ${(BigInt(boundary.loopIndex) + 1n).toString()}`
          : "Sin paso observado"}
      </p>
      <div
        className="sequencer-scroll"
        tabIndex={0}
        aria-label="Cuadrícula del patrón, desplazamiento horizontal"
      >
        <table className="step-grid">
          <thead>
            <tr>
              <th scope="col">Sample por fila</th>
              {Array.from({ length: 16 }, (_, step) => (
                <th
                  scope="col"
                  key={step}
                  className={step % 4 === 0 ? "beat-start" : ""}
                >
                  {step + 1}
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {Array.from({ length: 4 }, (_, row) => (
              <tr key={row}>
                <th scope="row">
                  <label htmlFor={`pattern-slot-${row}`}>Fila {row + 1}</label>
                  <select
                    id={`pattern-slot-${row}`}
                    disabled={disabled}
                    value={selected?.slots[row] ?? ""}
                    onChange={(change) =>
                      void session.setPatternRowSlot(
                        row,
                        Number(change.target.value),
                      )
                    }
                  >
                    {!selected && <option value="">Sin consultar</option>}
                    {Array.from({ length: 16 }, (_, slot) => (
                      <option key={slot} value={slot}>
                        Slot {slot}
                        {occupied & (1 << slot) ? " · cargado" : " · vacío"}
                      </option>
                    ))}
                  </select>
                  <small>
                    {observed
                      ? `Observado: slot ${observed.pattern.slots[row]}`
                      : "Sin patrón aplicado"}
                  </small>
                </th>
                {Array.from({ length: 16 }, (_, step) => (
                  <td key={step} className={step % 4 === 0 ? "beat-start" : ""}>
                    <button
                      type="button"
                      aria-label={`Fila ${row + 1}, paso ${step + 1}`}
                      aria-pressed={selected?.steps[row][step] ?? false}
                      aria-current={
                        boundary?.step === step ? "step" : undefined
                      }
                      data-applied={
                        observed
                          ? String(observed.pattern.steps[row][step])
                          : "none"
                      }
                      disabled={disabled}
                      onClick={() =>
                        selected &&
                        void session.setPatternStep(
                          row,
                          step,
                          !selected.steps[row][step],
                        )
                      }
                    >
                      <span aria-hidden="true">{step + 1}</span>
                    </button>
                  </td>
                ))}
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      <p className="muted step-legend">
        Relleno: seleccionado · punto: activo en el motor · marco: último paso
        observado. La posición puede saltar pasos entre actualizaciones. Los
        slots vacíos no producen sonido. En pantallas estrechas, desplaza la
        cuadrícula para ver más pasos.
      </p>
      {observed && (
        <p data-testid="pattern-counts">
          Notas observadas: {observed.counters.started} iniciadas ·{" "}
          {observed.counters.missingAsset} sin sample ·{" "}
          {observed.counters.poolFull} sin voz libre.
        </p>
      )}
      {event && (
        <p className="muted" data-testid="pattern-event">
          Última nota observada: fila {event.row + 1}, paso{" "}
          {event.boundary.step + 1}, slot {event.slot} ·{" "}
          {event.outcome.status === "started"
            ? "iniciada"
            : event.outcome.reason === "missingAsset"
              ? "rechazada: slot vacío"
              : event.outcome.reason === "poolFull"
                ? "rechazada: sin voz libre"
                : `rechazada: ${event.outcome.reason}`}
          .
        </p>
      )}
    </section>
  );
}
