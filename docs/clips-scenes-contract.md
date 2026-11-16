# Clips and scenes — stage 13 contract

Stage 13.1 implements the standalone `daw-domain` document types. Stage 13.2
implements opt-in engine scheduling and direct processing-owner controls, with
independent event/PCM references and instrumented processing memory checks.
Stage 13.3.a connects [audio sessions, shared queues and observations](launcher-service.md).
Stage 13.3.b adds [Project/file v2 and independent scene export](project-format.md).
Stage 13.3.c adds [application document/session coordination](launcher-application.md).
Desktop UI remains the next increment, 13.3.d;
integrated/native verification and delivery closure are 13.4. The current desktop
still presents the legacy pattern for Pattern mode. Clip playback and saved v2
clips are available through Rust APIs and scene export through CLI; desktop clip
editing/playback is not yet exposed.

## Document model — implemented in 13.1

- Four launcher rows, each with 16 stable clip slots, and 16 stable scenes.
  Slots are zero-based identities; deletion never compacts another slot.
- A `Clip` owns one bar of 16 on/off sixteenth notes and one of the 16 sample
  slots. It is one row of the existing `Pattern`, with unity trigger gain.
  Empty clip slots and populated clips with all notes off are distinct.
- A `Scene` has exactly four actions: `Keep`, `Stop`, or `Launch(clip_slot)`.
  References are local to the corresponding row. Default actions are `Keep`.
  Launching a scene replaces all non-Keep rows at the same musical boundary.
- `ClipLauncher::set_scene` validates all references before replacement. An
  empty referenced slot is an error. Removing a referenced clip is an error;
  first edit the referring scenes. Replacing its content preserves its identity.
  Invalid rows/slots/scenes/cells reject without mutation; row validation comes
  first, then slot/step. A scene index is checked before its references.
- `resolve_scene` is a pure projection from current owned clip snapshots to
  new owned snapshots. Keep preserves one, Stop clears one, Launch copies one.
  This is neither a pending-request store nor a playback cursor.
- `from_pattern` copies all four rows into slot 0 and configures scene 0 to
  launch them. All other slots are empty and other scenes Keep. Project ID,
  BPM, mixer, assets and the original pattern are untouched. `pattern_from_clips`
  is a shared-phase projection for fixtures: absent rows have disabled notes
  and default row slots. It does not represent independent row launch phases.

Row identity is not a mixer destination: a launched clip chooses a sample slot;
the existing slot route chooses its mixer channel. The global pool remains 16
voices, with existing headroom, ramps, pan law and newest-note rejection. Clips
own neither PCM nor a separate tempo, mixer, gain, device or sample bank.

The entire model is fixed-size, Copy and independent of OS/audio/serialization.
It has no mutable alias to an accepted launch. Editing a slot affects future
requests; it cannot retroactively edit a snapshot already accepted or playing.
Live clip-content updates require an explicit relaunch in this first version.

## Quantization and transition rules — implemented in 13.2

`LaunchQuantization` defines Step (1 sixteenth), Beat (4 sixteenths) and Bar
(16 sixteenths, default). All grids are anchored at transport musical zero,
using the exact rational phase/BPM and absolute-frame ceiling rules of the
[sequencer contract](sequencer-contract.md). Do not sum rounded step lengths.
The engine represents local clip phase by an integer origin step per playing row
on the global exact clock: local phase is global phase minus that origin,
wrapping after 16 steps. All supported grids land on integer steps, so sharing
the global fraction preserves independent row launch positions exactly.
At a launch, reset that row to step 0 at the transition phase.
Keep rows preserve local phase. A launch of the already playing clip restarts it.

A request accepted at a callback boundary chooses the earliest matching grid
boundary that has not yet been processed. A boundary assigned to the next frame
is eligible only if still pending; an already emitted boundary is never replayed.
This includes a pending boundary slightly behind the next-frame phase due to
frame ceiling. With no such pending boundary, a phase between grid boundaries
waits for the next one. Several musical
boundaries may share a frame at very low rates; process in exact musical order.
For example, at musical step 5.25, Beat targets step 8 and Bar targets step 16;
a request at step 8 before that boundary is processed may target step 8.

Processing order at each boundary is: apply due row transitions, then emit
newly due row notes in row order, then render the frame. A switched/stopped row
does not emit its outgoing note at that boundary. Incoming step 0 emits once;
the next bar of that clip emits its next step 0 once. Render windows remain
half-open: an event at the block end belongs to the next nonempty block.

Row Stop cancels future notes for that row but lets existing one-shots finish
naturally; switching clips does likewise. Voices retain their original sample
and captured mixer route. There is no crossfade or voice stealing. Missing
samples/full pool reject the occurrence using existing counters without retry,
phase reset, partial scene application or cancellation of another row.

At most one pending action per row is retained. Requests share the existing
bounded command FIFO and its once-per-callback command budget. A scene is one
atomic Copy command containing four resolved actions/snapshots and a shared
quantization. Scene validation and queue acceptance happen before mutating
selected intent. Later commands replace pending actions only for their non-Keep
rows, including when their target boundary is later. Keep does not cancel an
earlier pending request. Stop-all replaces all four pending actions. FIFO order
decides conflicts; no notes or heap-owned resources travel through the queue.
All target/counter/frame arithmetic is checked before render state mutation.

The engine implements these bounds and replacement rules for direct requests
in call order. Stage 13.3.a adds the shared service FIFO/budget and applied request
reports. Selected-document coordination is implemented in [13.3.c](launcher-application.md).

## Direct engine API — implemented in 13.2

`AudioEngine::with_clip_launcher(bank, tempo, mix)` owns the same mixed sampler,
fixed voice pool and master ramp, starts stopped with no active clips, and is
separate from the existing sequenced-pattern constructor. `sequencer_snapshot`
is None in clip mode; `launcher_snapshot` is None in all previous modes.

`ClipLaunchRequest::new` validates all four owned Keep/Stop/Launch payloads and
their clip identities before returning a Copy request. `from_scene` snapshots
validated document content while retaining Keep as an action. Do not use
`resolve_scene` to build a launch command: it projects current active content
and would erase the distinction between Keep and relaunch.

`request_clip_launch` applies between renders, returns a monotonically increasing
nonzero processing-owner request ID, and rejects wrong mode or arithmetic
overflow without mutation. These IDs are distinct from future service command
IDs. Even an all-Keep request receives an ID, with no row transitions. While
stopped, non-Keep actions select immediately without sound; otherwise each row
stores at most one pending action. Its target is the first multiple of the grid
at or beyond the clock's next unprocessed boundary ordinal. A later request
replaces that row's pending action regardless of the two target times.

Snapshots expose owned active clip content/identity/request ID/origin, pending
content/ID/target, last applied transition per row, last accepted ID, lifetime
transition/note counters, exact global phase and last observed boundary/event.
`row_step` projects the local phase at the next frame, which may still have
pending boundaries. The last event records local clip step, global boundary,
sample/clip slot, row, device frame and actual trigger outcome. It is one latest
event, not an event history or a service acknowledgment.

Whole-block preflight checks clock/frame/boundary counters, exact enabled-note
counts before/after each pending transition and transition counters, before
touching PCM, ramps, voices or meters. Counting is bounded by four rows and one
16-step period per segment, independent of rendered duration. Errors silence
the output and preserve all state; empty output consumes no transitions/events.
The existing adapter `validate_render` entry point includes the new mode.

Settled Stop resets global/local positions and clears last boundary/event/
transition observations and pending actions, retaining active snapshots,
accepted-ID sequence, lifetime transitions and note counters. A request submitted
during Stop release can wait for an interrupted-stop resume; if Stop settles,
that pending action is discarded with the rest of transient scheduling state.

## Transport and observation — engine and service implemented

| Action                                | Required behavior                                                                                                                                                      |
| ------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Initialize                            | No active clips, no pending launches, stopped. No automatic scene or Play.                                                                                             |
| Launch/row Stop while settled stopped | Select/clear active row snapshots immediately, reset affected local phases, emit no notes; next Play starts selected clips at step 0.                                  |
| Launch while playing                  | Schedule using the chosen global grid; transport continues.                                                                                                            |
| Pause                                 | Freeze global/local phases and pending musical targets immediately; retain voices through existing master release/freeze behavior.                                     |
| Launch while paused or releasing      | Retain a pending quantized action for resume; no implicit Play.                                                                                                        |
| Play/resume                           | Resume phase and pending transitions once; master attack follows existing behavior.                                                                                    |
| Transport Stop                        | Immediately cancel pending actions and stop scheduling; on settled Stop reset global/local phases and clear voices, retaining active clip snapshots for the next Play. |
| Play before transport Stop settles    | Resume frozen phases with pending actions already cancelled; reset phases only if Stop settled.                                                                        |
| Tempo change                          | Preserve phases and pending musical targets; recompute future frame placement from the new exact increment.                                                            |
| Scene/clip document edit              | Change selected document only; playing/accepted snapshots remain owned and unchanged until explicit launch.                                                            |
| Session close/reopen                  | Discard live/pending state and command observations. New session starts stopped with no active clips.                                                                  |

Expose selected document, accepted request and observed applied rows separately.
Pending and active observations include row clip identity plus an applied revision
or content snapshot, local position and target musical phase. Do not infer
application from a matching slot alone: the same slot may have been edited or
relaunched. Retain exact IDs and the existing inconclusive/Unknown behavior for
lost telemetry. A UI timer neither schedules notes nor confirms transitions.
Per-row scheduled actions must remain distinguishable from per-note outcomes;
an accepted scene does not promise that every note will find a free voice.

## Persistence and export — implemented in 13.3.b

`Project` now owns `ClipLauncher`, playback mode, quantization and initial scene.
The [project format](project-format.md) writes v2 and reads v1/v2, with exact
numeric round trips and complete graph validation. The original 13.1 model was
standalone until the persistence adapter could preserve its state in this step.

13.3.b introduces an explicit version-2 schema, storing fixed clip/scene slots,
selected quantization and an explicit
initial scene for deterministic export; keep the legacy pattern for the existing
pattern mode. Add explicit pattern/launcher playback mode. Migration from v1
preserves ID/title/BPM/mix/routes/assets/pattern exactly, constructs scene 0 with
`from_pattern`, sets initial scene 0 and Bar quantization, and retains pattern
mode. V2 readers validate the whole graph before replacing selection; older v1 readers
continue to reject v2. Do not write v2 fields under version 1 or down-save clips.
Retain the 1 MiB document cap and existing relative-asset/copy/commit guarantees.

Never persist active voices, pending launches, applied observations, live phases
or command/session IDs. Saving edits does not capture a performance. Launcher
export explicitly chooses a scene, starts that scene from silence at musical
zero, repeats for the requested bars and applies the existing cut/finish policy.
It snapshots scene content/mix/BPM/assets and reuses the same engine as live
playback. Keep from silence remains inactive. It does not export UI launch
history. A timed arrangement/performance export is future work. Existing pattern
mode export remains supported; CLI `export-beat --scene 0..15` explicitly selects
a scene. [13.3.b verification](../progress/etapa-13-proyectos-escenas.md) covers
migration, relocation, strict graphs and independent scene PCM/duration/tail references.

## Acceptance and next deliveries

13.1 domain tests cover fixed bounds/defaults, all cells/sample slots, every
row/clip/scene identity, atomic invalid edits and missing references, safe removal,
Keep/Stop/Launch resolution, duplicate sample events, owned snapshots and exact
pattern import/projection. This proves document semantics, not audio scheduling.

13.2 must compare independent rational event/PCM references for Step/Beat/Bar,
exact/just-before/just-after boundaries, same-clip relaunch, independent row
phases, atomic scenes, supersession/Keep/Stop, coincident boundaries, tempo
changes and Pause/Stop/resume. Compare partitions 1/127/256/4096/full, fractional
BPM and low rates. Cover missing assets/full pool, overflow without mutation,
and instrumented zero allocations/frees in processing with owner-side teardown.

These engine checks are implemented and recorded in the
[13.2 report](../progress/etapa-13-transiciones-motor.md). They do not measure
physical callback timing or establish launcher audibility on hardware.

13.3 adds bounded service/application commands, explicit launcher sessions,
generated DTO/IPC/ACL, grid and observed phases, queue rollback/loss/reopen,
v1 migration/v2 round-trip/relocation/invalid graphs and exact scene-export
durations/tails/partition equivalence. Split these into reviewable increments.
13.4 adds integrated references, affected regressions/check/build, explicit
launcher native diagnostics/listening, evidence audit and exact-delivery CI.
Existing sequencer measurements are historical evidence for that earlier path.
