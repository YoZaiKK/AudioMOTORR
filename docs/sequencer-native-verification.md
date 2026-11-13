# Native sequencer verification protocol — stage 11.4

Prepared in 11.4.a with [explicit diagnostics](sequencer-diagnostics.md).
**No physical sequencer measurement or listening is recorded by this protocol.**
The local [integration acceptance](sequencer-integration-acceptance.md) and
[diagnostic report](../progress/etapa-11-diagnosticos-secuenciador.md) remain
separate from the 11.4.b measurements and 11.4.c delivery audit.

## 11.4.b — physical matrix and listening

1. Retain the current check/build results, Git revision/dirty status and source
   fingerprint. Build release diagnostics and save the exact executable and hash.
   Record OS/CPU/toolchain, CPAL host, explicit output ID/name, advertised formats
   and actual selected configuration. Previous sampler/mixer measurements and
   their listening confirmations do not validate the sequencer.

   ```powershell
   cargo build -p daw-cli --release --features rt-diagnostics --locked
   .\target\release\daw-cli.exe devices
   ```

2. Generate new fixture directories, preserving earlier files/manifests. At
   48 kHz each bank contains kick/snare/hat/accent at 5760/8640/2880/11520 frames.
   WAVs are float32 and their finite source peaks are below 0.121. The runner
   starts with channel gains 0.25 and existing 1/16 headroom. Record all hashes,
   source/output channels, decoded metadata and retained PCM budgets. No sample
   rate conversion or device fallback is permitted; stereo input cannot target
   mono output.

   ```powershell
   node scripts/sequencer-fixtures.mjs target/sequencer-native-mono-NEW --channels 1
   node scripts/sequencer-fixtures.mjs target/sequencer-native-stereo-NEW --channels 2
   ```

3. On a real output advertising 48 kHz stereo, run six separate cases:
   mono-source and distinct-stereo banks × initial patterns empty/beat/dense.
   Use 127.25 BPM and one bar initially, with the exact enumerated device ID.

   ```powershell
   .\target\release\daw-cli.exe sequencer-native-check --input-dir target/sequencer-native-stereo-NEW --device EXPLICIT_ID --sample-rate 48000 --channels 2 --pattern beat --bpm 127.25 --bars 1
   ```

   Save stdout/stderr, exit status, command and timestamps per case. Each case
   checks its own reference timeline, missing-asset phase, pause/edit/resume,
   mute/unmute, Stop and close/reopen. Even an empty initial pattern is followed
   by sound-generating control/reopen phases. Additional 95.5/300 BPM or physical
   mono outputs are separate runs, not implied by the six-case matrix.

4. Inspect observed phase, last boundary/event frames and counters, exact command
   IDs/watermarks, frozen paused state, whole-window peaks, selected routing and
   fresh reopened state. The native runner observes sampled snapshots, not an
   exhaustive event history; cumulative counters and rational checks bridge
   intermediate snapshots. Lost required observations are failed/inconclusive,
   never an inferred pass. Initial callbacks may already have advanced stopped
   device frames before the initializer returns.
5. Inspect both sessions' cumulative callback/frame counts, frame extrema,
   allocations/frees/bytes, finite signal extrema, max jumps, timing histogram,
   load ratio and deadline misses. p50/p99 are histogram upper bounds. Aggregate
   timing/memory publication lags a callback; sample probes include the current
   callback. The runner waits two later callbacks after Stop. Whole-session
   metrics include control phases and are not pattern-specific percentiles.
   Preserve failures and investigate before any repeat.
6. Obtain an actual listening observation on the selected physical output or
   headphones. Record who listened, connection/output, date, fixture/pattern/BPM,
   whether the repeating beat and Pause/Stop/mute controls were heard, and any
   cuts/errors. If using the desktop, load the same four assets, select
   **Inicializar secuenciador**, enter the reference cells/slots and explicitly
   Play. Keep settings long enough for the listener to compare. A runner exit
   code, sample probe or earlier “Se escucha” does not establish current hearing
   or acoustic continuity.

The native short samples may not exhaust the voice pool; explicit pool-full
behavior is checked by the offline long-tail dense fixture. There is no acoustic
loopback assertion. Physical unplug/recovery, long stress runs, extra devices,
rates/formats, physical mono and Linux/macOS native audio stay outside measured
scope unless separately exercised. Scope any success to the recorded system.

## 11.4.c — evidence, delivery and closure

Keep input manifests, full per-case logs, executable/source hashes and both
sessions' metrics. Separate processing results, listening and unmeasured acoustic
properties. Reconcile acceptance criteria, tests/build and current source identity;
do not attribute historical CI to later changes. After delivery, inspect both
hosted Linux/Windows jobs for the exact revision. Close stage 11 only when its
recorded criteria are met, with remaining verification limits explicit.
Persistence and full beat export remain stage 12; clips/scenes remain stage 13.
