# Native mixer verification protocol — stage 10.4

Prepared in 10.4.a alongside the [mixed diagnostics](mixer-diagnostics.md).
**This protocol is not a physical measurement report.** The
[local integration acceptance](mixer-integration-acceptance.md) remains separate
from native measurements and user listening. Historical sampler measurements
and its confirmed audibility do not verify this new processing path.
The subsequent [10.4.b measurement report](../progress/etapa-10-medicion-nativa-mezclador.md)
records six successful mixed cases/reopens on Speakers Realtek; listening
confirmation and delivery/CI closure are still pending.

## 10.4.a — runner and local evidence

`mixer-bench` explicitly constructs a four-channel mixed engine;
`mixer-native-check` explicitly initializes mixed native sessions and checks
gain/pan/mute, captured routes, whole-window levels and lifecycle. The
[implementation report](../progress/etapa-10-diagnosticos-mezclador.md) and manifest
record tests, release binaries and 24 offline benchmark cases. Neither the tests
nor those benchmarks open hardware.

## 10.4.b — native matrix and listening

1. Run the full check and desktop build. Build diagnostics with release and
   rt-diagnostics; retain the exact executable and its SHA256. Record source
   fingerprint, Git revision/dirty status, OS, CPU, Rust/Node/npm, CPAL host,
   explicit device ID/name, advertised format and actual selected configuration.
2. Create new fixtures without overwriting prior evidence. The existing stereo
   helper produces ten seconds of float32 WAV with distinct 997/440 Hz sides,
   peaks 0.1/0.05. A mono sine export at peak 0.1 provides the second input.

   ```powershell
   New-Item -ItemType Directory -Path target/mixer-native-fixtures
   cargo run -p daw-cli --no-default-features --locked -- export-wav --output target/mixer-native-fixtures/mono.wav --frames 480000 --sample-rate 48000 --channels 1 --frequency 997 --amplitude 0.1
   node scripts/sampler-stereo-fixture.mjs target/mixer-native-fixtures/stereo.wav
   cargo build -p daw-cli --release --features rt-diagnostics --locked
   .\target\release\daw-cli.exe devices
   ```

   Record file hashes, rate, source/output channels, frame count and decoded PCM
   budget. The native runner loads four independent resources from the same path;
   their bytes count four times against the bank budget. No resampling or device
   fallback is allowed. Stereo input with mono output remains invalid.

3. Choose a real output from enumeration and run six separate cases: mono input
   and distinct stereo input, each with initial voices 0/1/16. Start with 48000 Hz
   stereo output if advertised. Use an explicit ID and one-second initial window.

   ```powershell
   .\target\release\daw-cli.exe mixer-native-check --input target/mixer-native-fixtures/stereo.wav --device EXPLICIT_ID --sample-rate 48000 --channels 2 --voices 16 --seconds 1
   ```

   Save complete stdout/stderr and exit status per case. Correlate phase logs,
   exact trigger results, target/effective ramps, captured destinations, callback
   windows and selected device. Each case includes its own close/reopen test.
   A zero-voice initial window is followed by sound-generating control phases.

4. Inspect both sessions' cumulative metrics: callback/frame counts, callback
   frame extrema, allocation/free counts/bytes, all-channel signal extrema,
   nonfinite count, sample jumps, timing histogram, max/nominal load and deadline
   misses. Report p50/p99 as histogram upper bounds. Do not assign whole-session
   metrics to the isolated initial voice window or infer acoustic quality from
   processing counters. Failures/lost observations stay failed or inconclusive;
   retain their evidence and investigate before any repeat.
5. Audition the mixed path with the chosen physical output/headphones. For a
   deliberate pan/mute comparison, use the desktop's **Inicializar con mezcla**,
   play a loaded mono sample, hold hard-left/center/hard-right and mute/unmute
   settings long enough to compare, and test a route on a new trigger. Record
   the user's actual confirmation separately from runner exit status. No
   automatic test can infer that headphones were connected or sound was heard.

No repeated hardware run, acoustic recording or listening claim belongs to
10.4.a. Physical mono output, extra sample rates/devices/formats, loopback,
unplug/recovery, prolonged stress and Linux/macOS native runs remain explicit
limits unless separately performed. Scope any success to the measured system.

## 10.4.c — evidence, delivered revision and closure

Preserve inputs, logs, executable hashes, source fingerprint, run options and
metadata in a new native evidence manifest/report. Keep previous manifests and
sampler evidence as historical snapshots. Repeat affected checks if code changes.
Inspect both hosted CI jobs for the exact delivered Git SHA; earlier sampler CI
does not attest to local mixer changes. Record native measurements, actual
listening confirmation, current checks/build, CI and remaining limits before
marking stage 10 complete. This protocol does not perform commit/push/publication. The
[10.4.c local audit](../progress/auditoria-cierre-mezclador-10.4c.md) verifies source,
artifacts and Git/CI scope and prepares delivery; hearing confirmation and a
delivered mixer revision with its own CI remain pending.
