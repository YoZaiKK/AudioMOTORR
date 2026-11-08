# Native sine audition and gain ramps

Implemented locally in stage 8, 2026-11-08. This extends the
[offline oscillator](offline-oscillator.md) through the existing service and CPAL
backend. The [realtime contract](realtime-contract.md) governs queues, ownership,
errors, and acknowledgement. The [stage report](../progress/etapa-8.md) records
commands, measurements, and remaining verification limits.

## Explicit initialization and ownership

`AudioEngine::new` and ordinary smoke commands retain silence.
`AudioEngine::with_sine` retains the immediate offline transport behavior used
by WAV export. `AudioEngine::with_ramped_sine` uses that same oscillator with a
per-frame envelope. No generic instrument abstraction or second DSP implementation
is introduced. The engine still depends only on daw-domain.

`daw-audio` can initialize either backend with an explicit sine frequency. Native
initialization validates against the negotiated sample rate; frequency must be
finite, positive, and strictly below Nyquist. Voice peak amplitude is fixed at
0.1 by the service. The application exposes `initialize_audio_with_sine`,
`initialize_manual_sine`, and `signal_set_gain`. Gain is finite and within 0–1;
the service rejects gain commands when the current engine has no sine.
Validation/queue rejection does not consume a command ID. Signal settings are
session playback state, not edits to the minimal project document.

IPC `initialize_audio` adds optional `sineFrequencyHz` (null/omitted means silence)
and the eleventh command, `signal_set_gain`, accepts a gain number. Rust DTOs
remain authoritative and TypeScript is regenerated. React selects silence by
default; choosing sine, entering frequency, and clicking Initialize explicitly
opens the output. Source/frequency changes require closing and reinitializing.
The UI limits frequency entry to 1–20000 Hz; service Nyquist validation still
applies. Mounting the UI never initializes audio. Browser controls stay disabled.

## Envelope and transport semantics

The ramp length is `ceil(sample_rate / 200)` frames: 240 frames at 48 kHz.
Each rendered playing frame updates gain by a constant step before multiplying
the oscillator sample; the last frame sets the exact target. Channels share
the same phase and envelope. Retargeting starts from current gain; requesting
the same target does not restart an in-progress ramp.

- Initialization is stopped, phase/position zero, effective gain zero, selected
  target gain one. Play attacks toward the selected target over one ramp.
- `SetGain` changes the selected target. During playback it ramps from current
  effective gain; while paused/stopped it sets the next Play level.
- Pause and Stop release toward zero. Actual transport remains playing during
  that release and transport position advances for those frames. Pause then
  preserves phase/position. Stop resets phase/position exactly at release end.
  The remaining frames of a larger block are silence.
- Play during release resumes from current phase/gain toward the selected level.
  A later command can supersede the pending transport transition. FIFO ordering
  and the 32-command budget remain unchanged.
- Device frames advance for every valid block. Empty buffers do not advance
  phase/envelope/counters. Invalid renders silence the buffer and preserve all
  engine state, including phase and ramp.

Snapshots distinguish `requestedTransport` from actual `transport`, expose
`sineFrequencyHz`, `sineAmplitude`, effective `gain`, selected `targetGain`, and
`rampRemainingFrames`. These bounded numeric fields use JSON numbers; existing
u64 counters/IDs remain decimal strings. A command acknowledgement means its
target was applied, not that its ramp has finished. Observe actual transport and
remaining ramp frames to confirm completion. UI observations below 15 Hz can skip
the entire 5 ms transition; the UI never claims sample-accurate metering.

Shutdown drops the stream without waiting for a ramp; window closure and device
failure cannot guarantee a fade. Reinitialization resets voice/gain/position and
retains document tempo, service command-ID progression, and observation sequence.

## Verification and optional instrumentation

Automated tests compare sample arrays with the offline oscillator/envelope and
across partitions 1/127/256/4096, including retargeting and partial blocks. A native
adapter unit test exercises the actual chunk/conversion function with a callback
larger than its 256-frame scratch buffer and proves the command budget applies
once per callback. Mock-runtime IPC and jsdom tests open no hardware.

`npm run check` also tests daw-audio with `native-audio,rt-diagnostics` and lints
the diagnostic CLI. The native unit test measures zero allocations in the
processing function; the real-device lifecycle test remains ignored by default.
Linux headless CI keeps its explicit no-default-features package selection.

The optional `rt-diagnostics` feature pins allocation-counter 0.8.1. It replaces
the executable allocator and measures allocations only on the measured thread,
as documented by [allocation-counter](https://docs.rs/allocation-counter/0.8.1/allocation_counter/).
Default desktop builds do not enable it. The feature adds Copy Rust telemetry;
it adds no diagnostic IPC command or UI logging. Controller responses box the
larger snapshot outside the callback; realtime queue payloads remain Copy.

```powershell
cargo run -p daw-cli --release --features rt-diagnostics --locked -- engine-bench --block-size 127 --iterations 20000
cargo run -p daw-cli --release --features rt-diagnostics --locked -- tone-check --seconds 2
```

`engine-bench` renders preallocated stereo buffers at 48 kHz, exercises transport
and gain, and reports allocation count, p50/p99/max processing time, maximum
time/buffer-duration ratio, and count exceeding that nominal duration. Block
sizes are 1–4096, iterations 1–1000000. It opens no device, but this combined CLI
feature compiles CPAL. Its numbers are an instrumented CPU microbenchmark, not
an operating-system scheduling guarantee. Timer resolution can produce 0 ns.

`tone-check` opens real hardware, plays 1–60 seconds (default 2), changes gain,
pauses/resumes/stops, closes, and reopens the same explicit output ID. Options
include `--frequency` and `--device`. It requires callback progress, settled
ramps, nonzero signal, zero measured allocations/non-finite samples/processing
budget overruns, and bounded adjacent-sample changes. Failure returns nonzero.

Native measurement surrounds command drain, rendering, sample conversion, and
snapshot publication. It includes diagnostic overhead and excludes CPAL host,
driver, other threads, callback arrival jitter, and physical output. The nominal
budget is callback frames / sample rate. `maxLoadRatio` measures processing load,
not total CPU utilization. Probe samples are pre-conversion first-channel f32;
all generated channels are identical. Maximum adjacent-sample jump includes
chunk/callback boundaries and is compared to the sine derivative bound plus
the maximum envelope step. It does not establish acoustic continuity or absence
of dropouts. Timing/allocation aggregates publish on the next callback (one
callback behind); sample probes update during the current callback. Metrics
reset on reinitialization and may omit the final unpublished callback.

Physical unplug, other physical formats/devices, long stress runs, loopback
recording, native Linux, and macOS remain unverified. Historical reports and
screenshots retain their original dates and scope.
