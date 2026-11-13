# Application pattern coordination — stage 11.3.b

Implemented locally on 2026-11-12 after [sequenced audio sessions](sequencer-service.md).
The application coordinates the editable Project pattern with service acceptance
and observed engine state. [DTOs, IPC, ACL and the desktop grid](sequencer-desktop.md)
are implemented in 11.3.c; integrated acceptance remains 11.3.d.
See the [stage report](../progress/etapa-11-aplicacion-secuenciador.md).

## Rust application API

- `get_pattern()` returns a Copy snapshot of the selected Project pattern.
- `sequencer_set_pattern(pattern)` replaces the selected pattern atomically.
- `sequencer_set_step(row, step, enabled)` changes one validated cell.
- `sequencer_set_row_slot(row, slot)` changes one validated sample selection.
- `initialize_manual_sequenced_sampler(preparation_id)` consumes a ready bank
  and installs the latest selected pattern, mixer settings and project tempo.
- `initialize_sequenced_sampler(preparation_id, device_id)` performs the same
  installation using the native service and its exact layout/rate/device policy.

Existing silence, sine, raw sampler and manual mixed-sampler APIs keep their
behavior. Reading or selecting a pattern does not open audio, trigger a note or
issue Play. Mixer edits and transport/tempo remain shared with the sequenced
session; manual trigger outcomes stay separate from scheduled note outcomes.

## Selected, accepted and applied state

The three edit methods return Copy `PatternEdit { pattern, accepted }`.
`pattern` is selected document intent at that operation, not an assertion of
current playback. `accepted: Option<CommandAccepted>` has explicit semantics:

| Observed service state                                               | Selection                                                       | Receipt / result                                                                        |
| -------------------------------------------------------------------- | --------------------------------------------------------------- | --------------------------------------------------------------------------------------- |
| Uninitialized, including loading/cancelling or a ready prepared bank | Commit validated document edit.                                 | None; no ID or session invented.                                                        |
| Ready sequenced session                                              | Enqueue, then commit document only after successful acceptance. | Some(id), not playback confirmation.                                                    |
| Ready silence, sine, raw sampler or manually mixed session           | Preserve document.                                              | InvalidArgument; close audio or explicitly initialize a sequenced session.              |
| Failure, request error or closing service                            | Preserve document.                                              | Preserve original error code/detail; closing is Busy, unexplained Error is StreamError. |
| Invalid indices or full queue                                        | Preserve document and unrelated tempo/mixer selection.          | InvalidArgument / QueueFull, without a consumed command ID.                             |

Sequential edits build on **selected** document state, which may already include
accepted commands that have not reached the engine. Building on an older applied
snapshot would lose those edits. Whole-pattern replacement uses one FIFO command;
later cell/slot edits follow it in order. Returned selections are independent
Copy values. Application mutations require mutable access and are serialized by
the caller's existing application ownership.

Read `get_engine_snapshot().sequencer` and `last_applied_command` for actual
pattern, musical position, scheduled counters and latest event. Manual zero-frame
renders do not apply pending edits. A newer observation cannot prove the exact
intermediate value of an earlier command. Notes rejected for MissingAsset or
PoolFull do not undo an accepted pattern selection; acceptance applies the edit,
not a guarantee that every future note can start.

A request timeout is inconclusive: the service might have queued the edit while
the caller did not receive acceptance. The application keeps the selected document
unchanged when acceptance is unconfirmed, sends no compensating command and
invents no acknowledgment. Query observed state before an explicit retry or
reconciliation. A deterministic test models a lost reply after real enqueueing,
then proves that the engine can apply the command while the document stays old.

## Preparation, installation and restart

Pattern edits are available while preparation owns a worker, is cancelling, holds
a ready bank or reports a preparation error, as long as audio is uninitialized.
They do not poll/wait for the decoder, change preparation IDs/status/metadata or
touch PCM. Cancellation and loading failures retain selected pattern/mixer state.

Installation reuses existing preparation-ID, ready-bank and audio-state gates.
Stale/cancelled/failed/missing banks cannot initialize a session. Busy does not
replace the existing session. A valid bank is consumed on success or native
opening failure; no source path is reopened, PCM copied or preparation-time
pattern substituted. Latest settings are read at installation under the caller's
exclusive application access. The engine starts stopped with fresh observations
and no implicit command acknowledgment.

Shutdown discards pending engine commands but retains confirmed document intent.
Repreparing and reopening installs that intent explicitly, including accepted
pattern/mix/BPM changes that never reached a callback. Old commands and IDs are
not replayed; service command IDs continue monotonically. Project tempo retains
the existing transport acceptance policy; this increment adds no offline tempo
setter or persistence format.

## Verification and limits

Real-WAV application tests cover selected/applied separation, multiple pending
edits, atomic replacement, shared FIFO limits, routing/paused voices, fractional
BPM, Stop/restart, unknown assets/pool exhaustion, invalid/wrong-mode edits,
stale/cancelled/failed preparation and native-disabled failure without fallback.
Sources are deleted before installation to prove reuse of immutable prepared PCM.
A synchronized worker test covers Loading and Cancelling without decoder waits.

These are control-side use cases; requests and errors may allocate. Engine,
audio-service and callback sources are unchanged from 11.3.a. No new hardware
audition or callback timing measurement is implied. Desktop exposure is implemented in 11.3.c,
integrated acceptance is [complete locally in 11.3.d](sequencer-integration-acceptance.md),
with native/delivery closure remaining in 11.4. Project persistence
and beat export remain stage 12.
