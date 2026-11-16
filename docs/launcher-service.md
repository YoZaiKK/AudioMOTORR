# Launcher audio sessions — stage 13.3.a

The Rust audio service connects the [13.2 clip engine](clips-scenes-contract.md)
to explicit manual/CPAL sessions, a bounded shared command FIFO and applied
observations. [Project v2/scene export](project-format.md) follows in implemented
13.3.b; [application coordination](launcher-application.md) is implemented in
13.3.c and desktop exposure remains the next increment.
See the [stage report](../progress/etapa-13-sesiones-clips.md).

## Initialization and ownership

`AudioHandle` adds:

- `initialize_manual_clip_launcher(bank, tempo, mixer)`.
- `initialize_native_clip_launcher(device_id, bank, tempo, mixer)`.
- `request_clip_launch(ClipLaunchRequest) -> CommandAccepted`.

Initialization transfers an owned, prepared bank on the control side and
constructs `AudioEngine::with_clip_launcher`. It starts stopped and empty, with
no selected/active/pending clip and no implicit scene, trigger or Play. Mixer
and tempo are installed exactly as supplied. Native layout and rate must match
the bank; the existing supported-format preference applies, without resampling
or automatic device fallback. Feature-disabled native initialization returns
`NativeAudioDisabled`; Busy takes precedence while another session is running.

Requests snapshot validated clip content outside processing. Only fixed-size
Copy values cross the realtime FIFO: no paths, PCM, decoding or serialization.
Expired initialization requests discard their banks on the receiving control
thread. Manual shutdown reclaims PCM there. The existing native teardown mailbox
returns the owning processor after stream close, including callback destruction
on a backend worker. The worker does not free its PCM buffers.

## FIFO, processing and transitions

The launcher shares the existing 128-command queue, 64-snapshot queue and
32-command budget with transport, BPM, master gain, manual triggers and mixer
controls. Byte sizes grow to carry launcher payloads/observations; capacities
remain fixed. A scene is one atomic four-row `ClipLaunchRequest`, not a series
of separately accepted row commands. The control owner requires a ready launcher
session before enqueueing. Wrong modes/full queues consume no service command ID
and leave observed state unchanged. `ClipLaunchRequest` constructors validate
clip slot identities before a request can be submitted. Empty sample selections
remain legal; occurrences can report MissingAsset in the engine.

Drain controls once at callback start, before any native scratch chunks or due
notes. FIFO order is preserved across launcher/mixer/manual/transport commands.
Keep leaves an earlier pending row action intact. Later non-Keep actions replace
that row's pending action, including when their musical target is later.
All rows of a scene initially share one target; a later request may supersede
only part of that scene. Row Stop preserves natural tails. Transport Stop
cancels pending actions and resets phase only when the existing release settles.

`render_manual(0)` is a complete no-op, including command drain. A valid empty
native callback can process controls and publish a report without advancing
musical frames or emitting a note. Commands already processed before a render
error remain processed; whole-callback preflight prevents a partially rendered
valid prefix. No additional controls are drained between scratch chunks.

## Three observations and two ID sequences

| Observation                                     | Meaning                                                                                                                                     |
| ----------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------- |
| `CommandAccepted.command_id`                    | The control owner enqueued the request; it has not necessarily reached the engine.                                                          |
| `launcher.last_request`                         | The latest launcher command processed by the realtime owner: service `command_id` mapped to `Ok(engine_request_id)` or an engine error.     |
| `launcher.state.active/pending/last_transition` | The engine's owned content, row identities, request IDs, musical targets/origins and actually applied transitions after the whole callback. |

`ServiceSnapshot.launcher: Option<LauncherTelemetry>` contains the complete
`LauncherSnapshot` plus that latest `ClipLaunchReport`. It is Some only for a
launcher session; the old `sequencer` observation is None there. Each queued
launcher command creates at most one kernel request ID, independent of other
command kinds. Service command IDs remain monotonic across sessions; engine
request IDs restart with a new launcher engine. Neither is a voice handle.

An accepted kernel request can still be pending on a grid boundary. An all-Keep
request receives an ID without changing any active/pending row. The latest report
may therefore refer to a different request from the one currently playing.
Do not infer that a scene fired from `last_applied_command`, a matching clip slot
or a successful request report alone. The existing `last_applied_command` means
command processing, including a processed request whose result is an error.

Engine request-counter/target overflow is reported as a Copy CounterOverflow
result and leaves kernel state unchanged, without formatting or allocation in
processing. An internal launcher command sent to the wrong source mode is an
invariant failure: no false applied acknowledgment, silent rendering and the
existing stream-error path. Public callers cannot bypass the ready/source gate.

The phase, latest note outcome and counters come from the engine. Scheduled
notes do not receive service command IDs or overwrite `sampler.last_trigger`.
Manual trigger counts describe manual commands only; active voice counts include
both. Mixer peaks aggregate the complete callback, including every scratch chunk,
with existing mono/stereo meaning.

## Loss, errors and lifecycle

A full snapshot queue drops observations, never commands or musical events.
The next callback retries publication with current phase, cumulative counters,
pending/applied rows and latest request report. It cannot reconstruct intermediate
requests or note outcomes. If a requested command's report was lost, observers
must retain an inconclusive/Unknown result; never infer its kernel ID from a
later ID, compensate or replay the request automatically.

Shutdown, stream failure and failed native opening clear launcher observations.
A fresh session resets clips, phases, pending actions, reports, scheduled counts
and engine request IDs; it discards all previous queued commands. Service command
IDs and observation sequence retain their existing monotonic behavior. Session
reopening does not restore clip intent from a document in this increment.

## Verification and boundaries

Public service cases cover the acceptance/processing/transition distinction,
shared FIFO limits/budget, manual/scheduled ordering, Keep/supersession/scenes,
tempo/Pause/Stop, wrong modes, Busy, disabled-native behavior and reopening.
Internal cases cover observation loss/recovery, whole-callback overflow, internal
wrong-mode commands, service-ID exhaustion, failure/restart, expired-bank cleanup
and owner-side PCM reclamation.

Device-free native adapter references cover mono/stereo, scratch blocks
1/127/256/513 and f32/f64/i16/u16: 32 combinations across seven callbacks each.
They compare converted PCM, applied launcher/engine state, reports, counters,
manual telemetry and complete-callback meters against unsplit callbacks, including
the 33rd command waiting for the next callback. Instrumented processing covers
drain/render/conversion/publication without allocations/frees. A second matrix
checks device-free realtime processing and lost publication across six rate/channel
combinations, with positive PCM destruction checks outside processing.

These tests do not open audio hardware or establish physical timing/audibility.
The previous sampler/mixer/sequencer measurements are historical, not launcher
measurements. The kernel reference suite in 13.2 supplies independent musical
event/PCM references; this increment verifies the adapter around that kernel.
Project v2/scene export is implemented in 13.3.b and application in c; desktop is d, integration is e,
and [diagnostics/native protocol](launcher-diagnostics.md) is implemented in 13.4.a.
Physical measurements/listening and delivery closure remain 13.4.b/c.
