# Project persistence and beat export — stage 12 contract

The first format is UTF-8 JSON with `format: "audiomotorr-project"` and
`version: 1`. It stores a canonical decimal-string project ID, title, BPM,
fixed 4×16 step/row-slot pattern, four gain/pan/mute channels, 16 slot routes
and 16 optional asset references. Unknown versions/fields, malformed dimensions,
invalid values and documents above 1 MiB fail before replacing the selected
project. V1 has no migration predecessor; future versions require explicit migration.

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

Beat export constructs an independent sequenced `AudioEngine` from decoded saved
or selected assets and the selected pattern/mixer/BPM. It does not use live
`render_manual` or copy DSP into CLI. One bar is four quarter notes, with an exact
frame duration derived from the engine's validated BPM representation. Bars are
bounded 1–4096 and blocks 1–4096, mono/stereo output. Missing assets used by an
enabled row fail before creating output. Pool-full uses the live engine's newest
occurrence rejection policy and is reported in export counters.

`cut` ends after the requested musical frames. `finish` disables future pattern
notes at that boundary and renders the remaining one-shot voices to natural
completion, without a Stop envelope or another loop's step 0. The engine's 5 ms
master attack and unity target are used. Export reports musical/tail/total frames
and note counters; output is float32 WAV, exclusive destination, RIFF size checked
before opening, bounded scratch storage, errors propagated and partial output
identified if writing/finalization fails.

Desktop provides explicit title/path, save/open, preparation and beat export
actions through scoped generated IPC. Browser mode stays native-free. CLI gains
document inspection and `export-beat`. Acceptance covers relocation/round-trip,
versions/invalid files/missing assets/write failures without losing selection,
live versus independent export, exact durations/tails and partition equivalence.
