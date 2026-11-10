# Sampler sessions and trigger observations — 9.4.b

Implemented locally on 2026-11-09, after [engine integration](sampler-contract.md).
This is the audio-library boundary. [Stage 9.4.c](sampler-desktop.md) now wires
application/DTO/IPC/frontend and exposes explicit WAV loading and sampler controls. Tests use the manual processor and
native adapter without opening hardware; physical native sampler audition and
callback timing remain unverified. See the [implementation report](../progress/etapa-9-sesiones-sampler.md).

## Initialization and ownership

AudioHandle::initialize_manual_sampler(bank, tempo) consumes an already validated
SampleBank and derives its mono/stereo output config. initialize_native_sampler
takes device_id, bank and tempo; it selects an advertised matching channel count
and exact sample rate, preferring f32 among supported formats. No app resampling,
downmix or fallback device is added. Existing silence/sine initialization keeps
its default-device-configuration behavior. Native support disabled at compile time
returns NativeAudioDisabled explicitly.

Preparation/file decoding remains outside the controller and processing. Banks
cross the ordinary bounded control-request channel in a Box; no PCM/path/decoder
enters the realtime SPSC queues. Initialization while a session exists returns
Busy and preserves it. Ownership is consumed even on failure; an unsubmitted
request is discarded by its caller, an admitted/expired request by the control
owner. Source payload budgets retain the preparation contract's scope; this is
not a global cap on independent callers' preallocated banks or queued requests.

The manual processor and native callback use the same owning AudioEngine. A
NativeStream guard closes CPAL before reclaiming/dropping the processor. Its
CallbackState destructor returns Box<Realtime> through a dedicated one-slot
teardown mailbox. CPAL may destroy its closure on a backend worker; that worker
transfers ownership rather than freeing PCM. The control owner receives/destroys
it after close, including build/start-failure cleanup.

The teardown mailbox is not a realtime command/outcome queue. Processing never
accesses it, takes its locks or sends owning values. Teardown may wait for CPAL
to release the callback; there is no safe timeout that revokes a still-borrowed
processor. The 10-second request deadline does not bound OS/driver destruction.

## Enqueueing and processing

AudioHandle::trigger_sample(slot, Gain) returns CommandAccepted with a monotonic
command ID after pushing a Copy trigger into the existing 128-entry FIFO. Known
source/invalid/empty-slot errors fail before enqueueing without consuming an ID.
Transport and dynamic voice availability are evaluated by the processing owner,
so Play then Trigger can start while Trigger then Play is processed as NotPlaying.

Engine control and trigger payloads share at most 32 commands per entire callback,
once before native scratch chunking. EngineCommand remains unchanged; a private
audio RealtimeCommand wraps engine controls or slot/gain. The engine's read-only
validate_render(sample_count) preflights the whole callback after commands,
before any chunk advances frames. Per-chunk engine validation remains in place.
An error preserves post-command render clocks/envelope/cursors and leaves output
silent, rather than producing earlier chunks before a later overflow.

Manual render zero remains a no-op. Stop retains PCM for later triggers; shutdown
or failure destroys the session outside processing. A fresh bank/session resets
voices and sampler telemetry; command IDs keep increasing for that AudioService.

## Copy observations, not a per-trigger log

Rust ServiceSnapshot adds optional SamplerTelemetry; silence/sine use None:

| Field              | Meaning                                                                                                                                         |
| ------------------ | ----------------------------------------------------------------------------------------------------------------------------------------------- |
| occupied_slots     | Immutable 16-bit occupied-slot mask for this bank; permits control-side static validation.                                                      |
| active_voices      | Voice count observed after rendering; a delayed observation.                                                                                    |
| last_trigger       | Optional TriggerReport with exact command_id and Copy Result<usize, SamplerTriggerError>. Ok(index) means Started; the voice index is reusable. |
| started / rejected | Cumulative processed-trigger counts within the session, saturating at u64::MAX. Admission failures do not count.                                |

last_applied_command means processed, including dynamic trigger rejection; it
does not mean all covered triggers started. Trigger results include NotPlaying
or PoolFull; defensive RT error variants also cover NoSampler/InvalidSlot/MissingAsset.
The control path normally rejects those static errors before enqueueing.

Snapshots retain capacity 64 and may drop intermediate publications. Later
callbacks recover the latest outcome/ID/counters. A newer non-trigger command
can advance last_applied while last_trigger remains older. Match an exact ID to
claim its result; never assign a newer trigger's outcome to an older pending ID.
No lossless outcome history or guaranteed telemetry after callbacks cease is
provided. Started reports voice activation, not proof of audible hardware output.

These fields are Rust audio-library observations. Stage 9.4.b left DTOs unchanged;
[9.4.c](sampler-desktop.md) now exposes IDs/counters as decimal strings through
generated TypeScript/IPC and explicit loading/session/trigger UI with mock tests.

## Verification boundaries

[Public session tests](../crates/daw-audio/tests/sampler_service_contract.rs) cover
admission versus application, queue/pool limits, FIFO order, release, shutdown,
restart and disabled native support. Private tests cover failure/expired ownership,
telemetry loss and saturation, exact config selection, native/manual sample and
outcome equivalence across a multi-chunk callback, and teardown on a simulated
backend worker. Instrumented tests check zero allocations/deallocations during
processing and owner-side PCM destruction separately. Native build-failure
ownership is simulated without forcing physical device or driver failure.

No new hardware, RSS, deadlines, loopback, disconnect or Linux/macOS native
verification is claimed. Publication/CI must refer to its own exact revision.
