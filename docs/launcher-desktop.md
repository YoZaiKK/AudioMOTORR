# Launcher desktop contract — stage 13.3.d

The desktop exposes [application coordination](launcher-application.md) through
eight scoped commands. Only the local `main` window has their generated Tauri
permissions; the capability does not grant remote access or shell execution.
Commands execute through the existing serialized application access on a blocking
worker. Audio callback code and queue budgets are unchanged.

| Command                            | Contract                                                                                                                                                        |
| ---------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `get_launcher_settings`            | Selected mode, quantization, initial scene, 4 × 16 optional clips and 16 four-row scenes.                                                                       |
| `get_launcher_status`              | Selection, last confirmed owned receipt, processing progress and service snapshot.                                                                              |
| `launcher_edit`                    | Tagged clip creation/replacement/removal, step/sample edits, scene or row-action replacement, quantization or initial scene. Document edits never launch audio. |
| `launcher_launch`                  | Tagged scene/clip/row-stop/all-stop target. Returns service acceptance and owned actions/content/quantization.                                                  |
| `launcher_replace_and_launch_clip` | Explicit compound operation; commits replacement and receipt after queue acceptance.                                                                            |
| `project_set_playback_mode`        | Requires closed audio. Does not initialize or play.                                                                                                             |
| `project_set_tempo`                | Offline document edit or accepted live tempo update.                                                                                                            |
| `initialize_project_sampler`       | Installs prepared PCM using the selected mode and latest settings; starts stopped, without a scene launch.                                                      |

Inputs use fixed arrays, tagged enums, `u8` indices and application/domain
validation. Nested unknown fields, including fields on Keep/Stop, are rejected.
Malformed invoke arguments are rejected by Tauri; application failures keep the
existing typed `ApiError`. Engine rejection reasons are display strings attached
to a tagged rejected outcome. Preparation IDs accept only canonical decimal
`u64` strings. There is no generic method dispatcher.

Generated TypeScript bindings expose every `u64`/`u128` ID, clock, origin, target,
counter and phase fraction as decimal strings. Bounded indices remain numbers.
Service command IDs and engine request IDs occupy separate fields. Project
documents now include launcher settings; saving/opening preserves both pattern
and launcher content. Export uses the selected mode and saved initial scene.

`engine-snapshot` adds optional launcher telemetry: active owned content and
origin/request per row, pending actions and target/request, last applied
transitions, exact shared clock/fraction, counters, last boundary/note and the
latest processing report. `rowSteps` is computed by the engine from that same
snapshot. It projects the local step at the next audio frame; its boundary may
not yet have run. The UI labels that projection separately from observed notes
and transitions, and does not advance time in JavaScript.

The desktop hook shares existing action serialization and snapshot subscription.
Selected content changes only after confirmed edit responses; audio events cannot
replace it. Sequence comparisons use `BigInt` and ignore duplicate/stale events.
A receipt is Accepted until its **exact service ID** matches a processing report.
Processed exposes the engine ID but does not imply an applied transition. A
watermark without that exact report yields Unknown. Known results are retained
through later snapshots and a late status-query reply for the same service ID
and session. A different receipt or closed session never inherits that proof. Closing/failure clears transient receipts and invalidates
late acknowledgements; reopening never replays them. Explicit status consultation
can recover the application's latest confirmed receipt without submitting audio.
Failed actions preserve selection; timeout has no automatic retry/compensation.

The UI provides a scrollable clip matrix, independent selected-clip editor,
per-scene Keep/Stop/Launch controls, quantization, initial export scene, explicit
launch/row-stop/all-stop and current-mode initialization. Active and queued badges
come only from engine telemetry. Pattern controls and its initializer are disabled
when the project selects Launcher. Browser mode makes no native invokes.

Verification is recorded in the [desktop stage report](../progress/etapa-13-desktop-clips.md).
[Integrated acceptance 13.3.e](../progress/cierre-integracion-clips-13.3e.md) now covers
WAV/IPC sessions, portable files and independent PCM export checks, plus the real
WebView2 file workflow with audio uninitialized. Preparation polling checks the
latest synchronously observed state before scheduling another IPC request.
Native device timing/listening remains 13.4 work.
