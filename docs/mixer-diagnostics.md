# Mixed sampler diagnostics — stage 10.4.a

`mixer-bench` and `mixer-native-check` explicitly exercise the mixed sampler.
The existing sampler diagnostics retain their legacy processing path. The
[local report](../progress/etapa-10-diagnosticos-mezclador.md) records implementation
checks and offline measurements; the [native protocol](mixer-native-verification.md)
defines physical verification. The subsequent
[stage 10 closure](../progress/cierre-etapa-10.md) records measurements, CI and
user-confirmed audible output within its limits.

## Commands and gates

```powershell
cargo run -p daw-cli --release --features offline-diagnostics --locked -- mixer-bench --iterations 2000 --channels 2
cargo run -p daw-cli --release --features rt-diagnostics --locked -- mixer-native-check --input PATH.wav --device EXPLICIT_ID --voices 16 --seconds 1
```

Both commands require release. `offline-diagnostics` enables instrumentation
without CPAL; `rt-diagnostics` additionally enables native audio. A native command
without native support returns NativeAudioDisabled before opening the input.
The native parser requires an input and explicit nonempty device ID and validates
all options before preparation. No fallback output or automatic native execution
is introduced. Ordinary smoke/check/frontend commands remain device-free.

| Command              | Options                                                                       | Defaults and limits                                                                                                                                                                                         |
| -------------------- | ----------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `mixer-bench`        | `--block-size`, `--voices`, `--iterations`, `--channels`                      | All blocks 1/127/256/4096 and voices 0/1/16; 2000 iterations; stereo at 48000 Hz. Explicit block 1–4096, iterations 1–1000000, mono/stereo.                                                                 |
| `mixer-native-check` | `--input`, `--device`, `--sample-rate`, `--channels`, `--voices`, `--seconds` | Required WAV/device; 48000 Hz stereo, initial workload one voice for one second. Rate 100–192000, output mono/stereo, initial voices 0/1/16, window 1–5 seconds. WAV duration seconds+4 through 15 seconds. |

The benchmark reuses the validated benchmark option parser. Native runners share
the existing option parser, bounded wait/transport acknowledgement and metrics
validation helpers. Only their visibility changes; legacy sampler algorithms
and invocation behavior remain unchanged.

## Offline reference and measurement

The benchmark creates four owned synthetic assets before measurement. For frame
`n` and slot `s`, the left sample is `((n % 17) - 8) / [32,64,128,256][s]`;
stereo right is `-left/2`. Each asset lasts `max(2*block,512)` frames. Routes are
slot-to-same-number channel. Settings are:

| Channel | Gain  | Pan | Mute |
| ------- | ----- | --- | ---- |
| 1       | 1     | −1  | off  |
| 2       | 0.5   | +1  | off  |
| 3       | 0.25  | 0   | off  |
| 4       | 0.125 | 0   | on   |

Voices are distributed round-robin over the four slots. The 16-voice workload
has four voices per channel, including four muted voices. They must finish
naturally and be available for subsequent retriggers. Zero/one-voice cases
remain distinct; four assets are retained in every case.

An independent dyadic oracle checks each output sample and each channel's peak
with tolerance 1e-7. It uses fixture coefficients, not the DSP panner or observed
settings, to calculate the reference. Mono ignores pan. The output sum retains
the fixed 1/16 headroom and settled unity master. A 240-frame master warmup and
64 workload warmups precede measurement. PCM, output and timing storage exist
before the measured loop.

Timing covers trigger admission when needed, active-voice inspection and render;
sample/meter validation is outside the timed interval. Allocation/free counters
cover the whole workload loop, including validation, and must remain zero.
Owner-side teardown is measured separately and must free exactly four PCM buffers
with their expected bytes. This positive control is not a callback deallocation.
Sorting/reporting occur afterward. Exact sorted p50/p99, maximum, nominal load
ratio and overrun count are reported. Offline overruns are observations, not a
claim of native callback deadline failure or hard realtime guarantees.

## Native lifecycle and observations

The native runner prepares the input WAV independently into slots 0–3, checking
matching metadata and existing aggregate budgets. It installs the selected
settings through `Application::initialize_mixed_sampler` and verifies the native
backend, mixer presence and exact selected device. All editing, I/O and logging
occur on the control side, outside processing.

1. Observe exact NotPlaying rejection; Play; start the requested 0/1/16 voices
   across slots/channels. With 16 voices, observe exact PoolFull rejection.
   Require stable activity over the frame-counted initial measurement window.
2. Stop and start a separate four-voice control phase. Set gains
   0.25/0.5/0.75/1 and pans −1/+1/−0.5/+0.5. Wait for the accepted FIFO watermark,
   selected targets, zero remaining ramps and exact effective coefficients.
   Require a later measured window with signal on all four musical channels.
3. Reroute slot 0 to channel 4 and trigger again. Existing voices retain
   destinations `[1,2,3,4]`; the fifth voice captures channel 4.
4. Mute all channels and require a subsequent complete zero-level window with
   five voices retained. Pause; route future slot-0 triggers to channel 3;
   change all channels to gain 0.25, centered, unmuted while paused. Require
   frozen transport and preserved captured destinations, then resume.
5. Stop and retain the bank. Retrigger slot 0, confirm its new captured route,
   observe natural completion and Stop again.
6. Close, prepare again, verify unchanged input metadata, and reopen the exact
   device with retained document selection. Counters/voices/acknowledgement reset;
   command IDs remain monotonic. Play, trigger, require a later nonzero output
   window, Stop, validate metrics, and close to Uninitialized.

The initial zero-voice window is silent, but later control/reopen phases generate
sound in every workload. Native diagnostics are cumulative across the entire
session, including those phases. They are not isolated 0/1/16-voice percentiles.
The native histogram's p50/p99 values are approximate bin upper bounds. A fresh
reopened session has separate metrics. Initial native callbacks may already have
advanced frames/meters before the installation reply.
Timing/memory aggregates publish one callback behind; sample probes include the
current callback. Missing final publication is not a zero measurement.

Missing exact trigger outcomes are inconclusive rather than successful. Meter
layout, finite range and nonempty windows are checked. The runner rejects
measured allocations/frees, nonfinite output, zero/out-of-range cumulative signal,
missing callbacks/frames and deadline misses through the shared metrics validator.
Telemetry may omit intermediate callbacks; no lossless peak history is claimed.
Polling is bounded, though OS reads and driver teardown remain noninterruptible.

## Verification boundary

Unit tests run the same control lifecycle with real WAV preparation and manual
render in mono/stereo at 0/1/16 voices, including reopen and exact outcomes.
The benchmark matrix checks samples, meters, natural completion and ownership;
CLI tests check release/feature gates, help, option rejection and dispatch.
Native-enabled release test builds do not open a device.

This increment produces offline evidence and a compiled native runner. It does
not establish physical output, acoustic continuity, a native timing result or
hosted CI for the local mixer revision. Those remain 10.4.b–c.
Subsequent [10.4.b measurements](../progress/etapa-10-medicion-nativa-mezclador.md)
now record six native cases and reopens on Speakers Realtek.
[Implementation delivery and exact-revision CI](../progress/publicacion-mezclador-10.4c.md)
are verified for e7f543f; user listening is confirmed in the stage 10 closure.
