# Desktop mixer — stage 10.3.c

The desktop exposes the [application contract](mixer-application.md) through
generated DTOs, four additional IPC commands and four React channel strips.
The audio implementation and application policy from 10.2–10.3.b are unchanged.
See the [local implementation report](../progress/etapa-10-desktop-mezclador.md).

## IPC and observation

| Command                    | Arguments                         | Response                                                                                |
| -------------------------- | --------------------------------- | --------------------------------------------------------------------------------------- |
| `get_mixer_settings`       | None                              | `MixerSettingsDto`: four channels and 16 routes from the selected document.             |
| `mixer_set_channel`        | `channel`, `gain`, `pan`, `muted` | `MixerEditDto`: selected settings and optional queue receipt.                           |
| `mixer_set_route`          | `slot`, `channel`                 | `MixerEditDto`, with the same acceptance policy.                                        |
| `initialize_mixed_sampler` | `preparationId`, `deviceId`       | `ServiceSnapshotDto`, after explicit prepared-bank consumption and native installation. |

Channel/slot indices remain zero-based at the boundary. The UI names musical
channels 1–4 and sample slots 0–15. Preparation IDs must be canonical decimal
strings; JSON numbers are rejected. The existing main-window-only capability
grants the four commands individually, bringing the application total to 20.
No filesystem, shell or unrestricted window permission is introduced.

`ServiceSnapshotDto.mixer` is null outside a mixed session. Otherwise it contains
the applied target `settings`, four effective channel/ramp snapshots, 16 captured
voice destinations and complete-callback meters. DTO arrays have fixed lengths.
Meter sequence/frames and command IDs cross JSON as decimal strings, including
values above JavaScript's exact-number range. An unused mono right peak is null.
Bindings are generated with `npm run bindings:generate` and checked against Rust.

## Editing and feedback

Each strip has draft gain, pan and mute values and an explicit Apply button.
Blank/non-finite/out-of-range drafts cannot be sent. Accepted receipts update
the selected document view; errors retain both the previous selection and the
draft for correction. Initial settings are queried independently of device
enumeration, and editing waits for that query. “Consultar ajustes de mezcla”
queries the document again through the same serialized operation guard.
[Acceptance 10.3.d](mixer-integration-acceptance.md) adds a selection-query
generation guard so superseded startup replies/errors cannot replace newer
explicit selection. Offline receipts describe the operation without claiming
the current session is still uninitialized.

Offline edits show that settings were selected for the next session; they have
no invented command ID or applied audio. Live edits show their receipt as pending
until an ordered mixed-session snapshot has `lastAppliedCommand >= commandId`.
The FIFO watermark confirms processing, including when intermediate snapshots
were coalesced. It does not prove that those values are still the latest target
or that a ramp has completed. Current effective coefficients and remaining frames
come exclusively from the observed snapshot.

The hook compares sequences/IDs with BigInt, discards stale/duplicate snapshots
and handles events preceding an invoke response. Pending edits become unknown
when a mixed session is lost. A session-generation guard also covers loss before
the receipt arrives; a later session's watermark cannot revive that confirmation.
The last receipt describes one edit, rather than an unbounded command history.
Timeout/rejection does not fabricate acceptance or automatically retry. Query the
selected settings and inspect the current observation before deciding to retry.
As before, an entirely unobserved external close/reopen cannot be identified from
service snapshots without a public session identity; the desktop serializes its
own lifecycle operations. Multi-client coordination is outside this increment.

Routes are selected explicitly for all 16 slots. The UI shows each applied route
separately and explains that existing voices keep their captured destinations.
Channel edits are available with audio uninitialized or a ready mixed session;
legacy sources, failed/closing sessions and browser preview disable them.
Starting a mixed session is a separate “Inicializar con mezcla” action. It uses
the prepared bank and selected output, refreshes preparation even after failure,
and never issues Play. Legacy sampler initialization remains available.

## Levels and layout

Four channel strips and a master section show actual linear peaks. Channel peaks
are after gain/pan/headroom and before the master envelope. Master peaks are after
that envelope. No measured window is claimed until sequence and frames are both
nonzero. The UI shows the latest observed frame count and sequence and notes that
intermediate callbacks can be missed. It does not synthesize silence, animate
unmeasured peaks, hold maxima or change DSP values. Mono renders one meter per
channel/master; stereo renders two. Numeric values accompany accessible meters.

The grid changes from four columns to two at 760 px and one at 480 px. Labels,
keyboard focus, form submission and disabled states use native HTML controls.
Browser preview invokes no native commands and makes no audio claim.

## Verification boundary

The mock-runtime IPC test installs a manual mixed session and observes real
application/engine processing, ramps, routing and a known `1/32` right peak.
It also checks offline receipts, invalid requests, legacy rejection, shutdown
retention and closed-service errors. Serialization tests cover u64 maxima, mono
nulls and fixed arrays; capability tests reject the new commands from other windows.
React tests cover ordering, queue/timeout failures, session loss, query latency,
device-enumeration failure, explicit initialization and measured levels.

These mock checks remain separate from physical playback and hosted CI.
[Local integrated acceptance 10.3.d](mixer-integration-acceptance.md),
[native measurements](../progress/etapa-10-medicion-nativa-mezclador.md) and
[implementation delivery/CI](../progress/publicacion-mezclador-10.4c.md) are verified.
The user has confirmed audible mixed output;
[stage 10 is closed](../progress/cierre-etapa-10.md). No acoustic continuity
claim follows from processing counters, CI or that listening confirmation.
