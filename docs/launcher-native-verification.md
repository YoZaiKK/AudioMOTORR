# Launcher native verification protocol

The runner is implemented and its lifecycle is exercised with manual callbacks;
physical launcher results are recorded in [13.4.b](../progress/etapa-13-medicion-nativa-clips.md),
with listening recorded separately. Earlier sampler,
mixer and sequencer measurements do not measure this launcher. Native execution
requires an explicitly chosen device and generates sound, including later control
phases when the initial clip workload is empty.

Build and prepare a fresh fixture directory:

```powershell
node scripts/sequencer-fixtures.mjs target/launcher-input --channels 2 --sample-rate 48000
cargo build -p daw-cli --release --features rt-diagnostics --locked
target/release/daw-cli.exe devices
target/release/daw-cli.exe launcher-native-check --input-dir target/launcher-input --device "EXACT_DEVICE_ID" --channels 2 --bpm 127.25 --quantization bar --clips beat --bars 1
```

The existing generator exclusively creates kick/snare/hat/accent.wav and a SHA256
manifest; it does not overwrite an existing directory. Native inputs must contain
those four files, PCM16/float32 mono/stereo, with the selected output rate and
length greater than zero through two seconds. There is no resampling or fallback
device. The command accepts rate 100..192000, output channels 1/2, clips
empty/beat/dense, BPM 20/95.5/127.25/300, quantization step/beat/bar and bars 1/2.
Actual device support determines whether an exact format opens successfully.

The runner performs the following bounded lifecycle:

1. Prepare WAV outside processing, choose Launcher mode, initial scene 7 and
   selected tempo/mix/quantization. Initialize the exact device stopped with empty
   rows, no receipt or replay. Warm up at least 66 complete callbacks before the
   initial launch. This is runner stabilization, not an exclusion from cumulative
   callback metrics.
2. Launch scene 7 explicitly, observe its exact service report and engine ID,
   then Play. Validate the rational phase, boundaries, scheduled counters and
   signal/empty meter windows over the requested bars. Scheduled notes must not
   contaminate manual trigger outcomes.
3. Pause and freeze the observed phase. Edit an active document clip without
   changing the owned active copy. Submit scene 8 (Launch/Stop/Keep/Launch),
   supersede row 0 with a separate step launch, and edit that clip afterward.
   Record the accepted targets and distinct engine IDs. Change BPM while paused;
   phase/events must remain frozen.
4. Resume and verify that rows 0/1/3 apply their captured owned targets/actions;
   row 2 keeps its previous origin/content/request. The new document edit must
   not rewrite accepted content or saved initial scene 7.
5. Launch a clip referencing empty sample 15 and observe MissingAsset. Mute all
   channels while scheduled attempts continue, wait for settled silent windows,
   then unmute and observe nonzero signal. Stop all rows explicitly; wait for
   their quantized transitions and natural voice tails before checking silence.
6. Stop transport, reset clock and pending requests, settle voices and close.
   Reprepare and reopen the same device with identical asset metadata and empty
   rows. Play without launching must remain silent, including metrics warmup;
   only an explicit launch produces notes. Service IDs must advance, engine
   request IDs restart, then Stop and close to Uninitialized.

Each phase logs exact integer clocks/counters, owned rows/pending/transitions,
service and engine IDs, selected output and input metadata. A missing exact
processing report is inconclusive and fails without replay. A watermark only
establishes processing, not a result or transition. Waits have deadlines; OS WAV
reads remain subject to the existing preparation/cancellation contract.

Metrics are cumulative **per entire session**, including the runner's silent
warmup and later control phases. There is no first-64-callback exclusion in native
instrumentation. Snapshot publication precedes recording that callback's processing
duration, so observed processing metrics can lag the snapshot callback counter by
one callback. Report histogram p50/p99 upper bounds,
maximum callback time/load, nominal deadline misses, allocation/free activity,
finite output, signal peak/jump, callback and frame sizes. The runner rejects
zero measured activity, nonfinite output, processing allocations/frees or deadline
misses and requires observed signal in each full session. It does not prove
acoustic continuity, driver latency or absence of external dropouts.

For 13.4.b, retain exact binary/source hashes, feature/profile, CPU/OS/driver and
device ID/name/format, WAV hashes, complete stdout/stderr and exit codes. Proposed
matrix: mono/stereo input sets × Step/Beat/Bar; use exact supported output formats,
include empty and dense controls plus fractional BPM, and preserve failures rather
than rerunning to select favorable timings. Record reopen results and obtain a
separate listening observation. If a format cannot open, record that limitation
explicitly. Stress, device unplugging, loopback/acoustic continuity and other
platforms remain separate observations, not inferred successes.
