# Sequenced audio sessions — stage 11.3.a

Implemented locally on 2026-11-12 after the [11.2 engine](sequencer-contract.md).
The public Rust audio service now initializes sequenced sessions, queues pattern
edits and observes applied musical state. [Application coordination](sequencer-application.md)
is implemented in 11.3.b; [desktop exposure](sequencer-desktop.md) is implemented in 11.3.c.
See the [stage report](../progress/etapa-11-sesiones-secuenciador.md).

## Initialization and ownership

`AudioHandle` adds:

- `initialize_manual_sequenced_sampler(bank, tempo, mixer, pattern)`.
- `initialize_native_sequenced_sampler(device_id, bank, tempo, mixer, pattern)`.
- `set_pattern(pattern)`, `set_pattern_step(row, step, enabled)` and
  `set_pattern_row_slot(row, slot)`.

Initialization moves an owned bank plus validated Copy pattern/mixer settings
to the control owner, creating `AudioEngine::with_sequenced_sampler` outside
processing. It starts stopped with current supplied settings, no implicit note
or Play. Native device layout/rate must match the bank exactly; the existing
supported-format preference applies. No resampling or automatic device fallback.
Without `native-audio`, initialization returns `NativeAudioDisabled`. Opening a
second session while running returns Busy, including in feature-disabled builds.

No decoding, file access or heap-owning command payload enters processing.
Expired requests discard owned banks on the receiving control thread. Manual
shutdown reclaims PCM there. Native callback destruction returns its engine via
the existing teardown mailbox; PCM is reclaimed by the owner after stream close,
including when a backend worker releases the closure.

## FIFO and application boundary

Pattern edits use the existing Copy FIFO shared with transport, BPM, master gain,
manual triggers and mix controls. Capacities remain 128 commands and 64 snapshots,
with at most 32 commands drained once at callback start. The fixed payload byte
size grows to carry a complete pattern; capacities stay bounded. A replacement
pattern is one atomic command, never 64 separately applied cell edits.

The control owner validates ready/source mode and row/step/slot before enqueueing.
An empty but valid slot is a legal pattern selection; its scheduled occurrence
reports MissingAsset. Invalid edits and full queues preserve applied state and
consume no command ID. Acceptance only means enqueued; the next callback's
`last_applied_command` and applied pattern provide observations of processing.
Queue order applies before any events due at that callback's first frame. Notes
then follow the engine's within-block scheduler, without a command ID per note.
No commands are drained between native scratch chunks.

All scheduled and manual notes share the existing 16-voice pool. Manual triggers
processed at callback start precede scheduled notes. `SamplerTelemetry` retains
explicit trigger-command outcomes and started/rejected counts; active voices
include scheduled notes as well. Scheduled results never overwrite
`sampler.last_trigger` or acquire an unrelated manual command acknowledgment.

## Observations, clocks and loss

`ServiceSnapshot.sequencer: Option<SequencerSnapshot>` exposes the applied pattern,
exact musical position/fraction, last observed step boundary, lifetime scheduled
counts and latest scheduled event/outcome. It is Some only for sequenced sessions.
Next-frame position may have a pending boundary; `last_boundary` describes an
actually processed step, including empty steps. `last_event` is only the latest
event, not a lossless event history. Its frame and `(loop, step, row)` identity
describe the occurrence; a voice index is not a stable voice handle.

The snapshot is captured after the entire callback, covering every scratch chunk.
Mixer peaks still aggregate over the complete callback and retain mono/stereo
meaning. Musical frames exclude Pause/Stop release while the device/legacy
transport clocks retain their existing behavior. Stop resets musical position
and latest boundary/event only when settled, retaining scheduled lifetime counts.

Snapshot queue overflow drops observations, never commands or musical events.
The next callback retries publication with current phase, cumulative scheduled
counts and applied ID. A later snapshot cannot reconstruct missed intermediate
edits or individual outcomes. Observers must not infer an older event from the
latest one. Manual `render_manual(0)` is a complete no-op, including command drain.
A valid empty native callback retains existing behavior: controls can apply and
the callback count advances, but no musical frame or scheduled note is emitted.

Shutdown, stream failure or failed native initialization clears sequencer
observations. A fresh session uses a new bank and explicit pattern/settings,
resetting musical state, scheduled counters, voices and pending commands.
Service observation sequence and command IDs remain monotonic across restart.
Render errors after command draining preserve those commands' processing order;
whole-callback preflight prevents a partially rendered valid prefix. The control
owner presents the stream error and clears stale musical observations.

## Verification and remaining work

Public service tests cover installation, replacement/cell/slot FIFO order,
shared limits, invalid modes/indices, pending boundaries, tempo/transport,
reopening and the native-disabled gate. Internal tests cover lost snapshots,
invalid internal commands, whole-callback overflow, failed streams, expired
requests and owner-side PCM reclamation.

The native adapter is tested without opening hardware: mono/stereo ×
1/127/256/513-frame scratch × f32/f64/i16/u16, against an unsplit callback.
It compares actual PCM conversion, exact phase/event/counters, manual telemetry
and complete-window mixer peaks. Instrumentation covers drain, render,
conversion, aggregation and publication with zero allocations/frees, plus a
positive PCM teardown check on the owner.

Application-selected document coordination is [implemented in 11.3.b](sequencer-application.md);
generated DTOs/IPC/ACL and the grid are [implemented in 11.3.c](sequencer-desktop.md);
integrated acceptance is [complete locally in 11.3.d](sequencer-integration-acceptance.md). Native hardware diagnostics
and exact-revision CI close stage 11 in 11.4. Adapter tests are not hardware
timing measurements or an auditory acceptance.
