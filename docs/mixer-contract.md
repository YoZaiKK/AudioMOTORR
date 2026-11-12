# Four-channel mixer contract — stage 10.1

Specified 2026-11-11 after [stage 9 closure](../progress/cierre-etapa-9.md).
**Core implemented locally in 10.2:** document state, direct engine controls,
mixing, ramps and per-render peaks. **10.3.a adds mixed audio sessions, queued
controls and whole-callback meter aggregation**; see the
[service contract](mixer-service.md). **10.3.b coordinates application selection,
acceptance and installation**; see the [application contract](mixer-application.md).
[Desktop integration 10.3.c](mixer-desktop.md) exposes these contracts through
IPC/ACL/generated DTOs and four control strips. See the [10.2 report](../progress/etapa-10-mezclador-motor.md).
[Integrated acceptance 10.3.d](mixer-integration-acceptance.md) now verifies
decoded four-channel references and lifecycle/UI regressions locally.
Stage 10.2 implements and tests the engine/document state; 10.3 connects the
service/application/desktop; 10.4 verifies the integrated delivery.

The current API uses `MixerSettings`/`MixerChannel` in `daw-domain` and explicit
`AudioEngine::with_mixed_sampler(bank, tempo, settings)`. `Project::mixer_mut`
edits the document only. Processing owners call `set_mixer_channel` and
`set_mixer_route` directly; `mixer_snapshot` returns selected applied settings,
effective ramp values, captured active-voice destinations and per-call meters.
These engine methods are direct processing-owner edits. AudioHandle now queues
the corresponding service operations through the existing FIFO; its snapshot
reports complete-callback windows while the direct engine API stays per-render.

## Scope and ownership

Exactly four musical channels feed the existing master output. The sample bank
retains 16 immutable slots and a **single global pool of 16 voices**. Four channels
do not create four independent voice pools. Bank ownership, decoder budgets,
sample-rate matching and owner-side PCM teardown retain the sampler contract.

Use fixed channel state, a fixed slot-to-channel table and fixed voice metadata.
The editable document owns selected routing/gain/pan/mute; the engine owns their
applied state and ramps. A new session receives the document settings explicitly.
Project persistence is stage 12. No bus graph, effects, solo, sends or dynamic
channel creation belongs to this increment.

Existing silence/sine and raw `Sampler` APIs retain their current behavior.
The mixed sampler is opt-in; its changed summation topology is verified separately
instead of silently changing historical raw-sampler reference samples.

## Routing and voice lifecycle

- Channels are indexed 0–3; sample slots remain 0–15. Initial routing is
  `channel = sample_slot % 4`, distributing the first four slots across four channels.
  Initial channel settings are gain 1, pan 0 and mute false.
- Every slot has exactly one selected destination. Invalid indices reject the
  change before mutation. Routing may be edited while playing.
- Capture the slot's **applied destination at successful trigger time** into its
  voice metadata. Routing edits affect future triggers; active/paused voices
  retain their destination until completion or Stop.
- The lowest free global voice is selected; reject the newest trigger on pool
  exhaustion. Muted or zero-gain voices still advance, occupy their slot and
  complete normally. Muting must not freeze or destroy voices/PCM.
- Gain/pan/mute edits affect all active voices assigned to that channel through
  the channel processing stage. Voice trigger gain remains fixed per one-shot.
- Requested/actual transport, master release, Pause cursor preservation and Stop
  reset/PCM retention keep the existing engine semantics.

## Gain, panorama and output policy

Channel gain is finite in [0, 1]; pan is finite in [-1, 1]; mute is boolean.
Reject non-finite/out-of-range values rather than clamping them. The master
gain and per-trigger gain retain [0, 1]. There is no gain boost or limiter here.

Use a **linear balance law with unity at center**, consistently for mono and
stereo sources. For a settled pan value `p`:

```text
left(p)  = 1              if p <= 0, otherwise 1 - p
right(p) = 1 + p          if p <  0, otherwise 1
```

Mono is duplicated to the output channels before this law; stereo preserves its
left/right source channels. Hard left/right attenuates the opposite side to zero;
center leaves both unchanged. This is not a constant-power panner. The choice
preserves existing centered sampler levels and never boosts either side.
For mono output, ignore pan in DSP while retaining its selected/applied value;
stereo inputs remain invalid for a mono bank/output under the existing contract.

For each output frame, in this exact order:

1. Sum source sample × trigger gain into each destination channel, visiting
   active voices in ascending global voice-index order. Use f32 processing.
2. Apply the channel's effective gain and left/right balance coefficients.
3. Apply the fixed `1/16` headroom to each channel contribution.
4. Sum the four contributions in ascending channel-index order.
5. Apply the existing master envelope once per output frame.

With at most 16 normalized voices and all processing coefficients within [0, 1],
the mathematical full-scale bound remains 1. Floating-point comparisons use a
documented small tolerance for rounding. Do not add per-channel `1/4` attenuation,
hard clipping or normalization that depends on the current active-voice count.
Channel grouping can change f32 rounding relative to the historical raw voice
sum; partition invariance is required within the new processing topology.

### Control smoothing

Use frame-based linear ramps of `ceil(sample_rate / 200)` frames (5 ms),
retargeted from the current applied value. Gain/mute share one effective-gain
ramp: target = selected gain when unmuted, otherwise zero. Changing selected
gain while muted leaves the effective target zero; unmute targets the latest
selected gain. Pan retargets left/right coefficients with the same frame duration.

A channel starts with its selected coefficients already applied; the existing
master attack handles playback onset. While Play/release renders active audio,
channel ramps advance once per rendered frame. When transport is fully stopped
or paused with zero master gain, snap channel coefficients to selected targets
so edits made in silence apply on resume. Empty buffers never advance ramps.
No trigonometry, allocation, resource destruction, locks or I/O occurs per frame.

## Observation and meters

Keep selected document state, queued command acceptance and observed applied
state distinct. Mixer controls use the existing bounded FIFO and monotonic
command IDs. Queue overflow rejects a command without changing applied state;
observation loss must not be reported as confirmation of a particular command.
Batch all four channel settings into a fixed Copy payload when installing a
session; normal edits can use individual validated fixed-size commands.

Expose fixed snapshots for four channels and master. Each reports the peak
absolute value of its **actual rendered contribution**: channel peaks after its
gain/pan/headroom, before summing/master; master peak after master envelope.
Use two peak values for stereo and one meaningful value for mono; mark the other
unused instead of representing a second physical channel. No RMS or peak hold
is required in this increment.

Measure over a whole backend callback, including all internal chunks, and reset
the window only at the next callback boundary. Manual/offline rendering reports
its explicit call window. Expose measured frames and a monotonic observation
sequence so consumers know the window; an empty render preserves meter state.
The UI may miss intermediate callback peaks when snapshots are coalesced. Display
smoothing is a visual policy and must not alter the measured values or audio.
During master release, channel contributions can remain nonzero while master
falls; after complete Pause/Stop, rendered silence reports zero.

## Layer boundaries

| Layer         | Responsibility                                                                                                                       |
| ------------- | ------------------------------------------------------------------------------------------------------------------------------------ |
| Domain        | Fixed editable four-channel state and slot routing with validated values. No filesystem, backend or UI dependencies.                 |
| Engine        | Voice destination capture, deterministic fixed-state mix, ramps and meter accumulation. Raw sample resources stay immutable.         |
| Audio service | Bounded command submission, processing once per callback, complete-callback meter windows and Copy observations.                     |
| Application   | Validate/edit document, explicitly apply changes, initialize sessions from selected settings and preserve failure/restart semantics. |
| Desktop       | Generated DTOs/ACL/IPC, four control strips and applied-state/level feedback; native operations disabled in browser mode.            |

## Required reference cases for implementation

Use independent expected samples and event schedules; do not derive the oracle
by calling the implementation under test. At settled gain/master values:

| Case                                                                   | Reference                                                                                        |
| ---------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------ |
| One mono voice with sample 1, trigger gain 1, channel gain 1, centered | Stereo contribution `(1/16, 1/16)`.                                                              |
| Same voice with channel gain 0.5 and pan +1                            | `(0, 1/32)`.                                                                                     |
| Stereo sample `(0.8, -0.4)`, centered channel gain 1                   | `(0.8/16, -0.4/16)` within the declared f32 tolerance.                                           |
| Sixteen identical full-scale mono voices, all settings at unity/center | Both output sides approximately 1, regardless of their channel assignment.                       |
| Opposite channel contributions                                         | Master can cancel while both channel meters remain nonzero.                                      |
| Muted channel after release settles                                    | Zero contribution; voice cursors still advance and global capacity is retained until completion. |

Also cover invalid values without state mutation; every slot/channel route;
rerouting while playing/paused; 16-voice exhaustion across all channels;
gain/pan/mute ramp endpoints and mid-ramp retargets; edits while muted/paused;
mono behavior; source completion; Stop/retrigger and shutdown ownership;
unaligned/overflow error rollback; empty and partial blocks. Compare samples and
final state across partitions 1/127/256/4096 with identical frame-indexed edits.
Prove zero processing allocations/deallocations with instrumentation and a
positive outside-processing destruction control, as in stage 9.

Service/UI acceptance additionally covers command queue overflow, stale/lost
observations, callback-chunk peak aggregation, failed/restarted sessions and
browser restrictions. Close stage 10 only after implementation checks/build,
native verification and inspected CI of its own delivered revision.
