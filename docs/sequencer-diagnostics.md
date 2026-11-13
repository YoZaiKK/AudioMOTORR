# Sequencer diagnostics — stage 11.4.a

`sequencer-bench` and `sequencer-native-check` explicitly construct sequenced
sessions. Sampler/mixer diagnostic commands keep their existing source modes.
The [local report](../progress/etapa-11-diagnosticos-secuenciador.md) records
checks and offline measurements. The [native protocol](sequencer-native-verification.md)
defines the subsequent physical verification; it is not a measurement report.

## Commands and gates

```powershell
cargo run -p daw-cli --release --features offline-diagnostics --locked -- sequencer-bench --bars 2 --channels 2
cargo run -p daw-cli --release --features offline-diagnostics --locked -- sequencer-bench --pattern dense --bpm 300 --block-size 127 --bars 1
cargo run -p daw-cli --release --features rt-diagnostics --locked -- sequencer-native-check --input-dir DIR --device EXPLICIT_ID --pattern beat --bpm 127.25 --bars 1
```

Both commands require release. `offline-diagnostics` enables instrumentation
without CPAL; `rt-diagnostics` additionally enables native audio. Native options
are validated before directory reads or device access. Without native support,
valid native arguments return `NativeAudioDisabled` before preparation. No
default-device fallback or automatic stream opening is introduced.

| Command                  | Options                                                                                  | Defaults and limits                                                                                                                                        |
| ------------------------ | ---------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `sequencer-bench`        | `--block-size`, `--pattern`, `--bpm`, `--bars`, `--channels`                             | Blocks 1/127/256/4096 × patterns empty/beat/dense × BPM 95.5/127.25/300, two bars, 48 kHz stereo. Explicit block 1–4096, bars 1–8, output mono/stereo.     |
| `sequencer-native-check` | `--input-dir`, `--device`, `--sample-rate`, `--channels`, `--pattern`, `--bpm`, `--bars` | Required directory/device; 48 kHz stereo, beat, 127.25 BPM, one bar. Rate 100–192000, mono/stereo, bars 1–2. Four named WAVs, each >0 through two seconds. |

Both commands accept the exact BPM presets 20/95.5/127.25/300. This bounded set
permits independent rational clock checks; it does not restrict the general
application tempo API. Patterns are `empty`, `beat` and `dense`: all off;
kick at 0/4/8/12, snare at 4/12, hat at even steps, accent at 3/7/11/15;
or all 64 cells on. Rows initially use slots 0/1/2/3.

## Offline reference and measurement

The benchmark prepares four immutable synthetic assets lasting
12000/18000/6000/24000 frames. For slot `s`, age `a` and output side `c`, the
sample is `((7*a + 3*s + 11*c) % 17) / 32 - 0.25`. Routes are 2/0/3/1;
channel gains 0.5/0.75/0.25/1, pans −1/0.5/1/0, channel 3 muted.
Master attacks to unity over 240 frames. Mono ignores pan. Long dense tails
at 300 BPM deliberately exhaust the global 16-voice pool.

An independent oracle computes each absolute boundary
`ceil(k * 48000 * 60 / numerator)` for BPM `numerator / 4`, maintains its own
ordered voice pool and rejects newest occurrences when full. It checks every
PCM sample, entire-window channel/master peaks (tolerance `1e-7`), exact phase,
boundary/event identity/frame/outcome, cumulative counters and captured routes.
It does not call the scheduler or DSP mixer/panner. Missing assets are covered
by the native lifecycle fixture; all four benchmark slots are occupied.

After 64 warmup callbacks, the measured loop renders at least the requested
number of musical bars, rounded up to a whole block. A minimum two-bar default
ensures even single-frame cases cross real boundaries. The output/timing arrays,
PCM and oracle state exist before measurement. Only engine render and its call
bookkeeping are timed; reference validation is outside the timer but inside the
allocation/free measurement. All measured allocations, bytes, frees and freed
bytes must be zero. Engine destruction afterward must free exactly four PCM
buffers: 240000 bytes for mono, 480000 for stereo. This is an owner-side positive
control, not callback deallocation.

Reports include start/end musical frames, measured counter deltas, last observed
boundary/event, exact sorted nearest-rank p50/p99, maximum, nominal load ratio
and overrun count. Warmup length is 64 callbacks, so different block sizes start
measurement at different musical positions; their counter deltas are not a
partition-equivalence assertion. Each result is validated at its actual position.
The [11.3.d acceptance](sequencer-integration-acceptance.md) separately checks
partitioned decoded PCM/event equivalence.

Sorting/report allocation occurs afterward. The bounded longest timing array is
4608000 entries (eight bars at 20 BPM, block 1), about 36.9 MB. Offline overruns
are retained observations; they do not measure driver scheduling, native callback
deadlines or acoustic continuity. The current Windows matrix records 23 such
overruns in 12 of 72 cases, all with block 1; no timing guarantee is inferred.

## Native fixture and lifecycle

`scripts/sequencer-fixtures.mjs NEW_DIR [--channels 1|2] [--sample-rate HZ]`
creates four float32 one-shots and a SHA256 manifest. Defaults are 48 kHz stereo;
the directory and files must be new. `kick.wav`, `snare.wav`, `hat.wav` and
`accent.wav` last 0.12/0.18/0.06/0.24 seconds, with distinct tone/noise shapes,
stereo sides and zero endpoints. Options are checked before directory creation;
existing files are never overwritten. This is diagnostic input generation,
not project/beat export.

The runner prepares all four files asynchronously with existing budgets, then
installs through `Application::initialize_sequenced_sampler`. It checks the exact
native backend/device, selected pattern/mixer/tempo, fresh counters/voices and
absent old acknowledgements. All parsing, file I/O, waits and logging run on the
control side. The runner:

1. Applies BPM while stopped, Plays and observes the requested frame-counted bars.
   Each sampled snapshot is checked against the rational timeline, expected
   attempts, latest event identity/frame and `started + pool_full == attempted`.
   All assets are present in this phase. Manual-trigger telemetry must stay empty.
   Empty patterns are silent; other patterns require a nonzero observed window.
2. Stops/reset, installs a separate beat with row 4 pointing to empty slot 15,
   Plays and observes a missing-asset counter increase. This control phase
   generates sound even after an empty initial workload.
3. Pauses and waits for release to settle. Replaces the pattern with valid slots,
   reroutes future kick notes to channel 4, mutes channels and changes BPM to
   300 (or 95.5 when initially 300). Musical phase, counters, last events and
   captured active-voice routes must remain frozen during these edits.
4. Resumes muted and requires continued scheduling with a subsequent silent
   window. Unmutes, requires a later started note/nonzero window, Stops and checks
   zero phase/voices and cleared last event/boundary. Two later callbacks permit
   final diagnostics publication.
5. Closes, prepares again, checks unchanged metadata and reopens the exact device.
   Selected pattern/mixer/tempo survive; runtime state/acknowledgements reset and
   command IDs remain monotonic. Plays, observes signal, Stops and closes to
   `Uninitialized`. Initial and reopened metrics are logged separately.

Phase logs use `sequencer-native-phase`; the shared acknowledgement helper emits
`sampler-control` with exact IDs. Full pattern and phase/counter snapshots are
retained, rather than invented per-note command acknowledgements. Timing/memory
metrics are cumulative over each complete session, including control phases;
they are not isolated empty/beat/dense percentiles. Native histogram p50/p99 are
approximate bin upper bounds. Metrics reject measured allocation/free, nonfinite
or missing/zero/out-of-range signal, missing callbacks/frames and deadline misses
using the existing validator. Physical short-asset dense patterns may never fill
the voice pool; the offline dense fixture explicitly verifies that policy.

Manual tests execute the same lifecycle with real WAV preparation in mono/stereo
for all three initial patterns, including reopen. Release CLI tests cover the
72-case matrix, 20 BPM, argument validation and feature/profile gates. These tests
do not open hardware. Follow the [protocol](sequencer-native-verification.md)
for measurements and an actual listening observation in 11.4.b.
