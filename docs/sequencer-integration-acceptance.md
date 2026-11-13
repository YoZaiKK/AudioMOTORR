# Sequencer integration acceptance — stage 11.3.d

Local acceptance is complete on 2026-11-12. The production sources match the
stage 11.3.c capture; this increment adds one Rust integration-test file.
The [dated report](../progress/cierre-integracion-secuenciador-11.3d.md) and
[manifest](../progress/evidencia-integracion-secuenciador-11.3d.json) record the
source identity, checks and limits. Native measurements, listening and delivery
CI remain stage 11.4.

## Independent decoded beat

[`sequencer_acceptance_contract.rs`](../crates/daw-app/tests/sequencer_acceptance_contract.rs)
adds two tests with 42 matrix combinations, rather than 42 distinct tests:

- Decoded engine PCM: mono/stereo × 95.5/127.25/300 BPM × blocks 1/127/256/4096
  = 24 combinations.
- Prepared application/service session: the same outputs and tempos × blocks
  127/256/4096 = 18 combinations. Each callback is compared with the decoded
  engine and independent reference.

Four 48 kHz WAVs combine PCM16 and float32, mono sources and distinct stereo
sides. Source values are exactly representable in both encodings; lengths are
2048/8000/1024/6000 frames. Both the synchronous bank coordinator and asynchronous
application preparation decode the files. After both banks are ready, all source
files are deleted before installation/rendering. Pattern/mixer edits after
preparation must take effect at installation; the selected fractional tempo
survives closing an earlier session before its queued command was applied.

The four-row pattern has 18 occurrences per bar: kick at 0/4/8/12, snare at 4/12,
hi-hat at even steps and accent at 3/7/11/15. It runs for one bar plus 257 frames,
including the following step 0: 17 boundaries and 20 started occurrences.
At 300 BPM source tails overlap. Slot routes are 2/0/3/1; channel gains are
0.5/0.75/0.25/1, pans −1/0.5/1/0 and channel 3 is muted. Muted notes still
occupy, advance and release voices. Master gain is 0.5 with a 240-frame attack.

The reference calculates absolute boundary `ceil(k × 48000 × 60 / numerator)`
for BPM `numerator / 4`; it does not call the scheduler or domain event iterator.
It owns an ordered 16-voice pool, evaluates the fixture waveform by source age,
sums in voice/channel order and independently applies gain, pan, 1/16 headroom
and the master envelope. It compares every engine PCM sample and each channel
and master peak over the entire render window with tolerance `1e-7`.

Exact checks cover musical phase, loop/step, last boundary/event, frame position,
row/slot/voice outcome, counters and captured voice routes. The application adds
engine/mixer snapshot equality, active voice count, FIFO watermark and silence
report. Scheduled occurrences must leave manual-trigger counters and reports
untouched. Shutdown clears observations and preserves selected pattern/mixer.

`Application::render_manual` returns a report, not PCM. Per-sample assertions run
on the bank decoded through the real preparation coordinator in `AudioEngine`;
the independently prepared application bank is checked through its real service
snapshots/meters. No new production test API is exposed.

## Acceptance coverage

The current check reruns these existing suites alongside the new fixture.
They are complementary layers, not a single browser-to-audio automation.

| Requirement                                                                  | Executed evidence                                                                                                                                     | Limit                                                                  |
| ---------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------- |
| Full decoded beat, partitions, tempo, mixer and event identity               | New two-test matrix above                                                                                                                             | Device-free manual backend                                             |
| Boundary edits, Pause/Play, Stop, queue admission and rollback               | [Application tests](../crates/daw-app/tests/sequencer_application_contract.rs), [engine reference](../crates/daw-engine/tests/sequencer_reference.rs) | Deterministic fixtures                                                 |
| Stale/cancelled/failed preparation, missing slots/pool full, shutdown/reopen | Application tests, [service contracts](../crates/daw-audio/tests/sequencer_service_contract.rs)                                                       | Physical driver failure/unplug not exercised                           |
| Snapshot loss, command budget, scratch chunks/conversions, PCM ownership     | [Service tests and adapter contracts](sequencer-service.md)                                                                                           | Native adapter exercised without a physical stream                     |
| Selected/applied state, unknown receipts, early/late replies, exact IDs      | [UI tests](../src/app/App.test.tsx), [desktop contract](sequencer-desktop.md)                                                                         | Mock client/events                                                     |
| Fixed DTO shape, atomic commands, u64/u128 strings and main-window ACL       | [IPC contract](../src-tauri/tests/sequencer_ipc_contract.rs)                                                                                          | Mock Tauri runtime with real application/service state                 |
| Processing allocation/free and teardown                                      | [Sequencer memory contract](../crates/daw-audio/tests/sequencer_memory_contract.rs), instrumented audio tests                                         | No allocation claim about loader, IPC or React                         |
| Actual grid editing/layout                                                   | [11.3.c WebView2 capture](screenshots/stage11-sequencer.json), unchanged production source hashes                                                     | Historical visual capture, audio uninitialized; not repeated in 11.3.d |

The full local check also covers the earlier sampler/mixer/transport contracts,
54 frontend cases, seven IPC cases, nine manual CLI scenarios, generated bindings,
formatting, lint, web build and Rust Clippy. The two new cases and nine application
cases pass in optimized release as well. A fresh Windows desktop debug build
without an installer completes the acceptance. Test executions overlap between
feature sets; they are not added together as distinct cases.

## Following diagnostics and native verification

[11.4.a now implements explicit sequenced diagnostics](sequencer-diagnostics.md)
and a native verification protocol using the existing engine/application path,
with 72 offline measurements and manual lifecycle tests. Existing `sampler-bench`, `mixer-bench`
and native checks initialize other source modes; their measurements do not
measure the sequencer. Next, run identified physical devices and obtain listening
observations in 11.4.b. Stage 11.4.c audits
the delivered revision and its hosted Linux/Windows CI before stage 11 closes.
Project persistence and beat export remain stage 12.
