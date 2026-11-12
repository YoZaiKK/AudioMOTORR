# Native sampler verification protocol — stage 9.5

Prepared during the 9.4.d integration audit on 2026-11-09. **This is a protocol,
not a measurement report.** The [9.5.b results](../progress/etapa-9-medicion-nativa.md)
record six real-device cases. Subsequent repetitions and user-confirmed audible
headphone output complete [stage 9 closure](../progress/cierre-etapa-9.md) within
the recorded limits. No sampler hardware session, native timing result,
new screenshot or hosted CI result was produced by 9.4.d. The
[acceptance matrix](sampler-integration-acceptance.md) defines local integration
evidence; the [Spanish handoff](../progress/continuidad-etapa-6.md) tracks next work.

## 9.5.a — diagnostic runner implemented locally

Implemented on 2026-11-10: sampler-bench and sampler-native-check. The
[diagnostic contract](sampler-diagnostics.md) defines commands, arguments, oracle,
measurement regions and limitations; the [report](../progress/etapa-9-diagnosticos-sampler.md)
records local evidence. offline-diagnostics runs without CPAL; rt-diagnostics
also enables native audio. Existing engine-bench/tone-check remain sine-specific.

The offline benchmark uses preallocated buffers/timing storage and deterministic
distinct-channel PCM, with 0/1/16 voices, all required block sizes, natural
completion/retrigger, exact sample oracle, timing and allocation/free controls.
The native runner uses existing Application preparation/session/trigger APIs,
explicit WAV/device, frame-based workload windows and exact-ID confirmation.
Its lifecycle is tested with a real WAV and manual audio, without opening hardware.

Optional native metrics now include deallocations, frames, all-channel probes
and a fixed 64-bin histogram; quantiles are approximate upper bounds. Metrics
are cumulative across the full lifecycle, not isolated voice-window percentiles.
The native instrumented runner compiles in release but remains unexecuted with
hardware during 9.5.a; [9.5.b](../progress/etapa-9-medicion-nativa.md) now records
six Realtek/WASAPI cases with zero measured allocation/free/overrun. Offline CPU numbers do not substitute for that evidence.

## Inputs and preparation using existing commands

Create a fresh output directory; export-wav refuses to overwrite existing files.
The following are existing device-free commands, not a native sampler test:

```powershell
New-Item -ItemType Directory -Path target/stage9-5-fixtures
cargo run -p daw-cli --no-default-features --locked -- export-wav --output target/stage9-5-fixtures/mono.wav --frames 480000 --sample-rate 48000 --channels 1 --frequency 997 --amplitude 0.5
node scripts/sampler-stereo-fixture.mjs target/stage9-5-fixtures/stereo.wav
```

Both are float32 WAV, within existing per-resource limits. The stereo helper
creates distinct 997/440 Hz channels at peaks 0.1/0.05 and refuses overwrites.
9.5.b verified its byte-exact reproduction of the measured input. The diagnostic
benchmark and manual lifecycle also use independent distinct-channel oracles. Record each input SHA256, rate, channels,
frame count, occupied slots and decoded PCM budget. Repeated paths in different
slots own separate resources and count separately. Keep fixtures outside the
measured processing region and preserve existing decoder/bank ceilings.

## 9.5.b — controlled native run after runner tests pass

1. Run the full check and Windows debug build first. Build diagnostic code in
   release with rt-diagnostics, and record the exact sampler-native-check invocation/options. Record source fingerprint/Git revision, dirty status, Rust/Node/npm,
   OS, CPU, CPAL host, device ID/name, advertised config and selected format.
2. Start from closed audio; prepare once, open stopped, then issue Play and
   explicit triggers. Correlate exact command IDs with Started/rejected outcomes.
   A native runner should keep at most one unresolved trigger when requiring
   exact confirmation, or explicitly report telemetry loss as inconclusive.
   For 16-voice tests, choose long resources and verify all 16 remain active
   during the intended interval. Keep host-selected callback frame extrema.
3. Run separate 0/1/16-voice windows, gain changes, Pause/release/resume,
   Stop/retrigger, natural completion, shutdown and same-device reopen. Stop
   retains the bank; shutdown releases it and requires preparation for reopening.
   Repeat with mono/stereo source/output layouts that the actual device supports.
4. Measure command drain, whole-buffer validation, rendering, sample conversion
   and snapshot publication. Include instrumentation overhead and report excluded
   host/driver work, callback arrival jitter and other threads. Nominal duration
   is frames / negotiated sample rate; processing overruns are not acoustic dropouts.
5. Observe telemetry after a settling interval, then stop and reclaim ownership.
   Current aggregates publish one callback behind and can omit the last callback;
   an absent terminal measurement is not zero. Record exact measured callbacks/
   frames, session reset behavior and whether any result is inconclusive.

Native acceptance for the recorded workload requires callback/frame progress,
verified active voices, correct exact-ID processing/rejection and transport,
zero measured allocations/deallocations in processing, finite bounded output,
no measured nominal processing overruns, orderly close/reopen and owner-side PCM
cleanup. A short run establishes only its observed workload/host conditions.

Raw one-shot boundaries may jump: source start/end and overlapping triggers are
not covered by the sine derivative bound. Compare discontinuities against a
deterministic sampler reference and envelope policy; do not apply tone-check's
smooth-sine threshold to arbitrary WAVs or silently redefine discontinuities as
dropouts. Physical audition and loopback are distinct from generated sample probes.

## 9.5.c — report and closure

Use the implemented command documented in [sampler diagnostics](sampler-diagnostics.md);
require a resource lasting seconds+4 through 15 seconds, matching decoder/output
limits. Record file hashes before/after; the runner compares decoded metadata,
not file digests.

Store raw logs under a fresh target directory and a concise dated report under
progress, with command/options, source/input hashes, measured fields, assertions
and failures/inconclusive results. Preserve historical screenshots and reports.
Record physical audition separately; absence of loopback does not prove acoustic
continuity. Physical unplug, stress duration, other hardware formats/devices and
native Linux/macOS each need their own performed evidence or explicit limitation.

Recheck affected code, full check/build and documentation after fixes. Publishing
and hosted CI must identify the exact revision and both jobs; without publication
and inspection they remain pending. Stage 9 cannot be called fully closed based
only on this protocol, mocked adapters or previous published kernel/sine results.
