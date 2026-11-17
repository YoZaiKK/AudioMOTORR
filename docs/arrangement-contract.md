# Arrangement contract — stage 14

Stage 14 adds a deterministic song timeline using the existing one-bar clips.
The standalone domain model and its reference queries are implemented in 14.1;
opt-in engine playback is implemented in 14.2. Sessions, project persistence,
export and desktop editing remain later increments in the
[execution plan](../progress/plan-etapa-14.md).

## Document bounds and identities — implemented

| Property          | Contract                                                                                                                                                  |
| ----------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Rows              | The existing four musical rows; mixer routing still follows sample slots.                                                                                 |
| Region identities | 32 stable slots per row, 128 total; identities do not imply time order.                                                                                   |
| Source            | Clip slot 0–15 in the same row of `ClipLauncher`. No scene or cross-row reference.                                                                        |
| Placement         | Zero-based integer start bar, positive integer length in bars. Fixed 4/4, 16 sixteenth notes per bar.                                                     |
| Horizon           | Exclusive end at most 4096 bars / 65536 steps. Start at most 4095.                                                                                        |
| Interval          | Half-open `[start, end)`. Adjacent regions are valid; overlapping regions in the same row are rejected. Different rows may overlap freely.                |
| Duration          | Maximum exclusive region end across all rows; zero for an empty arrangement. Includes leading and intermediate gaps. No separate trailing-silence length. |
| Content           | Repeat the source's 16 steps for the region length. Each region starts at local step zero, including adjacent uses of the same clip.                      |

`ClipRegion::new` validates bounds before narrowing integer values; zero duration,
overflow-sized inputs and invalid source slots are rejected. Fields are private.
`Arrangement::set_region` validates row/identity, source existence and overlap
before replacing any state. Updating a region excludes its own identity from
the overlap check. Remove is idempotent for a valid cell. An edit never shifts,
sorts or renumbers another region. Region arrays have fixed capacity; no heap
allocation or new dependency is needed by these domain operations.

For example, row 0 can place clip 5 at bars `[1, 3)` and `[3, 4)`, then clip 9
at `[5, 8)`. Bar 0 and bar 4 schedule no notes. At bar 3 the second region starts
at step zero. Duration is eight bars even when another row is empty.

## References and owned snapshots — implemented

Region references are editable document values. `set_region` requires the source
clip to exist at the time of the edit. `references_clip` allows a future application
use case to reject deletion of referenced sources across both scenes and the
arrangement. `ClipLauncher` is independent and cannot enforce references owned
by a separate document: an external source removal can invalidate the arrangement.

`Arrangement::resolve` therefore revalidates every reference and returns either
one complete `ResolvedArrangement` or an error. A resolved region owns its `Clip`
value and placement. Source edits/removal and subsequent region edits cannot alter
an existing snapshot. Resolving again observes the latest source content and
rejects missing sources. No PCM, file path, device, timestamp or live state is
stored in these types.

`region_at_step` and `local_step_at` are pure musical-position queries. They accept
`u64` positions, including values beyond the supported horizon, and return no
region outside the half-open range. They neither advance a clock nor allocate
voices. A snapshot is not a callback command: service ownership/installation and
bounded realtime representation must be specified and verified in 14.2–14.3.

## Engine playback — implemented in 14.2

`AudioEngine::with_arranged_sampler(bank, tempo, mixer, resolved)` explicitly
constructs a stopped timeline engine. The constructor copies/sorts bounded owned
regions outside processing, retaining their stable row/region identities and
source clip content. Render traverses fixed per-row cursors. Default silence,
Pattern, Launcher and manual sampler constructors retain their existing modes.
There is no mutable arrangement installation/edit command in this increment.

The player reuses Sequencer's exact dyadic musical clock, sampler voice pool,
captured sample routing, mixer and 5 ms master envelope. At each due sixteenth
note it expires ending regions before selecting adjacent regions; gaps schedule
no notes. Simultaneous notes are attempted in row order. Missing assets precede
voice-pool checks, rejected notes are counted once without retry, and ending
regions never cut existing one-shot tails. Mixer edits affect future triggers'
routes; active voices retain captured routes as in the other modes.

Whole-window preflight checks device/transport/sampler/meter counters and the
timeline's advancing clock, due boundaries and exact note count before changing
any state. Note counting visits bounded regions and at most 16 partial-bar steps
per region, independently of the requested block's duration. Alignment/overflow
errors silence the whole output and leave state unchanged. Empty renders are
no-ops. Processing, transport, mixer controls and snapshot observations allocate
and free no memory; PCM is released by the owner after processing ends.

### Exclusive end, rounding and tails

The musical end is `duration_steps`. For constant BPM the end frame is
`ceil(duration_steps * sample_rate * 15 / BPM)`, using exact binary BPM.
The clock advances only up to that frame and clamps phase to the exclusive end,
discarding subframe overshoot. At the next nonempty playing render of the due
end frame, the engine drains remaining due boundaries below the end, then records
the end boundary without notes, clears active rows and sets `completed`.
At very low rates several earlier steps can share that same frame; they are
processed in order before completion. Nothing at or beyond the end is repeated.

Rendering exactly the musical frame count can therefore expose end phase with
`completed == false`: the end boundary is still pending at the next frame.
The empty arrangement completes on its first playing frame, with zero musical
frames, one end boundary and no notes. After completion, musical phase/frames,
boundaries, events and note counters freeze. Device/transport/sampler frames
and voices continue while transport is Playing, so tails finish naturally and
further output becomes silent. There is no automatic transport Stop or end fade;
explicit Stop retains the existing master release. Manual sample triggers remain
available through the existing processing-owner API while Playing.

`arrangement_snapshot()` returns duration, completion, observed active region
identities/ranges, exact musical position, last boundary/event and lifetime note
outcomes. Active rows describe applied boundaries; phase can already point at an
unconsumed next boundary. Observations are not queued acknowledgments or a full
event history. The player processes the end boundary once, including its boundary
counter; a four-bar arrangement therefore observes 65 boundaries on completion.

### Transport

Pause and Stop immediately freeze musical scheduling while the existing 5 ms
release renders voices. Settled Pause freezes voices too; Play resumes phase and
pending boundaries once. An interrupted Stop resumes the frozen phase; settled
Stop clears voices, resets timeline phase/cursors/completion and latest
observations, retaining the installed content and lifetime diagnostic counters.
Play after completion does not restart the arrangement; settle Stop first.
Tempo changes preserve musical phase and region placements, recalculating future
frame positions. Completed timelines remain completed through tempo changes or
Pause/Play. Seeking, loops, tempo maps and recording live launch history remain
outside this increment.

## Integration requirements — not yet implemented

Playback is driven by the engine independently of UI timers and live launcher
quantization. Service/application integration must retain this ownership and
separate editable/installed/observed values; never feed quantized UI launch
requests to drive the arrangement.

Use a new explicit arrangement playback mode when integration arrives. Existing
Pattern and Launcher modes retain their behavior. Validate/prepare a complete
owned snapshot outside the callback, with no files, JSON, heap allocation or
destruction in audio processing. Reuse sampler/mixer and exact event scheduling;
do not drive arrangement playback by feeding quantized UI launch commands.

14.1–14.2 deliberately leave `Project`, JSON v2, generated DTOs and desktop controls
unchanged so save/open cannot silently discard an editable arrangement. Add the
project field and a new versioned schema together in 14.3, with explicit v1/v2
migration to an empty arrangement and preserved original playback mode. Readers
must validate the complete graph before replacing the selected project. Persist
regions and source clips, never resolved/live snapshots or playhead state.

Offline arrangement export must use the same engine as playback, preserve silence
in gaps, derive musical duration from the final region, and apply the documented
cut/finish-tail policy. Empty arrangement export must fail clearly. Existing
pattern/initial-scene export remains supported. Desktop will expose a bounded
four-row editor and engine-observed position after service/application integration.

## Acceptance

14.1 domain tests cover all 128 cells, reverse insertion order, full 4096-bar
range, invalid/overflow inputs, all overlap shapes, adjacent regions, cross-row
overlap, atomic errors, missing sources, stable identities, duplicate references,
duration shrinkage, gaps, repeated clip steps and immutable resolved content.
An independent placement/step reference checks every position across a multi-row
fixture; the final bar and `u64::MAX` receive separate boundary coverage.

14.2 compares event/PCM results against independent references across block
partitions 1/127/256/4096/full, low rates and fractional tempos. Include gaps,
adjacent same/different clips, duplicate sample slots, missing assets, full voice
pool, start/end/overflow, Pause/Stop/resume and tempo changes. Instrument processing
allocation/deallocation and ownership teardown. The
[engine report](../progress/etapa-14-arreglo-motor.md) records local verification.
14.3 must add queue/lifecycle,
file/migration/relocation/export and desktop receipt tests. 14.4 must collect
integration, native measurements/listening and exact-delivery CI evidence for
the new path; historical launcher evidence does not establish these results.
