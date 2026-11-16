# Launcher application coordination — stage 13.3.c

The application connects the [v2 document](project-format.md) to
[launcher sessions](launcher-service.md), keeping selected content, queue receipts,
processing reports and actual row transitions separate. All work here runs on
the application/control side, outside audio processing. Desktop DTO/IPC/UI
exposure is implemented in [13.3.d](launcher-desktop.md). See the [stage report](../progress/etapa-13-aplicacion-clips.md).

## Selected document API

`get_launcher_settings()` returns an independent Copy `LauncherSettings` containing
the fixed launcher graph, selected playback mode, quantization and initial scene.
The application provides validated `launcher_set_document`, `launcher_set_clip`,
`launcher_remove_clip`, `launcher_set_clip_step`, `launcher_set_clip_sample`,
`launcher_set_scene`, `launcher_set_scene_action`, `launcher_set_quantization`
and `launcher_set_initial_scene` operations.

These operations edit document content only. They return selected settings,
not a queue acknowledgment, and never relaunch an active/pending clip. They are
available while audio is uninitialized, including Loading/Cancelling/Ready/error
of preparation, or with a healthy launcher session. Other live sources reject
launcher edits; service errors and shutdown cannot be treated as offline edits.
Candidate graphs/indices validate before mutation. Active and accepted snapshots
retain the content captured when their requests were submitted.

`project_set_playback_mode(mode)` requires closed/uninitialized audio, even if
transport is stopped. It may run during preparation and never restarts a device.
Pattern edit APIs and explicit sequenced-session initializers now require selected
Pattern mode; explicit launcher initializers require Launcher mode. A wrong-mode
initialization rejects before consuming a ready bank. The legacy pattern remains
stored independently and can be edited after selecting Pattern mode.

`project_set_tempo(bpm)` returns `TempoEdit { tempo, accepted }`: offline selection
has None, while any healthy live session requires shared-FIFO acceptance before
committing the selected BPM. Invalid/full/inconclusive requests preserve selection.
The previous `transport_set_tempo` retains its live-only receipt API. Existing
mixer edits share the same selected/accepted/applied policy in launcher sessions.

## Explicit launch API

`launcher_launch(LauncherTarget)` supports Scene(index), Clip(row, slot), StopRow(row)
and StopAll, with named convenience methods `launcher_launch_scene`,
`launcher_launch_clip`, `launcher_stop_row` and `launcher_stop_all`.
It requires selected Launcher mode and a ready launcher session; an offline
launch returns Uninitialized rather than retaining work for a later session.

The request snapshots current selected content and quantization in one bounded
Copy payload, retaining Keep as an action. Launch does not change saved initial
scene, document mode, clip content or dirty status. Initial scene is an explicit
document/export default, not the last scene played. Launching while stopped may
select rows when processed but never enqueues Play implicitly.

`launcher_replace_and_launch_clip` is an explicit compound operation: validate
the candidate clip graph, submit its owned snapshot, then commit document content
and last receipt only after queue acceptance. Queue-full or unconfirmed acceptance
preserves both. Ordinary document edits need no queue space; they can succeed
with a full realtime queue because they do not claim to affect playback.

## Receipts and observations

`LauncherLaunch { target, request, accepted }` confirms the service command ID
was enqueued. The application retains only its latest confirmed receipt, in
bounded transient control-owner state; it is never serialized in Project.
`get_launcher_status()` returns selected settings, that receipt, an observation
and `LauncherProgress`:

| Progress              | Meaning                                                                                                                                                   |
| --------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Accepted              | No processing observation for this accepted ID has been seen yet.                                                                                         |
| Processed(request_id) | The latest service report matches the exact command ID and supplies its kernel ID. This does not prove a quantized row transition.                        |
| Rejected(error)       | An exact-ID processing report contains the engine error.                                                                                                  |
| Unknown               | Observed processing has advanced past this command without its exact report. Never infer an older kernel ID or scene outcome from newer/matching content. |

Use `observed.launcher.state.active/pending/last_transition`, phase and actual
note outcomes for applied musical behavior. Processed can coexist with outgoing
clips still active and incoming actions waiting at their grid boundary. Manual
trigger outcomes remain in separate sampler telemetry. The UI must not schedule
notes or advance applied position from a timer.

A timeout is inconclusive. Even if a real enqueue occurred before a reply was
lost, the application does not commit a compound document edit, replace its last
confirmed receipt or send a compensating/replayed command. Query actual state
and let the caller explicitly reconcile. The same rule applies to tempo edits.
Query errors remain errors; failed queries do not manufacture an offline state.

## Preparation, initialization and lifecycle

`initialize_manual_project_sampler(preparation_id)` and
`initialize_project_sampler(preparation_id, device_id)` choose the source from
the current selected mode under application ownership. Pattern mode installs the
latest selected pattern/mix/BPM; Launcher mode installs an empty stopped launcher
with the latest mix/BPM. Explicit `initialize_manual_clip_launcher` and
`initialize_clip_launcher` are also available. No initial scene is launched
automatically, including after opening a launcher project.

These methods reuse existing stale-ID/ready-bank/audio-state gates. They consume
immutable prepared PCM on success or native-opening failure, without reopening
paths, decoding again or substituting preparation-time document settings.
Changing mode, graph, tempo or mix while preparation owns a worker/ready bank
does not mutate preparation metadata or wait for the worker. A wrong mode or Busy
preserves the ready bank; consumed/cancelled/failed banks cannot be reused.

Shutdown discards queued/pending runtime work and clears the last launch receipt
while retaining document content. Observed failed, closing, uninitialized or
other-source sessions clear stale launcher receipts. Reopening starts with no
active clips, pending actions, old receipts or replay; service command IDs remain
monotonic and the new engine's request IDs start fresh. Mode, graph, quantization
and initial scene survive save/open; runtime launch history does not.

Save/export use selected document state independently of audio. Performing a
scene launch does not change the scene later exported by default. The existing
v2 file adapter and scene exporter preserve this separation.

## Verification and boundary

Real-WAV application cases cover latest selection at installation after source
files are deleted, document versus owned active/accepted snapshots, full FIFO,
mode gates, partial-scene supersession, quantization, tempo/transport/manual notes,
preparation failure/cancellation/stale IDs, save/export/reopen and disabled-native
failure without fallback. A synchronized worker covers Loading and Cancelling
without blocking edits. Deterministic internal tests model lost replies after
real enqueues, exact-ID report loss and original failure preservation.

No engine, audio-service, persistence-schema or callback code changes in this
increment. These application use cases may allocate and query the control thread;
they are not called from realtime processing. No new physical hardware timing or
listening is implied. The [desktop layer](launcher-desktop.md) now exposes these launcher controls;
its verification is recorded separately for 13.3.d.
