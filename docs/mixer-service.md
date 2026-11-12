# Mixed sampler audio service — stage 10.3.a

Implemented locally on 2026-11-11. The [mixer core](mixer-contract.md) can now be
installed through the existing audio service and both manual/native backends.
[Application coordination 10.3.b](mixer-application.md) is now implemented locally;
[Desktop integration 10.3.c](mixer-desktop.md) now exposes selection, receipts,
applied state and measured levels. See the [implementation report](../progress/etapa-10-sesiones-mezclador.md).

## Sessions and ownership

`AudioHandle` adds:

- `initialize_manual_mixed_sampler(bank, tempo, settings)`.
- `initialize_native_mixed_sampler(device_id, bank, tempo, settings)`.
- `set_mixer_channel(channel, MixerChannel)` and `set_mixer_route(slot, channel)`.

Initialization moves an owning `SampleBank` and Copy `MixerSettings` to the
control owner. It constructs `AudioEngine::with_mixed_sampler` outside processing,
without file access, decoding or copying PCM. Native configuration must match
the bank rate and output layout exactly; the existing format preference and
explicit device selection remain in force. Without `native-audio`, native calls
return `NativeAudioDisabled` and do not simulate a device.

The engine starts stopped with the selected channel settings applied. Initial
manual meter windows have sequence/frames zero; native callbacks may already
have run before the initial service reply. Reinitialization while running is Busy.
Shutdown clears mixer observations and voices, retaining neither the bank nor
old commands. Opening another session requires a fresh prepared bank and explicit
settings. Command IDs and the service observation sequence survive restart.

Manual PCM destruction occurs on the control owner. Native sessions retain the
existing teardown mailbox: after stream close, the processing object returns
to its owner even when a backend worker drops the callback. Resource destruction
is excluded from processing; expired requests reclaim their owned bank on the
receiving control thread.

## Queue acceptance and applied state

Mixer edits are validated on the control owner, then enter the same bounded
Copy FIFO as transport, master gain and triggers. Capacities remain 128 commands,
64 observations and at most 32 commands drained once per callback. No bank,
String, Vec, Arc or heap-owning payload enters that realtime FIFO.

`CommandAccepted` confirms enqueueing. Applied targets and ramp values are
observed through `ServiceSnapshot.mixer: Option<MixerSnapshot>`, together with
`last_applied_command`. Queue overflow rejects the edit without consuming a
command ID or modifying applied settings. Invalid indices and wrong-source
calls fail before enqueueing. Routes for empty but valid bank slots are allowed.

FIFO order determines trigger-time destinations: Trigger/Route/Trigger can
create two voices with different captured destinations. Changing a channel
affects its already assigned voices through the existing ramps. Snapshots are
lossy; a later snapshot may overwrite intermediate settings or meter windows.
Its cumulative acknowledgement cannot identify an earlier individual value from
the most recent channel state. The next callback retries publication with the
latest applied state after a full observation queue is drained.

## Callback meter windows

The engine's direct `mixer_snapshot` retains its per-render-call window. The
service aggregates all render chunks into a **single backend callback window**:

- Channel peaks remain post channel gain/pan/headroom and pre master, including
  independent contributions that cancel in the output.
- Master peaks observe actual post-envelope f32 samples before native conversion.
- `meters.frames` counts the whole callback, including any silent tail after
  Pause/Stop. `meters.sequence` increments once per completed nonempty callback.
- Empty callbacks preserve the last completed meter window. The callback counter
  can advance independently of the meter sequence; manual zero-frame calls remain
  complete no-ops, including command draining.
- Mono exposes `right: None`; silence resets peaks to zero in the next nonempty
  window. Lost windows are not silently turned into peak hold across callbacks.

Production native rendering still uses 256-frame scratch chunks. The processor
preflights the whole output and all required engine-window sequence increments
before rendering the first chunk. Callback counters and aggregate windows are
checked for overflow. Invalid output/scratch layouts and failed invariants
silence the callback and signal a stream error without rendering a partial valid
prefix. Valid controls already drained at the callback boundary remain processed
if a subsequent render preflight fails; no audio-clock advancement is implied.

Stream failure clears service mixer telemetry so a prior peak is not presented
as a ready session. Fresh sessions reset callback/meter state and discard pending
old-session edits. This logic uses only fixed Copy state during processing and
adds no allocations, frees, blocking, formatting or I/O there.

## Verification scope

Tests compare a 1001-frame callback against an unsplit reference using scratch
sizes 1/127/256/513 and f32/f64/i16/u16 conversions. Independent early, middle
and late PCM pulses prove that the final scratch chunk cannot replace the peak
of the whole callback. FIFO budgeting, queue loss/recovery, cancellation,
empty/mono windows, errors, overflow, shutdown/reopen and owner cleanup have
dedicated cases. Optional instrumentation measures the complete adapter path,
including command drain, conversion, aggregation and publication.

These tests call the native processing adapter without opening hardware.
Physical mixed playback, real callback timing and hosted CI for the delivered
revision remain stage 10.4; sampler-stage hearing evidence is not attributed
to the new mixed session path.
