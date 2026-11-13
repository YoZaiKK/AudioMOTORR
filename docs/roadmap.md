# Roadmap after the silence foundation

Updated 2026-11-12. Stages 1–6 are the implemented silence foundation, published
in baseline `1ef9bd0`. Stage 7 is complete: engine oscillator, bounded application
rendering and WAV export are published in c6cf627 and passed hosted Linux/Windows
checks. See [stage 7 closure](../progress/cierre-etapa-7.md) for the exact evidence.
Stage 8 is complete: native audition, ramps, and bounded release measurements are
published in 1e6bca8 and passed both hosted jobs in
[run 42524752652](https://github.com/YoZaiKK/AudioMOTORR/actions/runs/42524752652).
The [stage report](../progress/etapa-8.md) records hardware evidence and limits.
Stage 9 is complete within its recorded scope; [closure](../progress/cierre-etapa-9.md)
includes exact-revision CI and user-confirmed audible headphone output. Stage
10.1 is [specified](mixer-contract.md) and the [10.2 core](../progress/etapa-10-mezclador-motor.md)
is implemented locally. [Service integration 10.3.a](mixer-service.md) now adds
mixed sessions, FIFO controls and full-callback meters.
[Application coordination 10.3.b](mixer-application.md) now installs current
document settings and distinguishes selection/acceptance/application.
[Desktop exposure 10.3.c](mixer-desktop.md) now provides four strips, routes
and observed levels. [Integrated acceptance 10.3.d](mixer-integration-acceptance.md)
is complete locally. [Mixed diagnostics 10.4.a](mixer-diagnostics.md) and the
[native protocol](mixer-native-verification.md) are implemented. Physical
mixed measurements 10.4.b are [recorded](../progress/etapa-10-medicion-nativa-mezclador.md),
with [local delivery audit](../progress/auditoria-cierre-mezclador-10.4c.md) complete.
[Implementation delivery and exact-revision CI](../progress/publicacion-mezclador-10.4c.md)
are verified for e7f543f. The user confirmed audible mixed output after a replay;
[stage 10 is closed](../progress/cierre-etapa-10.md) within the recorded limits.
[11.1 is implemented locally](../progress/etapa-11-modelo-patron.md): fixed
four-row, 16-step domain pattern and [sequencer contract](sequencer-contract.md).
[11.2 is implemented locally](../progress/etapa-11-planificador.md): exact-phase
scheduling and opt-in sequenced mixed playback, with independent references.
[11.3.a is implemented locally](sequencer-service.md): explicit sessions,
bounded pattern commands and applied phase/outcomes across complete callbacks.
11.1–11.2 are committed in `1effd48`; service 11.3.a is delivered in `2d4cc28`.
[11.3.b is implemented locally](sequencer-application.md): selected pattern,
queue acceptance, applied state and current settings at bank installation.
[11.3.c is implemented locally](sequencer-desktop.md): generated DTOs/IPC/ACL,
four-row grid, slots, receipts and observed position; real WebView2 editing/layout verified.
[11.3.d is accepted locally](sequencer-integration-acceptance.md): decoded beat
reference, lifecycle/UI/IPC coverage audit and current check/release/build.
[11.4.a is implemented locally](sequencer-diagnostics.md): 72 offline measurements,
explicit native runner, deterministic fixtures and a physical verification protocol.
[11.4.b physical processing is measured](../progress/etapa-11-medicion-nativa-secuenciador.md);
listening and exact delivery/CI closure remain pending.
[Stage 12 is implemented locally](project-format.md): portable JSON projects,
explicit saved-asset preparation, exact beat/tail render and desktop/CLI WAV export.
[Stage 12 is delivered and closed within scope](../progress/cierre-etapa-12.md)
in `6c8c52e`, with both hosted jobs verified. Stage 11 listening remains pending;
clips/scenes remain stage 13.
Historical delivery detail: 9.1–9.2 are complete and published; 9.2 implementation
7acc1db passed both hosted jobs in
[run 42530699397](https://github.com/YoZaiKK/AudioMOTORR/actions/runs/42530699397).
This delivers immutable resources, the fixed bank/raw voice kernel, and memory-lifetime tests.
Native sessions and desktop integration are implemented through 9.4.c, with
physical processing measurements and user-confirmed audibility recorded at closure.
Sequencer listening closure and stage 13 remain open; delivery/CI of 11.4.c and
stage 12 are recorded in the closure/publication reports.
See the [current kernel report](../progress/etapa-9-voces.md) and
[resource report](../progress/etapa-9-sampler.md).
9.3.a now specifies the [loading contract](sample-loading-contract.md), including
encoded-file/working-data caps and cancellation. 9.3.b now implements bounded
single-resource preflight/decoding and memory tests. 9.3.c implements complete-bank
coordination, permits and aggregate accounting, with maximum-bank coexistence
tests. 9.3.d verifies injected failures, synchronized cancellation and cleanup
ownership across threads. 9.3.e verifies end-to-end WAV/bank/offline-sampler
fixtures and consolidates the [acceptance matrix](sample-loading-acceptance.md).
9.3.f closes preparation locally; 9.4.a integrates the sampler into AudioEngine
with headroom/transport/ramps and direct Copy trigger results, verified offline.
9.4.b now connects manual/CPAL sessions, shared-FIFO triggers and lossy Copy
outcome observations, with owner-side PCM reclamation after closing the stream.
9.4.c adds application/DTO/IPC/frontend with mock verification. 9.4.d now closes
integration locally with an [acceptance audit](sampler-integration-acceptance.md)
and UI recovery/polling fixes. 9.5.a implements [sampler diagnostics](sampler-diagnostics.md). 9.5.b now records
[six Realtek/WASAPI runs](../progress/etapa-9-medicion-nativa.md), with listening
confirmation subsequently recorded at closure. The [9.5.c audit](../progress/auditoria-cierre-9.5c.md)
now verifies source/artifact integrity and both successful jobs of implementation
`4dc2c6c` in [run 42896571891](https://github.com/YoZaiKK/AudioMOTORR/actions/runs/42896571891).
The [closure report](../progress/cierre-etapa-9.md) records the final listening
observation under the [native protocol](sampler-native-verification.md). The
[engine report](../progress/etapa-9-integracion-motor.md) records new local evidence.
The [session report](../progress/etapa-9-sesiones-sampler.md) and
[service contract](sampler-service.md) describe the newer integration and limits.
See the [closure audit](../progress/cierre-preparacion-9.3.md) and the
[reader report](../progress/etapa-9-lectura-wav.md) and
[bank report](../progress/etapa-9-preparacion-banco.md) and
[fault/cleanup report](../progress/etapa-9-fallos-cancelacion.md).
The [loaded-sampler report](../progress/etapa-9-fixtures-integracion.md) records 9.3.e.
Stages 7–13 preserve the order of the original seven follow-up milestones.
The [Spanish execution plan](../progress/plan-etapas-7-13.md) and
[progress checklist](../progress/checklist.md) track their status.
The [status/objectives audit](../progress/estado-y-siguiente-etapa.md) compares
these milestones with implemented code and details the next increment.

## Product objectives and release boundaries

The foundation is delivered (1–6), signal generation/audition is delivered (7–8),
and sampler resources/raw voices are delivered (9.1–9.2). Loading (9.3) and
engine/session/desktop integration (9.4.a–c) are implemented locally. Integration
audit (9.4.d) and verification/closure (9.5) are complete within documented limits. The four-channel,
16-step groovebox still requires stages 10–11; saving/reopening/exporting the beat requires 12. Clips/scenes in 13 extend that product. The current sine WAV command is not
project/beat export. These are scope boundaries, not delivery dates or claims
of completed UI features.
The 16 playback voices are independent of the four musical mixer channels and
16 sequencer steps; mono/stereo describes PCM input/output channel layout.

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

## Stage 9 delivery steps

The [sampler contract](sampler-contract.md) distinguishes implemented resource
APIs from the fixed playback policy. This stage does not introduce a generic
instrument graph or implement the later four-channel mixer.

| Step | Responsibility                                                                                    | Exit criterion / status                                                                                                                                                                                                    |
| ---- | ------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 9.1  | Engine PCM resource representation, ownership/limits, fixed voice policy, deterministic fixtures. | Resource validation and four tests implemented; contract defines 16 immutable bank slots, 16 voices, reject-newest overflow, transport/headroom and future raw output oracles. Resource-only increment.                    |
| 9.2  | Engine bank and one-shot voice kernel.                                                            | Implemented: exact mono/stereo, overlap, exhaustion/reuse, retained PCM, error rollback and partitions 1/127/256/4096; debug/release memory tests report zero allocations/deallocations.                                   |
| 9.3  | Application resource preparation and bounded asset decoding outside processing.                   | Closed locally: validated owning bank and handoff to a caller-owned holder, bounded payload/buffers, cleanup and offline playback evidence. Service session installation remains 9.4; hosted CI for this worktree pending. |
| 9.4  | Integrated sampler transport/headroom, audio service outcomes, explicit desktop controls.         | Preserve silence/sine regressions; distinguish enqueued/processed/started or rejected triggers; mock IPC/frontend tests and real native audition.                                                                          |
| 9.5  | Verification, evidence, documentation and closure.                                                | Full checks/build, instrumented native sampler timing/allocation/deallocation/continuity measurements and exact published-revision CI; record physical/platform limitations.                                               |

## Stage 9.3 — closed locally; engine integration 9.4.a implemented

Delivers a validated immutable SampleBank prepared from WAV files outside
processing. Application policy and an I/O adapter own preparation/decoding;
the engine receives only validated PCM. PreparedBank transfers ownership to a
caller-owned bank holder; service session installation and desktop playback
arrive in 9.4. Closure is local, with hosted CI pending publication/inspection.

| Step  | Work                                                        | Acceptance                                                                                                                                                                              |
| ----- | ----------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 9.3.a | Input/format and memory contract specified 2026-11-09.      | PCM16/float32 classic/extensible WAV; per-input limits implemented in 9.3.b and aggregate budgets/permits implemented in 9.3.c.                                                         |
| 9.3.b | Single-resource reader/decoder implemented locally.         | Bounded standard-library preflight, PCM16/float32 classic/extensible, exact reservations, cancellation, ownership and debug/release memory/fixture coverage.                            |
| 9.3.c | Complete-bank coordinator implemented locally.              | Validated selections, ordered loading, shared permit through pending-result lifetime, checked aggregate budgets and debug/release maximum-bank memory tests.                            |
| 9.3.d | Fault/cancellation/cleanup verified locally.                | 17 second-asset fault/cancellation cases, synchronized publication/blocked reads, reader ownership traces and worker/receiving-thread allocator checks pass debug/release.              |
| 9.3.e | End-to-end fixtures and acceptance matrix verified locally. | Four tests pass debug/release: 18 format/header/channel combinations × four partitions, all PCM16 codes, empty/full banks and real-file exact/excess limits.                            |
| 9.3.f | Local preparation closure completed.                        | Acceptance audited against code/tests, 9.3.e checks/build reused, 110 source/configuration files fingerprinted; publication/platform limits explicit. Stage 9 remains open for 9.4–9.5. |

Existing limits remain 1048576 frames per resource (4 MiB mono / 8 MiB stereo),
16 slots and at most 128 MiB retained PCM per bank. The specified 9.3.a limits are
16 MiB per encoded file, 4096 chunks, 64 KiB combined decoder buffers, and one
extra 8 MiB PCM allocation for finalization. Preparation data is capped at
136 MiB + 64 KiB, or 264 MiB + 64 KiB including one old active bank. These are
managed payload/buffer budgets, not process RSS. Per-file/frame/buffer limits
and exact PCM reservation are implemented/tested in 9.3.b; aggregate accounting
and the coordinator permit are implemented/tested in 9.3.c. 9.3.d verifies
injected faults, cancellation/publication and cleanup thread ownership.
9.3.e verifies end-to-end loaded-bank playback fixtures and maps format/limit
coverage. 9.3.f reviewed criteria, dated checks and outstanding platform/publication
limits and closed preparation locally; native/service/UI integration remains 9.4.
One coordinator admits a single preparation/result; Busy rejects another until
the result is installed/discarded. Cancellation checks occur at most every 256
decoded frames and before publication; a completed bank is not retroactively revoked.
PCM16/float32 classic/extensible WAV is the fixed initial subset; PCM24/32,
compressed formats and resampling require a later explicit contract revision.
See [format/error/cleanup details](sample-loading-contract.md). The 9.3.b adapter
uses standard-library Rust without a new decoder dependency or engine file I/O.

## Stage 9.4–9.5 — integration and sampler closure complete within recorded limits

9.4.a–d are implemented/audited locally with offline/adapter/mock evidence.
Physical sampler acceptance and measurements remain pending in 9.5. The existing
[sampler transport/outcome contract](sampler-contract.md) defines behavior;
engine implements transport/headroom, and the service implements sampler sessions
and queued exact-ID outcomes. Application/DTO/frontend exposure is implemented
in [9.4.c](sampler-desktop.md). [9.5.b](../progress/etapa-9-medicion-nativa.md)
records short Realtek/WASAPI hardware measurements; subsequent audible output
confirmation is recorded in [stage 9 closure](../progress/cierre-etapa-9.md).

| Step  | Responsibility                                                                                                                               | Acceptance                                                                                                                                                                                                                                        |
| ----- | -------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 9.4.a | Implemented offline: owning AudioEngine sampler mode, headroom 1/16, 5 ms envelope, transport and direct typed Copy trigger result.          | Seven integration tests, internal overflow, WAV-to-engine fixture and zero-allocation/deallocation processing test pass debug/release. Audio sessions/queued telemetry are supplied by 9.4.b.                                                     |
| 9.4.b | Implemented: manual/CPAL sessions, shared-FIFO triggers, exact-ID Copy observations and owner-side processor reclamation after stream close. | Session/queue limits, telemetry loss/saturation, whole-callback chunk equivalence, failure/restart/expired requests and simulated teardown pass debug/release. Hardware/native timing unverified; UI/DTOs are supplied by 9.4.c.                  |
| 9.4.c | Implemented: application use cases, worker preparation/cancellation and explicit desktop controls with generated DTOs and IPC/ACL.           | Loading/cancellation/initialization/trigger flows and pending/final outcomes verified with mock runtime/frontend tests. Paths/decoders/PCM remain outside realtime queues; editable project persistence stays in stage 12.                        |
| 9.4.d | Completed locally: acceptance audit, device-list recovery and bounded polling fixes, regression/evidence and native protocol.                | Headless/full checks, Windows build, documented real audition where performed, and explicit platform/hardware limits. Publication/CI belongs to its exact revision; native sampler performance measurements and final stage 9 closure remain 9.5. |

Measure native sampler behavior and close stage 9 in 9.5. The
[native protocol](sampler-native-verification.md) defines the following sequence:

| Step  | Responsibility                                                                                            | Acceptance                                                                                                                                                                                                                                                          |
| ----- | --------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 9.5.a | Implemented locally: sampler benchmark and opt-in native diagnostic runner, offline feature without CPAL. | Existing application/service APIs; bounded preallocated measurements for 0/1/16 voices; allocation/deallocation accounting and offline/error tests. Offline oracle/manual lifecycle, all-channel probes and histogram tests pass; physical execution remains 9.5.b. |
| 9.5.b | Six native matrix cases measured; audible output subsequently confirmed by the user.                      | Exact-ID outcomes, voices, transport/gain, processing budget, memory, close/reopen and separately recorded listening scope.                                                                                                                                         |
| 9.5.c | Sampler closure complete within documented limits.                                                        | Both jobs successful for 4dc2c6c; measured hashes verified, audible output confirmed and historical evidence preserved.                                                                                                                                             |

The detailed
[Spanish audit and next-stage plan](../progress/estado-y-siguiente-etapa.md)
records completed evidence, remaining objectives and acceptance.

Recording, external MIDI, time stretching, VST/CLAP hosting, complex routing,
latency compensation, and collaboration remain later work. These stages do not
include placeholders or dependencies for those features.
