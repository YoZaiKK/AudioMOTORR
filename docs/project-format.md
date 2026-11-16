# Project persistence and beat/scene export — stages 12 and 13.3.b

Projects are UTF-8 JSON with `format: "audiomotorr-project"`. The reader accepts
versions 1 and 2; saving always writes version 2. Both store a canonical
decimal-string project ID, title, BPM,
fixed 4×16 step/row-slot pattern, four gain/pan/mute channels, 16 slot routes
and 16 optional asset references. Unknown versions/fields, malformed dimensions,
invalid values and documents above 1 MiB fail before replacing the selected
project. V1 is migrated explicitly in memory; unknown versions are rejected.

## Version 2 launcher fields

All four new fields are required, in addition to the legacy fields above:

| Field           | Required value                                              |
| --------------- | ----------------------------------------------------------- |
| `playback_mode` | `"pattern"` or `"launcher"`; independent of live transport. |
| `quantization`  | `"step"`, `"beat"` or `"bar"`.                              |
| `initial_scene` | Integer 0–15, the default launcher export scene.            |
| `launcher`      | Fixed clip and scene arrays described below.                |

`launcher.clips` has four rows of 16 optional entries. An empty slot is JSON
null; a populated clip is `{"sample_slot": 0, "steps": [16 booleans]}`.
`launcher.scenes` has 16 entries of four actions each: `{"action":"keep"}`,
`{"action":"stop"}`, or `{"action":"launch","slot":0}`. Launch references
the corresponding row's clip slot, not a sample slot or mixer channel. All
references must resolve to populated clips. Unknown fields are rejected in every
object, including Keep/Stop actions; duplicates and malformed dimensions fail.
All clips are installed into a candidate before validating its scene graph;
the selected project is replaced only after the entire file/assets validate.

`daw-domain::Project` owns the launcher, mode, quantization and initial scene.
Defaults are pattern mode, Bar, scene 0 and an empty launcher. These fields
participate in cloning/equality/dirty tracking and do not change a live engine.
The legacy pattern remains separately editable and stored in version 2.

## Version 1 migration

V1 ID/title/BPM/pattern/mix/routes/assets retain their original validation and
exact numeric values. Migration copies the four pattern rows into clip slot 0
and configures scene 0 to launch them, with pattern mode, Bar quantization and
initial scene 0. Other clip slots are empty and other scenes Keep. The copy is
a snapshot, not a live binding to later pattern edits.

Opening does not rewrite the v1 file or start audio. A later explicit save writes
v2 without dropping clip data. Older v1 readers reject v2; changing a v2 version
tag to 1 does not produce a supported down-save. Rust callers use `decode_project`
or `read_project` for either version; `ProjectDocument` is the strict v2 wire
model. `read_project_with_version` additionally reports the original file version,
used by CLI `project-check` rather than labelling a migrated input as v2.

## Portable assets and atomic I/O

Saved references use portable forward-slash paths relative to the project file,
with no absolute path, drive prefix, empty/dot/parent component or backslash.
Saving snapshots referenced samples into a newly created adjacent asset folder,
then commits the JSON. This makes Save As and relocation independent of original
source paths. Earlier asset folders are retained, since other saved documents
may reference them. Missing/unreadable/oversized inputs fail clearly. File I/O
and serialization run on the control side, never in audio processing.

New destinations are exclusive. Replacing an existing document requires an
explicit overwrite request; temporary same-directory JSON and a final filesystem
commit preserve the previous document on errors before commit. New document
creation uses an exclusive link of the completed temporary file. Unsupported
filesystem operations are reported; they do not fall back to truncating a file.

Opening requires closed audio and no active/ready preparation. It validates the
whole document and referenced file metadata before changing selection. Opening
does not decode, open audio or Play. Preparing saved assets is a separate action.
Saving/exporting snapshots selected document state, independently of queued or
applied playback state. A newly accepted file selection becomes document intent;
loading/cancelling/decoder errors do not manufacture an installed bank.

The file contains no PCM, command IDs, preparation IDs, meters, musical position,
active voices, runtime master gain or device choice. Dirty status compares the
selected document with its last saved/opened snapshot. Project title and asset
references are document fields; transport remains transient.

## Pattern and scene export

Beat export constructs an independent `AudioEngine` from decoded saved
or selected assets and document mixer/BPM. Pattern mode uses the existing
sequencer and selected pattern. Launcher mode uses the clip engine and saved
initial scene; `BeatRender::from_scene`/`open_scene` or CLI `--scene 0..15` choose
a scene explicitly, overriding document mode. The scene starts from silence
at musical zero: Keep and Stop leave a row inactive, Launch selects an owned
clip snapshot. There is no performance history or queued/live scene capture.
Configured launch quantization is retained, but cannot delay initial export
selection because it happens while the independent engine is stopped.
It does not use live
`render_manual` or copy DSP into CLI. One bar is four quarter notes, with an exact
frame duration derived from the engine's validated BPM representation. Bars are
bounded 1–4096 and blocks 1–4096, mono/stereo output. Missing assets used by an
enabled export row fail before creating output; unselected clips are not treated
as enabled rows. Every referenced asset still follows the existing validation/
decoding policy. Pool-full uses the live engine's newest
occurrence rejection policy and is reported in export counters.

`cut` ends after the requested musical frames. `finish` disables future pattern
notes, or submits Stop for all clip rows at their next unprocessed Step boundary,
then renders the remaining one-shot voices to natural
completion, without a Stop envelope or another loop's step 0. The engine's 5 ms
master attack and unity target are used. Export reports musical/tail/total frames
and note counters; output is float32 WAV, exclusive destination, RIFF size checked
before opening, bounded scratch storage, errors propagated and partial output
identified if writing/finalization fails. Pending/coincident musical boundaries
at the first tail frame cannot emit another note, even at subframe step rates.

Desktop provides existing title/path, save/open, preparation and beat export
actions through scoped generated IPC. Browser mode stays native-free. CLI gains
document inspection and `export-beat`. Acceptance covers relocation/round-trip,
versions/invalid files/missing assets/write failures without losing selection,
live versus independent export, exact durations/tails and partition equivalence.
Its existing save/open path preserves all v2 fields internally. [Launcher document/
session coordination](launcher-application.md) is implemented in 13.3.c; editor/DTO
exposure remains 13.3.d. Visible pattern controls require selected Pattern mode
and operate on the retained legacy pattern. The CLI can already export saved
scenes independently. No live session, hardware or audio callback participates
in file I/O or export. See the [13.3.b report](../progress/etapa-13-proyectos-escenas.md).
