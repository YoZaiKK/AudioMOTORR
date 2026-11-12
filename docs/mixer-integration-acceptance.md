# Integrated mixer acceptance — stage 10.3.d

The four-channel mixer is accepted locally across the document, engine, service,
application and desktop boundaries. This is separate from physical verification
and exact-revision hosted CI in 10.4. The
[local report](../progress/cierre-integracion-mezclador-10.3d.md) and its manifest
record the checked sources, commands, results and remaining limits.

## Acceptance matrix

| Requirement                                                        | Evidence                                                                                                                                                                                                           | Scope and boundary                                                                                                                                                                                                                                                                        |
| ------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Four channels, 16 routes, finite gain/pan and explicit opt-in      | [Domain tests](../crates/daw-domain/tests/mixer_contract.rs), [engine reference cases](../crates/daw-engine/tests/mixer_contract.rs), [application policy](../crates/daw-app/tests/mixer_application_contract.rs). | Invalid edits preserve selection/state. Legacy silence/sine/sampler keep their own behavior; mixed playback is explicit.                                                                                                                                                                  |
| Real WAV preparation feeds all four channels                       | [New application acceptance](../crates/daw-app/tests/mixer_acceptance_contract.rs).                                                                                                                                | Four temporary float32 WAVs, including mono and distinct stereo PCM, are decoded and then deleted before installation. Later renders depend on the retained bank.                                                                                                                         |
| Channel and master peaks match an independent reference            | Same acceptance suite, four-channel timeline.                                                                                                                                                                      | Constant PCM and dyadic coefficients yield exact rational peaks; coefficients are specified independently of engine calculations. This tests observed rendered peaks, not a public application PCM buffer API.                                                                            |
| Routing captures destinations; edits affect future triggers        | Engine routing cases, application FIFO cases and the new decoded timeline.                                                                                                                                         | Existing/paused voices retain channels while future triggers follow edited routes. Mute preserves voice occupancy and advancement.                                                                                                                                                        |
| Master gain, channel ramps and transport compose correctly         | New decoded timeline plus engine gain/pan/mute ramp tests.                                                                                                                                                         | Gain 0.5, mute, silent edits, Pause/resume, Stop, retained bank and explicit close/reopen. Engine tests check per-sample ramps; application acceptance checks independent settled/window peaks and state.                                                                                 |
| Partition behavior stays coherent                                  | Engine sample/state partition test and new application timeline at 1/127/256/4096 frames.                                                                                                                          | Identical edits occur at identical frame boundaries. Each application callback reports its own frames/sequence; combined measured peaks match the same reference at every partition size. Callback counts intentionally differ.                                                           |
| Global capacity and headroom                                       | New decoded pool case in mono/stereo, engine global-pool tests.                                                                                                                                                    | Four voices per musical channel produce full scale with 16 voices total. The seventeenth trigger is accepted into the FIFO then rejected as PoolFull. Muted voices finish and free capacity while the bank remains installed. Mono ignores retained pan and exposes no second meter side. |
| Cancellation is represented honestly                               | New decoded opposite-channel case and service cancellation test.                                                                                                                                                   | Channel peaks remain nonzero while opposing contributions cancel at the master. The manual render reports silence; this is mathematical cancellation, not physical silence measurement.                                                                                                   |
| FIFO admission, IDs, backpressure and bounded callback work        | [Application](../crates/daw-app/tests/mixer_application_contract.rs), [service](../crates/daw-audio/tests/mixer_service_contract.rs), [realtime](../crates/daw-audio/src/realtime_mixer_tests.rs).                 | Capacity 128, maximum 32 commands per callback and 64 Copy observations; invalid/QueueFull edits do not commit document changes or consume IDs. No invented confirmation after lost observations.                                                                                         |
| Complete native callback windows                                   | [Native adapter simulations](../crates/daw-audio/src/native_mixer_tests.rs).                                                                                                                                       | 1001-frame pulses, scratch sizes 1/127/256/513 and f32/f64/i16/u16 conversion. Whole-window peaks/budget are verified without opening hardware.                                                                                                                                           |
| Processing memory and owner-side teardown                          | [Instrumented mixer memory](../crates/daw-audio/tests/mixer_memory_contract.rs), native teardown tests and service lifecycle tests.                                                                                | Processing allocation/free instrumentation, positive destruction controls and returning PCM to its owner. These measurements exclude application/IPC allocation. Driver scheduling remains a separate native concern.                                                                     |
| Desktop wire format and permission boundary                        | [IPC tests](../src-tauri/tests/ipc_contract.rs), generated DTO checks and other-window rejection.                                                                                                                  | Twenty scoped commands; fixed arrays; u64 strings above 2^53 and at u64::MAX; mono null. Manual session setup opens no device and introduces no manual-render IPC command.                                                                                                                |
| Selected, accepted, applied and ramp-complete states stay distinct | [UI tests](../src/app/App.test.tsx), [mixer observation resolver](../src/features/audio/mixer-observation.ts), IPC/manual ramp case.                                                                               | BigInt ordering, event-before-reply, stale/duplicate observations, queue/timeout errors and session loss. FIFO processing acknowledgement is not a completed-ramp claim.                                                                                                                  |
| A late startup query cannot replace newer selection                | New frontend refresh/edit race regression and companion late-error regression.                                                                                                                                     | An explicit query or accepted edit invalidates the initial selection query. Its late result and error are ignored; requests still cannot be cancelled after crossing IPC.                                                                                                                 |
| Offline receipt remains truthful through initialization            | New frontend edit/initialize regression.                                                                                                                                                                           | The receipt describes a document selection without an audio command. It no longer claims the current service is uninitialized after opening a mixed session.                                                                                                                              |
| Browser, initialization and legacy behavior remain bounded         | Existing frontend/IPC/application suites and nine CLI scenarios.                                                                                                                                                   | No browser native calls, no automatic Play, refresh of consumed preparation after open failure, legacy mixer controls disabled and selection retained across shutdown.                                                                                                                    |
| Responsive controls and measured levels                            | Stage 10.3.c headless visual fixture and frontend tests.                                                                                                                                                           | Three historical simulated captures at 1200/760/390 px; layout styles are unchanged in 10.3.d. They are separate from this increment's final check/build and do not establish native mixed audio.                                                                                         |

## Independent four-channel reference

The new application timeline loads constant assets and sets master gain to 0.5
before starting the voices. All numbers below are signed contributions before
taking the absolute peak. The second asset preserves distinct stereo channels.

| Channel | Source PCM           | Gain / pan / mute | Contribution before master               |
| ------- | -------------------- | ----------------- | ---------------------------------------- |
| 1       | Mono 1               | 0.5 / −1 / off    | `(1/32, 0)`                              |
| 2       | Stereo `(1/2, −1/4)` | 1 / +1 / off      | `(0, −1/64)`                             |
| 3       | Mono −1              | 0.25 / 0 / off    | `(−1/64, −1/64)`                         |
| 4       | Mono 1/2             | 0.5 / 0 / on      | `(0, 0)` with its voice still advancing. |

The sum after master is `(1/128, −1/64)`, so the measured master peaks are
`(1/128, 1/64)`. The same independently specified timeline then changes a route,
adds voices, mutes a channel, pauses, changes settings in silence, resumes,
stops and reopens. Each settled checkpoint has its own rational oracle.

## Audit corrections and delivery boundary

Two regressions reproduced failures against 10.3.c before correction: a delayed
startup settings reply replaced a newer explicit refresh/edit, and an offline
receipt continued to describe audio as uninitialized after opening a session.
A selection-query generation guard now rejects superseded startup results and
errors; the receipt describes its operation without asserting the current service
state. A third regression covers the late-error branch. DSP, application policy,
queues, dependencies, bindings, permissions and layout styles are unchanged.

The public snapshot has no session identity. An entirely missed external
close/reopen cannot be distinguished; the desktop serializes its own lifecycle.
No multi-client or lossless command-history guarantee is introduced.

The existing CLI `sampler-native-check` calls `initialize_sampler`, and
`sampler-bench` uses the legacy sampler engine path. Their historical measurements
do not verify the new mixer. [Explicit mixed diagnostics 10.4.a](mixer-diagnostics.md)
now implement mixer-bench and mixer-native-check with their own reference workload.
[Native measurements 10.4.b](../progress/etapa-10-medicion-nativa-mezclador.md) now
record six mixed cases/reopens on Speakers Realtek under the
[native protocol](mixer-native-verification.md). [Delivery and CI](../progress/publicacion-mezclador-10.4c.md)
are verified for implementation e7f543f; listening confirmation remains pending. Loopback, unplug, prolonged stress
and other physical devices/platforms remain limits unless separately exercised.
