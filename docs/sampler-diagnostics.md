# Sampler diagnostics — implemented 9.5.a

Implemented locally on 2026-11-10. This adds the missing sampler-specific CLI
benchmark/runner and extends optional callback observations. It does not perform
human-confirmed physical audition or close stage 9. Native processing is now
measured in [9.5.b](../progress/etapa-9-medicion-nativa.md), separately from listening confirmation. See the
[implementation report](../progress/etapa-9-diagnosticos-sampler.md) and
[native verification protocol](sampler-native-verification.md).

## Features and commands

offline-diagnostics enables the diagnostic CLI and allocator without native-audio
or CPAL. rt-diagnostics keeps its previous native-audio behavior and now includes
offline-diagnostics. Both require release for command execution; debug unit tests
call the same benchmark/lifecycle functions directly. Default desktop still has
no callback measurement instrumentation, diagnostic IPC or DTO fields.

```powershell
cargo run -p daw-cli --release --features offline-diagnostics --locked -- sampler-bench --iterations 2000
cargo run -p daw-cli --release --features offline-diagnostics --locked -- sampler-bench --channels 1 --iterations 1000
cargo test -p daw-cli --features offline-diagnostics --locked --bin daw-cli --test diagnostics_contract
cargo test -p daw-audio --features native-audio,rt-diagnostics --locked
```

sampler-bench defaults to the 12 combinations of blocks 1/127/256/4096 and voices
0/1/16, 2000 iterations each, stereo at 48 kHz. --block-size (1–4096), --voices
(0/1/16), --channels (1/2) select a subset; --iterations is 1–1000000. Unknown,
duplicate, missing or out-of-range options fail before processing. There is no
input file or device in this benchmark.

## Offline oracle and measured region

The benchmark prepares one immutable bank slot and allocates output/timing storage
before measurement. Its deterministic PCM differs between channels: left uses a
17-frame binary-fraction sequence, right an 11-frame sequence. Source length is
max(512, 2 × block) frames. It settles the 240-frame master attack, warms 64 blocks,
then measures actual processing of the selected voice count. Completed voices
retrigger at the next block boundary using the retained bank. Partial final blocks
may have silent tails; voice_frames reports actual source activity, not an assertion
that every output frame has the requested number of voices.

Each timed region covers retrigger decisions/triggers, voice inspection and
AudioEngine rendering. The independent all-sample oracle, finite/peak checks,
timing-array write, sorting and formatting are outside that region. Thread-local
allocator measurement encloses the processing loop including validation. No file
I/O, service request, OS scheduling or physical output is measured.

The report includes exact device/source voice-frame totals, retrigger count,
observed voices before render, channel peaks, allocations/frees, p50/p99/max,
maximum time/nominal-buffer ratio and overruns. Offline percentiles use nearest
rank over the preallocated sorted durations. Timer overhead is included; 0 ns
can occur at clock resolution. An overrun is reported, not disguised by repeating
until a faster run. A completed offline benchmark is not a hardware deadline claim.

After the measured loop, separately measured destruction must free the one PCM
allocation and its exact payload bytes. This is a positive meter/ownership control;
the destruction is not part of rendering or callback timing.

## Native runner, explicitly opt-in

The following command opens a real output and is reserved for 9.5.b. Supply an
existing compatible WAV and the full device ID from devices; it was not executed
during 9.5.a. Six reference runs are now recorded in
[9.5.b](../progress/etapa-9-medicion-nativa.md):

```powershell
Get-FileHash target/stage9-5-fixtures/stereo.wav -Algorithm SHA256
cargo run -p daw-cli --release --features rt-diagnostics --locked -- sampler-native-check --input target/stage9-5-fixtures/stereo.wav --device "<enumerated-device-id>" --sample-rate 48000 --channels 2 --voices 16 --seconds 1
Get-FileHash target/stage9-5-fixtures/stereo.wav -Algorithm SHA256
```

Required --input and --device have no fallback. Defaults: 48000 Hz stereo, one
voice and one-second frame-progress window. --seconds accepts 1–5, --voices 0/1/16,
--channels 1/2, --sample-rate 100–192000. Existing WAV/resource budgets still apply.
The resource must last seconds+4 through 15 seconds, keeping serialized exact-ID
trigger confirmations and natural-completion verification bounded. Preparation
occurs before opening hardware and returns metadata of the exact immutable bank
through the new Rust-only Application::prepared_sample_info; no PCM or file path
enters realtime queues. Disabled native support fails before reading --input.

The runner opens stopped, confirms NotPlaying rejection before Play, confirms
Play/ramp and each successful trigger by exact ID. With 16 active voices it also
requires a PoolFull rejection without losing any voice. It then
checks the requested 0/1/16 voices throughout a frame-based window and logs its
backend/frame progress. It applies gain, pauses and verifies transport remains
frozen while device frames advance, resumes, Stops/clears voices, retriggers from
retained PCM, waits for natural completion and Stops again. It then shuts down,
prepares again and reopens the same selected device, requiring fresh counters and
another exact trigger/close. Tests exercise this lifecycle with real distinct-channel
WAVs and the manual service; they do not call a physical device.

Stage 9.5.b adds normal CLI trace lines for command/applied IDs, typed trigger
outcomes and reopened-session metrics. Formatting/file output stays outside the
callback. The [native evidence](../progress/etapa-9-medicion-nativa.md) preserves a
pilot with the earlier runner and six complete traced runs.

Metrics are cumulative across the entire session/lifecycle, including initial
callbacks, settling and the post-window retrigger. They are not isolated 0/1/16
window percentiles. Exact outcome loss or observation timeout fails as inconclusive;
failed/silent/non-finite/over-budget processing and allocations/frees also fail.
Signal peak must be positive and at most 1.000001. Zero-voice workload sessions
still contain a later one-voice lifecycle test. Raw one-shot jumps are not checked
against the sine derivative bound. Native close may block inside OS/driver calls.

## Bounded callback instrumentation

MemoryActivity derives deallocations/freed bytes from total allocations minus
the signed live balance, so frees of PCM allocated before the measured region are
visible. Saturating aggregate counters stay Copy. Positive tests cover both an
outside allocation freed inside measurement and an allocate/free pair whose live
balance is zero. The existing sine runner also rejects measured frees.

RealtimeMetrics adds deallocations/freed_bytes, measured_frames, a fixed 64-bin
log2 processing-time histogram, sample_values_observed and 32 fixed channel peaks.
Sample probes inspect every pre-conversion f32 channel; maximum adjacent jump is
tracked independently per channel across chunks/callbacks. Snapshot capacities
remain unchanged, with larger but bounded Copy payloads; no growing collection is
introduced. These Rust observations do not change generated desktop bindings.

Native p50_upper_ns/p99_upper_ns are inclusive histogram-bin upper bounds, not
exact percentiles: bucket 0 covers 0–1 ns, bucket k covers 2^k through 2^(k+1)-1,
and bucket 63 ends at u64::MAX. Empty measurements return None, never an invented
zero. max_processing_ns is the actual measured maximum.

Timing surrounds command drain, preflight, render, all-channel probes, conversion
and snapshot publication, including allocator/clock overhead. Aggregation of the
duration/memory result happens after timing. Host/driver work, arrival jitter and
other threads remain excluded. Timing/memory aggregates publish one callback
behind; sample probes include the current callback. Missing final publication is
not zero. Metrics reset with the session. Native deallocation/timing observations
now also have short Realtek/WASAPI hardware coverage in 9.5.b. Other hardware,
formats and acoustic output remain unverified.

Record source/tool/OS/CPU/device/config metadata and WAV hashes before/after the
native run. The runner prints the path and decoded metadata, not an input digest;
same-layout concurrent file rewrites are not detected by its metadata comparison.
Publication/CI, physical audition, loopback, unplug, stress and unverified platforms
retain their separate evidence requirements.
