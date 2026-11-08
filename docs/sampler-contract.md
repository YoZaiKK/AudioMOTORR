# Sampler contract and PCM resources

Stages 9.1–9.2, 2026-11-08. Implemented now: immutable SampleBuffer resources,
a fixed SampleBank, and the 16-voice raw Sampler kernel with exact fixture tests.
Asset loading, AudioEngine transport/headroom integration, sampler IPC/UI, and
native sampler audition remain later increments. Desktop still offers silence
and sine only. See the [roadmap](roadmap.md),
[current kernel report](../progress/etapa-9-voces.md), [resource report](../progress/etapa-9-sampler.md), and
[realtime contract](realtime-contract.md).

## Implemented resource representation

`daw-engine::SampleBuffer::new(config, samples)` consumes a `Vec<f32>`, validates
it, and retains an immutable `Box<[f32]>`. It exposes configuration, frame count,
and a read-only sample slice. There is no Clone, mutable accessor, shared owner,
decoder, filesystem operation, or device dependency. Debug output contains only
metadata. Construction can shrink/reallocate the Vec; rejection drops it. All
construction, compatibility validation, and destruction belong outside processing.

| Property             | Contract                                                                                                                                                                                   |
| -------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Metadata             | EngineConfig supplies a positive u32 sample rate and positive u16 channel count. Resource construction further restricts channels to 1 or 2.                                               |
| PCM                  | Interleaved f32, finite and normalized within -1..=1 inclusive. Signed zero and exact input bits are preserved. No conversion, clipping, or normalization is performed.                    |
| Length               | At least one complete frame; at most MAX_SAMPLE_FRAMES = 1048576 frames, independently of channel count.                                                                                   |
| Payload bound        | 4 MiB mono or 8 MiB stereo per resource, excluding allocator/metadata overhead and caller/decoder working memory. Approximately 21.85 seconds at 48 kHz; the bound is frames, not seconds. |
| Output compatibility | validate_for_output requires mono/stereo output at exactly the resource sample rate. Mono may map to mono or duplicate into stereo; stereo requires stereo and preserves left/right.       |
| Unsupported mappings | No downmix, resampling, pitch shifting, extra output channels, or implicit rate conversion. Compatibility checking does not render or alter PCM.                                           |

Construction rejects unsupported resource channels, empty PCM, partial frames,
excess frame count, then the first invalid sample, in that order. Invalid-sample
errors identify the interleaved sample index. Output checks reject unsupported
output channels, rate mismatch, then stereo-to-mono mapping. Invalid zero metadata
is already rejected by EngineConfig. Errors are typed `SampleError` values;
formatting messages happens outside the callback.

```rust
use daw_engine::{EngineConfig, SampleBuffer};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mono = EngineConfig::new(48_000, 1)?;
    let stereo_output = EngineConfig::new(48_000, 2)?;
    let resource = SampleBuffer::new(mono, vec![1.0, 0.0, -0.5, 0.25, 0.0])?;
    resource.validate_for_output(stereo_output)?;
    assert_eq!(resource.frames(), 5);
    Ok(())
}
```

The constructor's bound cannot prevent a caller from allocating an oversized Vec
before calling it. A later loader must validate declared sizes, decoded frame
counts, arithmetic, and its working-memory budget before allocating/decoding.
No untrusted-file loader exists in this increment.

## Implemented fixed bank and raw playback kernel (9.2)

SampleBank::new takes an output EngineConfig and an owned fixed array of
16 Option<SampleBuffer> entries. It rejects output channels outside 1–2 and
validates every occupied slot in ascending order, reporting its index and typed
compatibility error. Empty slots/banks are valid. Read-only get/config methods
expose resources; the bank has no mutator or Clone. Failed construction drops
the input on the caller's thread, outside processing.

Sampler::new takes ownership of that bank. trigger(slot, Gain) starts a one-shot
and returns the lowest free voice index, or InvalidSlot, MissingAsset, PoolFull
in that validation order. A returned index is reusable and is not a stable voice
handle. Trigger rejection preserves all cursors/counters. Gain is quantized once
to f32 per voice. render_interleaved overwrites caller-owned mono/stereo buffers;
it neither gates transport nor applies envelopes/headroom. The first six rows
below are implemented. Native headroom and the transport section remain future
integration requirements.

| Decision        | Required behavior                                                                                                                                                                                                                                                                                                                         |
| --------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Bank            | 16 fixed resource slots, immutable for one initialized session. Prepare/validate outside processing, then move the owning bank into the engine before starting callbacks. Missing slots reject triggers. Maximum retained PCM payload is 128 MiB for 16 maximum-size stereo resources; decoding/transient memory needs a separate budget. |
| Voices          | 16 fixed one-shot voices, allocated before processing. A voice stores only a slot index, frame cursor, active flag, and validated constant Gain in 0–1. Multiple voices may read the same retained resource.                                                                                                                              |
| Exhaustion      | Reject the newest trigger when all voices are active. Do not steal, restart, fade, or modify an existing voice. Allocate the lowest free voice index deterministically; process triggers in FIFO order. A zero-gain voice still occupies a slot until its one-shot ends or transport clears it.                                           |
| Timing          | A successful trigger starts with resource frame zero at the next rendered frame after command application. Commands apply between blocks / at callback start. Within-block event scheduling belongs to stage 11. Fixtures split blocks at explicit trigger positions.                                                                     |
| End             | Consume exactly the resource frame count, release the voice after its last frame, and render zero thereafter. No loop, interpolation, per-voice envelope, or automatic retrigger. Natural end can contain a discontinuity if the final PCM sample is nonzero.                                                                             |
| Sum             | The voice kernel sums sample × voice gain in ascending voice-index order, preserving stereo and duplicating mono. Its f32 output is not clipped; exact fixtures may exceed 1.                                                                                                                                                             |
| Native headroom | The later AudioEngine sampler mode applies fixed 1/16 headroom after voice summing, then the master gain envelope. Normalized resources and 16 voices bound full-scale summing. This is a temporary sampler output policy, not the four-channel mixer/pan/limiter planned for stage 10.                                                   |

The owning bank outlives every voice, including paused or finishing voices.
Voices never own/clone/drop PCM, Arc, Vec, Box, or other heap-backed payloads.
Trigger, rejection, natural completion, Pause, and Stop only update fixed state;
none may free resource storage. Stop retains the bank for later triggers. Replacing
resources requires shutdown and rebuilding the bank outside processing. A stream
must stop using its engine before that bank is destroyed. Driver/backend teardown
and destruction are separate from the processing callback's no-deallocation rule.

clear_voices immediately clears fixed voice state without freeing PCM or resetting
the raw rendered-frame counter. Calling it repeatedly is safe. The future
transport adapter must call it after its master release; this method is not a
new Stop command. A fresh Sampler starts with no voices and a zero counter.
snapshot returns Copy inspection state: output config, cumulative rendered_frames,
and 16 optional VoiceSnapshot entries (resource slot, next source frame, f32 gain).
This is a kernel inspection API, not an added service snapshot or IPC DTO.

```rust
use daw_engine::{EngineConfig, Gain, SampleBank, SampleBuffer, Sampler, SAMPLE_BANK_SLOTS};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = EngineConfig::new(48_000, 1)?;
    let mut slots = [const { None }; SAMPLE_BANK_SLOTS];
    slots[0] = Some(SampleBuffer::new(config, vec![0.5, 0.25, -0.25])?);
    let mut sampler = Sampler::new(SampleBank::new(config, slots)?);
    sampler.trigger(0, Gain::new(1.0)?)?;
    let mut output = [0.0; 4];
    sampler.render_interleaved(&mut output)?;
    assert_eq!(output, [0.5, 0.25, -0.25, 0.0]);
    assert_eq!(sampler.snapshot().active_voices(), 0);
    Ok(())
}
```

### Transport and observations

The later integrated sampler mode must retain stage 8's 5 ms master attack/release
and requested/actual transport distinction:

- Initialization is stopped with no active voices, effective master gain zero,
  and selected master gain one. Play alone does not create or retrigger voices.
- Accept a trigger only when both actual and requested transport are playing.
  Triggers while stopped, paused, or releasing are rejected; they are not deferred.
  Play followed by Trigger in the same FIFO can start a voice; the reverse order
  rejects that trigger. Repeated Play does not reset voice cursors.
- Pause advances active voices through the master release, then freezes surviving
  cursors. Play resumes those cursors with attack. Natural completion during a
  release still frees that voice. Stop releases, then clears voices and resets
  transport position; it does not destroy PCM. At zero gain, Stop can clear immediately.
- Master gain changes use the existing retarget policy. Voice gain stays constant
  for that one-shot. BPM does not alter sample rate, cursor speed, or sample pitch.
- Shutdown/failure may close without finishing a fade. Restart begins with no
  voices; session resource settings do not become project persistence implicitly.

Queue acceptance confirms enqueueing, not available voices or successful triggering.
When sampler commands reach the service, cumulative acknowledgement must still mean
the command was processed. Add explicit bounded Copy trigger outcome telemetry:
last trigger command ID and Started/PoolFull/MissingAsset/NotPlaying outcome, plus
started/rejected counters. Validate known asset/argument failures before enqueueing
where possible; check dynamic voice availability only in the processing owner.
Snapshots can omit intermediate outcomes and are not a lossless per-trigger log;
adapters must not infer an older command's result from a newer trigger outcome.
These commands/fields/DTOs are acceptance requirements, not added in 9.1 or 9.2.

Invalid render alignment or checked counter exhaustion must silence the whole
output and preserve all state. The raw kernel already checks alignment and its
u64 rendered-frame counter before advancing any voice; empty buffers make no
progress. Later integration must also preserve transport/master envelope and
the 128-command FIFO/32-command whole callback budget when chunking output.

## Executable fixtures and playback evidence

[sampler_fixtures.rs](../crates/daw-engine/tests/support/sampler_fixtures.rs) contains
small arrays and a bounded generator in Rust. They need no file, decoder, JSON,
window, or audio hardware; all fractions are exactly representable in f32.
The [resource tests](../crates/daw-engine/tests/sample_resource_contract.rs) already
use them to verify ownership/data preservation and metadata. Eight new
[kernel tests](../crates/daw-engine/tests/sampler_contract.rs) exercise the raw
playback oracles below, plus one internal overflow test. They do not establish
native sampler timing or transport integration.

| Fixture       | Metadata and PCM                                           | Raw-kernel oracle (tested in 9.2)                                                                                                              |
| ------------- | ---------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------- |
| MONO_PULSE    | 48 kHz, mono, [1, 0, -0.5, 0.25, 0], 5 frames.             | Trigger at frame 0, gain 1, render 7 frames: [1, 0, -0.5, 0.25, 0, 0, 0]. Stereo duplicates each frame.                                        |
| STEREO_STEPS  | 48 kHz, stereo, [0.25, -0.5, 0.5, -0.25, 0, 0], 3 frames.  | Left/right remain distinct; rendering 5 frames appends four zero samples.                                                                      |
| MONO_TAIL     | 48 kHz, mono, [0.5, 0.25, -0.25], 3 frames.                | Following frame is zero despite the nonzero final sample; no invented release samples.                                                         |
| partition_pcm | 48 kHz mono, 4103 frames; frame i = ((i mod 17) - 8) / 16. | Compare full output, state, and future tail for blocks 1/127/256/4096, splitting at scheduled commands and preserving the final partial block. |

Additional voice cases now tested:

- Two MONO_PULSE voices: gain 1 at frame 0 and gain 0.5 at frame 2.
  Raw mono frames 0–7 must be [1, 0, 0, 0.25, -0.25, 0.125, 0, 0].
- Sixteen simultaneous full-gain MONO_PULSE voices produce raw
  [16, 0, -8, 4, 0]; a seventeenth trigger is rejected without changing them.
  Integrated headroom scales these samples by 1/16 before master gain.
- Final-frame reuse, missing assets, zero-gain occupancy, first-free selection,
  rejection without cursor mutation, clear/retrigger with retained PCM, invalid
  render rollback, exact channel mapping, unsupported bank configurations, and
  ascending f32 summation order (including a cancellation/rounding oracle).
- Pause/resume/Stop/restart with master ramps remain integration acceptance.

[sampler_memory_contract.rs](../crates/daw-audio/tests/sampler_memory_contract.rs)
uses the existing optional allocation-counter dependency under rt-diagnostics;
the engine gains no dependency. It preallocates resources/scratch, then measures
100 rounds of full trigger bursts, exhaustion/missing/invalid rejection, empty
and variable blocks, natural completion, invalid blocks, inspection, and clear.
Zero total allocations plus zero signed net count/bytes establishes zero
deallocations within that measured region. A separate destruction measurement
must report exactly four frees and the retained PCM bytes as a positive control.
Assertions run in debug and release without opening hardware. Windows check/CI
already exercises the feature; default headless suites skip this feature-only
test while running all kernel behavior tests.

The positive-control destruction is deliberately outside processing. These
thread-local memory measurements do not measure callback arrival/deadlines,
physical continuity, or native sampler load. Repeat native release timing/load
and continuity checks when integration exists; stage 8 sine evidence does not
validate sampler callbacks.

## Delivery boundaries

9.1 implements resource validation and fixtures and fixes these decisions.
9.2 implements the bank and fixed voice kernel with the raw oracles above,
including invalid-render rollback and instrumented memory lifetime tests.
9.3 prepares bounded application-side loading/decoding and resource lifetime.
9.4 integrates sampler mode, transport/headroom, service outcomes, and explicit
desktop controls. 9.5 verifies full checks/build, native sampler measurements,
and revision-specific CI before closing stage 9. No mixer/sequencer/persistence
code or new dependencies are introduced by this contract.
