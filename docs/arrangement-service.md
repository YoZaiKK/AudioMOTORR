# Arrangement sessions and application — stage 14.3.a

Explicit manual/native audio sessions now install the
[14.2 timeline engine](arrangement-contract.md). Application use cases resolve
current clip content and consume prepared PCM, separately from the project file.
Project/schema/export integration is 14.3.b; desktop editing follows in 14.3.c.

## Control-side installation and ownership

`AudioHandle` provides:

- `initialize_manual_arranged_sampler(bank, tempo, mixer, resolved)`.
- `initialize_native_arranged_sampler(device_id, bank, tempo, mixer, resolved)`.

These are explicit source modes; no arrangement is inferred from existing
Pattern or Launcher settings. Both transfer ownership through the bounded
control request channel, construct/sort fixed timeline data on the control
owner, and start stopped at musical zero. They do not enqueue Play or restore
old transport, completion or region intent. Empty timelines/banks are legal.
Missing PCM at a scheduled note is an observed MissingAsset outcome.

Bank and resolved-document boxes never enter the realtime FIFO. Initialization
expiry discards them on the control receiver. A matching native device layout
and sample rate are required, with the existing format preference and no
resampling or fallback. Busy precedes feature/device opening while a session is
running. Without native-audio, a closed native initializer reports
NativeAudioDisabled without installing a manual session or changing observations.

Shutdown and failure discard queued controls and timeline observations. Manual
PCM destruction occurs on the control owner. Native callback destruction can
happen on a backend worker; the existing teardown mailbox returns the owning
processor after stream close, so PCM is reclaimed by the stream owner outside
processing. Reopening requires a new explicit bank/content installation.

## Commands and observations

Arrangement sessions use the existing shared command FIFO (128), callback budget
(32) and observation queue (64) for transport, tempo, master gain, manual triggers
and mixer controls. Installation does not consume a command ID. There is no
timeline-edit, replace, seek or loop command in this increment. Pattern/launcher
commands retain their mode gates; rejected modes/full queues consume no ID.

Commands drain once at callback start before any scratch chunk. The 33rd command
waits for the next callback, independently of chunk count. Manual render of zero
frames drains nothing; a valid empty native callback may process controls without
advancing music. Whole-callback preflight prevents partial output on overflow or
invalid buffers, while controls processed before that failure remain processed.

`ServiceSnapshot.arrangement: Option<ArrangementSnapshot>` and the internal Copy
applied snapshot carry the latest engine duration, completion, active region
identities/ranges, exact phase, boundary, note counters and last event. The field
is Some only for an arrangement session; sequencer/launcher fields are None.
No full arrangement, PCM, path, clock extrapolation or serialization crosses
these realtime queues. Snapshot byte size grows; capacities stay fixed.

`CommandAccepted` acknowledges enqueueing; `last_applied_command` observes
processing. Neither proves a note, region transition or completion: consult the
engine's arrangement observations. Scheduled notes do not get service command
IDs or overwrite manual-trigger reports. Active voice counts include both
manual and scheduled notes. Mixer peaks cover the complete callback, including
all converted scratch chunks. Counters/phase freeze at completion while tails
continue, following the engine contract.

A full observation queue drops reports without dropping commands or notes. A
later callback retries with current state and cumulative counters/acknowledgment.
Lost intermediate outcomes cannot be reconstructed, and completion must not be
inferred from silence or an elapsed UI timer. Failure/close clears the field;
fresh sessions start with zero counters/completion and discard stale commands.
Service IDs and observation sequence retain their monotonic lifetime behavior.

## Application preparation and transient context

`Application` provides:

- `initialize_manual_arrangement(preparation_id, document)`.
- `initialize_arrangement(preparation_id, document, device_id)` for native audio.
- `get_arrangement_status()` for installation context and actual service state.

The caller supplies an `Arrangement`; all references are resolved against the
current project's clips immediately before installation. Current tempo and mixer
are captured then, rather than at preparation time. Invalid references and stale
IDs are rejected before consuming a ready bank. Loading/cancelling/no-ready-bank
and audio-close gates retain existing preparation ownership. After taking the
bank it is consumed on success or opening failure, and the preparation permit
is released. No file loading, decoding or implicit re-preparation happens here.

`ArrangementInstallation` records the successfully supplied document, owned clip
content and initial tempo/mixer. It is a transient session context, independent
of selected project values and actual runtime state. Live tempo/mix can change
through existing accepted commands; the original installation context remains
unchanged. Caller edits cannot mutate installed clips. Close/failure/wrong-source
observations invalidate the context. Reopening captures new explicit content.

An initialization timeout can occur after a real service installation. The app
keeps no invented installed context or automatic retry: observed arrangement
state can be Some while `installed` is None. Query/reconcile or close explicitly;
duration or a matching source mode cannot recover the missing content identity.
Further initialization is Busy until audio is closed.

The project is not changed or marked dirty by this installation. Both existing
project playback modes can supply clips, but JSON v2 still stores only Pattern
and Launcher. This API does not promise that saving captures the transient
arrangement. Project/schema integration and timeline export arrive together in
14.3.b; IPC/DTO/desktop controls remain unchanged in this increment.

## Verification and limits

Service cases cover FIFO/budget/mode gates, queue rollback, completion/tails,
Busy/disabled native, close/reopen and monotonic IDs. Internal cases cover lost
observations, callback overflow, stream failure, expired installation and
control-side destruction. Device-free native references cover f32/f64/i16/u16,
mono/stereo and scratch sizes 1/127/256/513 across seven callbacks, with identical
converted PCM and whole-callback observations. Instrumentation checks zero
allocations/frees in processing and positive owner-side PCM reclamation.

Application tests load real WAVs and check current content/mix/tempo, unchanged
project state, immutable installed context, missing references, stale IDs,
loading/cancellation, queue-full rollback, Busy, failed opening and lost replies.
See the [stage report](../progress/etapa-14-sesiones-aplicacion.md). These checks
do not open hardware, measure native latency/timing or establish audibility.
