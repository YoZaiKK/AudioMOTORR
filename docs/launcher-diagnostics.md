# Launcher diagnostics

Stage 13.4.a adds release-only CLI diagnostics. The desktop, domain, engine,
audio service, application, project format and realtime callback code are unchanged.
See [acceptance](../progress/etapa-13-diagnosticos-clips.md) and the
[native protocol](launcher-native-verification.md).

```powershell
cargo run -p daw-cli --release --features offline-diagnostics --locked -- launcher-bench --channels 2
cargo run -p daw-cli --release --features offline-diagnostics --locked -- launcher-bench --clips beat --quantization bar --bpm 127.25 --block-size 127 --bars 2
```

`launcher-bench` accepts blocks 1..4096, channels 1/2, clips empty/beat/dense,
quantization step/beat/bar, bars 1..8 and exact BPM presets 20/95.5/127.25/300.
Defaults are 48 kHz stereo, two bars, four blocks (1/127/256/4096), three clip
workloads, three quantizations and BPM 95.5/127.25/300: 108 cases per channel count.
These diagnostic presets do not restrict the BPM supported by the application.

Four deterministic synthetic assets have distinct samples and routes with
gain/pan/mute. An initial owned launch starts four rows. After 64 render warmup
calls, the measured window submits a partial scene plus a superseding step launch,
then StopAll and another launch at fixed iteration fractions. Keep preserves
other pending rows. Quantized targets and row origins may differ across blocks;
cases are checked against their own actual request times, not compared as though
they received commands at identical sample frames.

A separate fixed-storage oracle computes rational boundaries, targets, per-row
origins/content/IDs, pending and applied transitions, voice allocation, rejection
counts, PCM, phase and window peaks. It does not call the engine clock or DSP.
Every sample must be finite and match within 1e-7; active/pending/transition,
note/phase/counter and voice routing observations must agree. Dense clips with
long overlapping samples exercise PoolFull as a checked outcome.

All buffers, requests and timing storage are prepared before measurement. The
timer includes kernel request submission when scheduled and rendering; it excludes
the reference comparison. The allocation counter includes both, and must observe
zero allocations, allocated bytes, frees and freed bytes. Dropping the engine
afterward must free exactly its four owned PCM buffers: 240000 bytes mono or
480000 bytes stereo. Formatting/sorting and output occur afterward.

Reports identify source, profile, rate, channels, block, workload, BPM,
quantization, bars, measured start/end frames, counter deltas, cumulative
transitions/request ID, memory/owner teardown, p50/p99/max and nominal overruns.
The window begins after block-dependent warmup. Times are observations of the
kernel and local scheduler; they include neither IPC/FIFO nor a native callback
and do not establish hard realtime behavior or acoustic continuity. Nominal
overruns are retained even when references and memory checks pass.

`offline-diagnostics` enables these checks without CPAL. `rt-diagnostics` also
enables native audio. Debug/profile and feature gates prevent diagnostic execution
through normal desktop startup; nothing launches automatically. Native options
are validated and NativeAudioDisabled is returned before directory I/O when the
native feature is absent. Existing sampler/mixer/sequencer commands retain their
behavior; the launcher reuses their WAV metadata and meter validation helpers.
