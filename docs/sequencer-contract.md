# Sixteen-step sequencer contract — stage 11.1

Specified 2026-11-12 after [stage 10 closure](../progress/cierre-etapa-10.md).
**11.1 implemented locally:** editable domain pattern and ordered step events.
**11.2 implemented locally:** engine scheduler, sequenced mixed rendering and
direct processing-owner edits. See the [engine report](../progress/etapa-11-planificador.md).
**11.3.a implemented locally:** [audio sessions, FIFO and applied observations](sequencer-service.md).
**11.3.b implemented locally:** [application coordination](sequencer-application.md).
**11.3.c implemented locally:** [DTOs/IPC/ACL and desktop grid](sequencer-desktop.md).
**11.3.d accepted locally:** [decoded beat and integration coverage](sequencer-integration-acceptance.md).
**11.4.a implemented locally:** [sequencer diagnostics and native protocol](sequencer-diagnostics.md).
**11.4.b–c pending:** native reference/measurements/listening and delivery acceptance.
The [11.1 report](../progress/etapa-11-modelo-patron.md) retains its earlier scope.

## Current engine API

`AudioEngine::with_sequenced_sampler(bank, tempo, mixer, pattern)` explicitly
enables sequencing and starts stopped. Existing constructors keep their previous
behavior. `set_pattern` replaces the applied pattern atomically; `set_pattern_step`
and `set_pattern_row_slot` validate individual edits before mutation. All are
direct processing-owner methods, without queue acceptance or document updates.
They return `SequencerControlError::NoSequencer` on other engine modes.

`sequencer_snapshot` returns Copy applied pattern, exact next-frame musical
position (loop, step and fractional numerator/denominator), last actually observed
boundary, lifetime counters and the latest event outcome. Counters distinguish
boundaries, attempted/started notes, missing assets and pool-full rejection.
`last_event` includes occurrence identity, row/slot, absolute device frame and
immediate voice result; the voice index is not a stable handle. This is only the
latest event, so a snapshot is not a complete history. Counters retain evidence
of earlier outcomes. `ServiceSnapshot.sequencer` now exposes this Rust snapshot
after complete callbacks. Application-selected pattern and acceptance are now
coordinated in 11.3.b and exposed through scoped IPC/UI in 11.3.c.

On settled Stop, phase, loop and last boundary/event reset; pattern, mix, tempo,
bank and lifetime counters remain. A new engine starts fresh counters. Render
preflight checks exact future boundary/event counts before any audio, meter or
voice changes, including when an overflow would occur later within the block.

## Document and musical resolution

`Project` owns one `Pattern`, separately from `Tempo` and `MixerSettings`.
The pattern is fixed, validated `Copy` state, with four rows and 16 boolean
steps per row. One step is a sixteenth note: four steps per quarter-note beat,
four beats per bar, one bar per pattern. Playback repeats indefinitely until
transport changes. There is no selectable time signature, swing, variable
length, velocity, probability, note-off, song arrangement or finite repeat count
in this increment. Finite export and sample tails belong to stage 12.

Each row selects one of the existing 16 bank slots. Defaults: every step off,
row slots 0, 1, 2, 3. All slots, including empty ones, are legal selections;
the document has no PCM or dependency on the currently prepared bank. Rows may
share a slot. An enabled cell requests one unity-gain one-shot at that step
boundary. Disabling a cell creates no event and sends no note-off.

`Pattern::step` / `set_step` use zero-based row and step indices. `row_slot` /
`set_row_slot` select the asset. Invalid rows, steps or slots return
`PatternError` before mutation; row validation precedes step/slot validation.
`events_at(step)` enumerates at most four `StepEvent` values in ascending row
order, each containing row, step and slot. Duplicate slots remain separate
events. Enumeration allocates nothing, has no playback cursor and does not
claim that a voice started. `Project::pattern_mut` edits selected state only.

Rows are musical lanes, not additional PCM channels or voice pools. Default
slots and default mixer routing align each row with one mixer strip, but the
existing slot-to-channel routing remains authoritative. Changing a row's slot
does not change that slot's mixer destination. Successful triggers capture the
applied mixer destination as before. Four rows share the existing 16-voice pool.

## Event identity, timing and rounding — implemented in 11.2

The engine, never a UI timer, advances musical phase. A scheduled occurrence
has `(loop, step, row)` identity, a selected slot and a frame offset inside the
current interleaved output block. Offsets count frames, not scalar PCM samples.
At constant tempo `B` and sample rate `F`, ideal boundary `k` occurs at
`k * F * 15 / B` musical frames. Step is `k % 16`, loop is `k / 16`.
Quantize each absolute boundary with **ceiling**, so an event is never early.
Do not round a step duration and repeatedly add it: that accumulates drift.

Example at 48000 Hz / 120 BPM: step 0 at frame 0, step 1 at 6000,
step 15 at 90000, next loop's step 0 at 96000. At 127.25 BPM, the first
nonzero ideal boundary is `2880000 / 509`, so it fires at frame 5659.
Partitioning the same playing interval into blocks of 1, 127, 256 or 4096
frames must preserve every occurrence and the resulting PCM.

Use half-open render windows `[start, end)`. A boundary at `end` belongs to
the next nonempty render, at offset zero. Empty renders emit nothing and
advance no state. Multiple boundaries quantized onto the same frame must all
be processed, ordered by musical boundary and then row. This includes very low
synthetic sample rates accepted by the engine. Events are generated incrementally;
there is no heap event list or callback-size event cap.

`Tempo` retains finite 20–300 BPM, including fractional values. Timing uses the
exact selected IEEE-754 value with carried fractional phase. A bounded integer
representation is possible: all valid tempos are integer multiples of `2^-48`
BPM. With `Q = 2^48`, one step has phase denominator `15 * F * Q`, and each
playing frame adds the integer `B * Q`. `u128` accommodates this denominator
and phase at all accepted `u32` sample rates. A pending-boundary marker prevents
re-emission at frame zero or after a block/command boundary. Check occurrence,
loop and frame counters before committing a render; overflow/alignment errors
silence output and leave engine state unchanged, as existing render errors do.

At a BPM change, preserve the musical phase and already emitted occurrences.
Only future phase increments use the new BPM; do not reinterpret elapsed frames
at the new tempo, restart a bar or duplicate the current step. Several accepted
tempo commands before the next frame leave the last tempo active. Tests must
cover fractional tempos, tempo edits, exact boundaries and long-run drift.

## Transport and voice lifecycle — implemented in 11.2

Sequenced mixed playback is explicit opt-in. Existing silence, sine, raw sampler
and manually triggered mixed sessions retain their behavior.

| Action                                 | Sequencer behavior                                                                                                                                                                 |
| -------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Initialize                             | Stopped, zero musical phase, first step pending; no implicit Play or trigger.                                                                                                      |
| Play from settled Stop                 | Step 0 is due before the first nonempty playing frame; start the existing master attack.                                                                                           |
| Play while playing                     | Continue phase; do not retrigger step 0 or the current step.                                                                                                                       |
| Pause request                          | Immediately freeze musical phase and stop creating scheduled events at the command boundary. Existing voices finish the master release, then freeze their cursors.                 |
| Play after Pause                       | Resume preserved phase and pending boundary once; continue retained voices through master attack. Release frames did not advance musical phase.                                    |
| Stop request                           | Stop scheduling immediately; existing voices finish the master release. On settled Stop clear voices, reset phase/loop and make step 0 pending. Keep bank, pattern, mix and tempo. |
| Play before Pause/Stop release settles | Cancel the requested release through existing transport behavior; resume preserved musical phase. A Stop resets the pattern only when it actually settles.                         |
| Pause/Stop already silent              | Apply the corresponding settled behavior immediately, including Stop reset.                                                                                                        |

The existing `device_frames` clock always advances. The existing
`transport_frames` clock still includes playing/release frames. Musical phase
is separate because Pause/Stop release produces audio without new beat events.
At fully silent master/channel settings, scheduling and voice aging still run
while requested transport is Playing. Muting does not pause the pattern.

Triggers occur before rendering their assigned frame; natural completion from
earlier frames may free a voice. At a shared frame, process ordered events before
rendering that frame, so an earlier accepted event consumes its voice first.
Missing assets or a full pool reject only that occurrence; do not steal voices,
retry it later, abort the pattern or reset phase. Engine snapshots expose
counters and the latest outcome; [service telemetry/loss handling](sequencer-service.md)
is implemented in 11.3.a. Application handling is implemented in 11.3.b;
desktop handling is implemented in 11.3.c.
Internally scheduled events do not acquire application command IDs or enqueue
one command per note. Any explicit manual triggers retain FIFO application order
before scheduled events at the callback's first frame.

## Live edits and selected/applied state — required in 11.2/11.3

Direct processing-owner edits apply between engine renders. The service drains
its existing bounded FIFO once at the start of a callback, before planning any
of its scratch chunks; it must not apply edits between those chunks. Pattern,
slot, tempo, mix and transport commands retain their FIFO order. Pending edits
apply before an event due at the first frame of that callback. Already emitted
occurrences never fire again because of an edit.

Step changes affect future occurrences, including a not-yet-emitted boundary
at the next frame. Turning on an already passed step waits for the next loop.
Turning off a step leaves existing sample tails alone. Row-slot changes affect
future triggers; active voices retain their original PCM and mixer destination.
No pattern edit changes musical phase. Edits while stopped/paused are retained
for the next Play. A complete pattern replacement must be one atomic Copy edit,
not a partially applied 64-cell batch.

Application-selected document, queue acceptance and engine-applied state remain
distinct. Queue-full/validation errors preserve the document; lost observations
must not invent an acknowledgment or compensate an inconclusive operation.
New sequenced sessions receive current selected pattern, tempo and mix explicitly.
An observed playhead describes engine progress, including silent/all-off steps;
the UI must not infer an applied edit or advance a playhead from a timer.
Persistence and a saved transport position remain outside stage 11.

## Acceptance for the next increments

- 11.1: all 64 cells, 16 slot selections per row, default silence, ordered
  duplicate events, invalid edits without mutation and independent project copies.
- 11.2: independent rational event/PCM references, several tempos and partitions,
  boundaries/loops, coincident events, tempo/step/slot changes, release/resume,
  missing assets, full pool, counters and zero allocation/free in processing.
- 11.3: bounded command acceptance/application, preparation/lifecycle recovery,
  generated bindings/ACL, four-row grid and observed position with loss handling.
  [Local integration acceptance](sequencer-integration-acceptance.md) now adds
  a decoded beat reference, coverage audit and current regression checks/build.
- 11.4: explicit sequencer diagnostics, reference beat on the native path,
  relevant measurements/listening and exact-revision CI before closing the stage.
