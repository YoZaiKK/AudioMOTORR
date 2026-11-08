# Roadmap after the silence foundation

Updated 2026-11-08. Stages 1–6 are the implemented silence foundation, published
in baseline `1ef9bd0`. Stage 7 is complete: engine oscillator, bounded application
rendering and WAV export are published in c6cf627 and passed hosted Linux/Windows
checks. See [stage 7 closure](../progress/cierre-etapa-7.md) for the exact evidence.
Stage 8 native audition and ramps are implemented locally, with release hardware
measurements in the [stage report](../progress/etapa-8.md). Stages 9–13 remain planned.
Stages 7–13 preserve the order of the original seven follow-up milestones.
The [Spanish execution plan](../progress/plan-etapas-7-13.md) and
[progress checklist](../progress/checklist.md) track their status.

Each stage depends on closing the previous one with behavior tests, documentation,
and recorded evidence. Add concrete types and dependencies only when needed.
The [implemented realtime contract](realtime-contract.md) remains authoritative;
the [offline contract](offline-oscillator.md) distinguishes implemented signal
behavior and offline export. The [increment report](../progress/etapa-7-oscilador.md)
records baseline hosted success and local oscillator validation separately;
the [export report](../progress/etapa-7-exportacion.md) covers the current increment.

| Stage | Deliverable and responsibility                                                                                                | Acceptance                                                                                                                                                                                                                                                                                         |
| ----- | ----------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 7     | Offline low-amplitude sine oscillator in daw-engine; daw-app coordinates bounded rendering; CLI writes WAV outside rendering. | Phase-continuous samples match reference values and block partitions; transport/errors preserve their defined phase semantics. Default silence remains unchanged. Reopen mono/stereo float32 WAV with exact frames/metadata, test errors, and pass headless/full checks. See detailed steps below. |
| 8     | Same oscillator through CPAL, engine gain ramps, and explicit application/desktop controls.                                   | Offline/native consistency and ramp behavior are tested. Measure callback deadlines, allocations, continuity, and load in release on identified hardware; retain lifecycle/error coverage and report unverified platforms.                                                                         |
| 9     | Sampler and fixed voice pool in engine; application prepares asset loading/decoding outside the callback.                     | Known fixtures render at scheduled positions. Voice exhaustion has an explicit policy, resource lifetime is safe, and callback playback performs no loading, decoding, allocation, or blocking.                                                                                                    |
| 10    | Four-channel mixer: document state, engine gain/pan/mute and meters, bounded audio telemetry, application/UI controls.        | Deterministic fixtures verify gain, pan law, mute, summing, output-level policy, and meter accuracy without callback IPC.                                                                                                                                                                          |
| 11    | 16-step sequencer: domain patterns, engine scheduling with within-block offsets, application use cases and UI.                | Identical event positions across partitions, fractional frames per step and BPM changes; explicit rounding/drift policy and no duplicate/skipped boundary events on transport changes.                                                                                                             |
| 12    | Versioned document, save/open, relative assets, beat export; application migrations and adapter file I/O.                     | Document round-trip, explicit version/asset/path errors, exact export duration and agreement with offline rendering. No persistence or filesystem access in callback.                                                                                                                              |
| 13    | Clips/scenes in domain, transition scheduling in engine, application/UI edit and launch use cases.                            | Define and test launch quantization and transitions across blocks/tempos; preserve bounded commands and separate editable/observed state. Timeline, piano roll, and automation follow later.                                                                                                       |

## Stage 7 delivery steps

7.0 is complete with both hosted jobs successful for `1ef9bd0`; 7.1 fixed the
engine API and export policies; hound 3.5.1 was selected in 7.3. Signal/export are
implemented with tests discovered by existing checks. 7.4/7.5 are closed after
run 42419963899 succeeded for c6cf627.

| Step | Work                                                                                     | Exit criterion                                                                                                                                                 |
| ---- | ---------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 7.0  | Inspect both hosted CI jobs for baseline 1ef9bd0 and resolve blockers.                   | Record run URL, SHA, result, or a concrete verification limitation. Do not infer remote success from local tests.                                              |
| 7.1  | Fix signal, phase, transport, validation, ownership, and WAV decisions.                  | Engine API and export policies fixed; hound 3.5.1 is pinned in CLI. No UI/IPC changes or generic instrument layer.                                             |
| 7.2  | Implement/test the opt-in oscillator inside daw-engine using existing buffers/transport. | Reference signal, parameters, phase, errors, mono/stereo, and partitions 1/127/256/4096 pass; default silence regressions remain valid.                        |
| 7.3  | Add application offline coordination and CLI WAV output.                                 | Reuse bounded buffers, keep live service/document independent, verify samples/metadata by reopening files, and expose creation/write/finalization/size errors. |
| 7.4  | Integrate tests/fixtures/checks/CI and update runnable documentation.                    | Keep nine silence scenarios, add offline signal/export coverage, run headless/full checks and Windows debug build, and record hosted results separately.       |
| 7.5  | Record evidence and close stage 7.                                                       | All deliverables/criteria are met, limitations are explicit, and the checklist reflects actual implementation.                                                 |

## Stage 8 delivery and evidence

| Step | Responsibility                                                                       | Evidence / acceptance                                                                                                                                                  |
| ---- | ------------------------------------------------------------------------------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 8.1  | Engine: validated gain, 5 ms per-frame envelope, requested/actual transport.         | Reference/partition/error/retarget tests; immediate offline constructor preserved.                                                                                     |
| 8.2  | Audio/application: explicit manual/native sine initialization and queued gain.       | Same oscillator; fixed 0.1 voice peak; lifecycle/queue regression tests and native chunk equivalence.                                                                  |
| 8.3  | Desktop: eleven IPC commands, regenerated DTOs, explicit source/frequency/volume UI. | Mock runtime and frontend behavior tests; browser operations disabled and initialization requires a click.                                                             |
| 8.4  | Optional release diagnostics and real Windows verification.                          | Thread-local allocation counting, processing/buffer-duration ratio, sample continuity probes, hardware identity, and explicit limits. See report for commands/results. |
| 8.5  | Local checks/build, documentation and publication evidence.                          | Record local results separately from any hosted CI for the new revision; preserve dated screenshots.                                                                   |

The [native contract](native-audition.md) specifies ramp/acknowledgement semantics
and diagnostic scope. Physical loopback, unplug, long stress runs, native Linux,
and macOS remain separate unverified cases.

## Next concrete increment: stage 9.1

Define the sampler contract and deterministic in-memory fixtures before adding
file loading or UI. Fix supported sample representation/rate/channels, immutable
resource ownership, maximum voice count, trigger/stop behavior, and an explicit
voice-exhaustion policy. Then implement bounded voice playback in daw-engine and
test exact samples, overlapping voices, lifetime, and partition equivalence.
Prepare/decode resources outside the callback through daw-app in a later stage 9
increment. Do not build the mixer or sequencer before closing sampler acceptance.

Recording, external MIDI, time stretching, VST/CLAP hosting, complex routing,
latency compensation, and collaboration remain later work. These stages do not
include placeholders or dependencies for those features.
