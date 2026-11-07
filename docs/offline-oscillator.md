# Offline oscillator and WAV contract

Updated 2026-11-07. Engine signal generation (7.2) and application/CLI export
(7.3) are implemented locally. Desktop, manual smoke, and native smoke retain
silence. `export-wav` explicitly generates a sine signal without initializing
an audio service, opening a device, or modifying a project.

The [realtime contract](realtime-contract.md) defines the engine/service boundary.
The [roadmap](roadmap.md), [execution plan](../progress/plan-etapas-7-13.md), and
[export report](../progress/etapa-7-exportacion.md) distinguish local evidence from
hosted results. Stage 7 closure and hosted verification of these changes remain pending.

## Ownership and responsibilities

| Package             | Implemented responsibility                                                                                                                                                                                                                         |
| ------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| daw-engine          | Validated sine configuration, phase, transport, and interleaved f32 rendering into caller-owned buffers. No device, filesystem, JSON, or external dependency.                                                                                      |
| daw-app             | offline::OfflineRender owns an independent engine and one scratch buffer, validates the request, renders exact blocks, then delivers samples to a fallible callback after render returns. No Application instance or live audio service is needed. |
| daw-cli             | export-wav parses options, validates WAV arithmetic before file creation, uses create_new to protect existing paths, encodes with hound 3.5.1, explicitly finalizes, and reports failure with nonzero exit status.                                 |
| daw-audio / desktop | Existing lifecycle, queues, DTOs, controls, and silence defaults are unchanged. Native signal playback/ramping belongs to stage 8.                                                                                                                 |

The manual backend's render_manual returns only a frames/silence report; it is
not an export interface. Offline synthesis reuses AudioEngine, with no DSP copy
in CLI. Sink and encoding work occur after the engine returns, outside rendering.

## Engine signal and transport

- AudioEngine::new keeps silence, including while playing. with_sine explicitly
  selects signal, starts stopped, and validates finite frequency strictly between
  zero and half the sample rate and finite peak amplitude in 0–1 inclusive.
- Invalid parameters return OscillatorError::InvalidFrequency or InvalidAmplitude.
  Input parameters are f64, output is f32, and normalized phase is f64 in [0, 1).
- Phase advances once per playing frame and produces the same sample per channel.
  It survives block boundaries, Pause, and repeated Play; Stop resets phase and
  transport position. Paused/stopped output is silence. BPM does not retune it.
- Invalid frame alignment or counter overflow silences the output and preserves
  phase/counters. Empty buffers are a no-op. No public phase setter, live retuning,
  resampling, or click-free transition guarantee is provided.

```rust
use daw_engine::{AudioEngine, EngineCommand, EngineConfig, Tempo};

let config = EngineConfig::new(48_000, 2).unwrap();
let mut engine = AudioEngine::with_sine(config, Tempo::default(), 440.0, 0.1).unwrap();
engine.apply_command(EngineCommand::Play);
let mut samples = [0.0_f32; 256 * 2];
engine.render_interleaved(&mut samples).unwrap();
```

## Application offline API

OfflineRender::new(config, frames, block_frames, frequency_hz, amplitude) validates
mono/stereo output, blocks of 1–4096 frames, and oscillator parameters. These
channel/block limits belong to this use case, not the underlying engine.

Construction allocates one buffer of at most 8192 f32 samples (32 KiB). render
consumes the operation, applies Play to its private engine, and calls a supplied
FnMut(&[f32]) -> Result<(), E> with complete interleaved blocks. The final block
may be shorter. The first sink error stops delivery; zero frames calls no sink.
The operation returns an EngineSnapshot or OfflineRenderError::Render/Sink.
OfflineConfigError distinguishes invalid channels, block size, and oscillator
parameters. No live service snapshot, document, or transport is changed.

## CLI and WAV format

```powershell
cargo run -p daw-cli --no-default-features --locked -- export-wav --output target/tone.wav --frames 48001 --block-size 127
```

The parent directory must already exist and the output must not exist. Choose a
new filename for repeated exports. Options:

| Option            | Policy                                                                                                                                     |
| ----------------- | ------------------------------------------------------------------------------------------------------------------------------------------ |
| --output PATH     | Required, nonempty path; create_new rejects existing files/directories without truncation. No overwrite flag or automatic parent creation. |
| --frames N        | Required exact nonnegative u64 frame count; zero creates a valid empty WAV. No seconds option.                                             |
| --block-size N    | 1–4096; default 256.                                                                                                                       |
| --sample-rate HZ  | Positive u32; default 48000, subject to frequency and WAV byte-rate validation.                                                            |
| --channels N      | 1 or 2; default 2.                                                                                                                         |
| --frequency HZ    | Finite, positive, below half the sample rate; default 440.                                                                                 |
| --amplitude VALUE | Finite 0–1 peak amplitude; default 0.1.                                                                                                    |

Duplicate/unknown options, missing values, and invalid numbers fail. Successful
completion prints export-wav OK only after finalization; exit status is zero.

The pinned writer is [hound 3.5.1](https://docs.rs/hound/3.5.1/hound/struct.WavWriter.html),
confined to CLI. It writes float32 WAVEFORMATEXTENSIBLE with a 68-byte header.
For frames F and channels C, preflight requires F <= floor((u32::MAX - 60)/(4*C))
and sample_rate * 4 * C representable as u32. This protects both RIFF size and
byte-rate fields before a file is created; it does not promise disk capacity.
These limits follow the [pinned writer source](https://docs.rs/hound/3.5.1/src/hound/write.rs.html)
and are tested against actual output headers and boundary arithmetic.

Encoding uses a BufWriter and explicit finalize to expose header/flush failures.
A creation error leaves no new output; a later write/finalization error may leave
an incomplete or partially valid file at the chosen new path. The CLI reports
that path and does not remove or retry it. Inspect/remove it explicitly before
reusing the name. Handles close on failure/success. There is no atomic rename,
power-loss durability guarantee, RF64, or implicit playback.

## Verification

```powershell
cargo test -p daw-app --no-default-features --locked --test offline_contract
cargo test -p daw-cli --no-default-features --locked --test export_contract
cargo test -p daw-cli --no-default-features --locked --bin daw-cli
```

Tests cover exact samples across partitions 1/127/256/4096, mono/stereo,
44100/48000 Hz export, buffer reuse and application independence, zero frames,
partial final blocks, parameter/path/size failures, and injected writer
initialization/sample/final-seek/flush errors with resource closure. Reopened WAV
samples match direct engine rendering exactly in the same build. Engine signal
reference tests additionally cover 96000 Hz and 6 channels; export stays mono/stereo.

The existing check and both CI jobs already select these packages/tests; no new
workflow action or Node dependency is needed. Nine silence CLI scenarios remain
unchanged. No generated WAV is committed. Local evidence does not establish a
hosted success for unpublished changes or realtime deadlines/continuity on hardware.
