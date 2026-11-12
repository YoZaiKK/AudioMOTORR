# Application mixer coordination — stage 10.3.b

Implemented locally on 2026-11-11 (Mexico City). The application now coordinates
editable project settings with [mixed audio sessions](mixer-service.md).
[Desktop integration 10.3.c](mixer-desktop.md) now exposes selected settings,
receipts, applied observations and measured levels. See the
[implementation report](../progress/etapa-10-aplicacion-mezclador.md).

## Rust application API

- `get_mixer_settings()` returns a Copy snapshot of the selected document settings.
- `mixer_set_channel(channel, gain, pan, muted)` validates and selects a channel edit.
- `mixer_set_route(slot, channel)` validates and selects a routing edit.
- `initialize_manual_mixed_sampler(preparation_id)` consumes the prepared bank
  and explicitly installs the latest selected settings and project tempo.
- `initialize_mixed_sampler(preparation_id, device_id)` performs the same operation
  using the native backend and its exact configuration/device policy.

Existing legacy sampler, silence and sine initialization keeps its prior behavior.
Neither reading mixer settings nor editing the document opens a device, triggers
a sample or issues Play. Live edits reuse existing service commands; the
application methods add no decoding, file access or DSP on the calling thread.

## Selection, acceptance and application

Both edit methods return a Copy `MixerEdit` with selected `settings` and
`accepted: Option<CommandAccepted>`. The optional command is meaningful:

| Service state                                          | Document result                                                         | Receipt / applied state                                                                                      |
| ------------------------------------------------------ | ----------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------ |
| Uninitialized, including sample preparation/ready bank | Commit the validated selection.                                         | `accepted: None`; no command ID is invented and no session is opened.                                        |
| Ready mixed sampler                                    | Enqueue the edit, then commit the document only if enqueueing succeeds. | `accepted: Some(id)` confirms acceptance, not the current target/effective DSP values.                       |
| Ready legacy sampler, silence or sine                  | Leave the document unchanged.                                           | `InvalidArgument`; close audio or initialize a mixed session explicitly.                                     |
| Service failure or request error                       | Leave the document unchanged.                                           | Preserve the original error code/detail. ShuttingDown is Busy; an Error state without detail is StreamError. |
| Invalid values/indices or full command queue           | Leave the document unchanged.                                           | Validation error or QueueFull; no successful receipt and no consumed command ID.                             |

Read `get_engine_snapshot().mixer` and `last_applied_command` for observed targets,
effective ramps, captured destinations and callback meters. Manual zero-frame
renders do not drain pending controls. A native callback may apply an edit before
its invoke reply arrives; the receipt itself does not determine that timing.
Later lossy observations can overwrite intermediate targets, so do not attribute
a newer channel value to an earlier individual edit.

A request timeout is not proof of rejection: an operation may have been queued
without its reply reaching the caller in time. The document remains unchanged
when acceptance is unconfirmed, while the audio side may still apply the edit.
Query actual state before retrying; the application sends no compensating command
and invents no acknowledgement.

The returned settings are the selected document at that edit, rather than an
assertion that playback already uses them. Server-side changes are serialized
by mutable application access; consumers must still handle asynchronous replies
and observations using their normal ordering/disposal rules.

## Preparation and installation

The preparation worker owns file decoding and PCM outside the application lock.
Document-only edits remain available while that worker is loading/cancelling
or holding a ready bank. They do not change preparation IDs, status sequence,
decoded metadata or budgets. Cancellation and loading errors do not reset the
document's mixer selection.

Installation reuses the existing preparation-ID and audio-state gates. Stale,
cancelled or missing banks cannot create a session; Busy retains the existing
session. A valid bank is consumed on success or opening failure. Its immutable
PCM is moved to the audio owner and is never read again from the original paths.
Selected settings are copied **at installation**, so edits made after preparation
are not replaced by a stale preparation-time configuration.

Shutdown discards pending realtime commands while retaining accepted document
intent. A newly prepared session explicitly installs that intent, starting
stopped at frame zero with fresh meter state. It does not replay old commands
or invent their acknowledgements; service command IDs remain monotonic.
Project tempo follows the existing acceptance policy and is also passed explicitly.

## Scope and verification

Tests exercise real WAV preparation with the source file deleted before session
installation, offline/live receipts, targets versus effective ramps, FIFO routing
and paused voices, QueueFull rollback, invalid/wrong-mode edits, shutdown/reopen,
stale/cancelled/failed banks, and native-disabled opening failure without fallback.
The blocked-worker test also checks that document edits do not wait for decoding.
The error-policy unit case preserves failure codes/details and rejects a closing
or failed service as a document-only selection opportunity.

The methods operate on the control side and may allocate for requests and errors.
The engine/audio processing sources and their memory rules are unchanged from
10.3.a. These tests do not establish physical mixed playback, project persistence
or a published revision/CI; integrated native verification remains 10.4.
