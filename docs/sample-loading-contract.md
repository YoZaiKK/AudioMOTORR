# Sample loading contract — stage 9.3.a

Specified 2026-11-09; **9.3.b–c implement bounded decoding and complete-bank preparation**.
Bank coordination, single-preparation permits and aggregate bank/coexistence
accounting are implemented locally. 9.3.d verifies injected faults, cancellation,
publication ordering and cleanup threads. 9.3.e verifies end-to-end loaded-bank
playback and consolidates the [acceptance matrix](sample-loading-acceptance.md).
9.3.f closes preparation locally; the
[closure audit](../progress/cierre-preparacion-9.3.md) records accepted evidence,
source fingerprint and outstanding publication/platform limits.
Existing SampleBuffer/SampleBank/Sampler
behavior remains defined by the [sampler contract](sampler-contract.md).
See the [Spanish decision report](../progress/etapa-9-contrato-carga.md) and
[roadmap](roadmap.md). Desktop sampler integration remains stage 9.4.

## Implemented single-resource adapter (9.3.b)

`daw_app::sample_loading::decode_wav(reader, target, cancel)` consumes a Read +
Seek reader and returns one owning SampleBuffer or typed LoadError. It closes
the reader on success/error/cancellation. `load_wav_file(path, target, cancel)`
opens a regular file and delegates to the same implementation. Cancellation
uses a caller-owned AtomicBool; it does not require an Application instance,
service thread, audio device, Tauri, or a new dependency.

The adapter is concrete standard-library Rust for the restricted WAV subset
below. hound remains confined to CLI encoding/test fixtures. It scans all chunks
before reserving PCM, uses a fixed 2048-byte decode scratch plus small header
arrays (compile-time bounded below 64 KiB), and checks cancellation around I/O
and at 256-frame blocks. It checks input length before allocation and again
after decoding, without promising detection of same-length concurrent rewrites.

PCM uses try_reserve_exact once; unexpected excess capacity is rejected. Filling
the validated frame count performs no Vec growth, then creates the immutable
resource and checks cancellation before returning it. Tests with borrowed Cursor
inputs measured one exact PCM allocation at 1-frame and maximum-size mono/stereo
inputs, in debug/release; rejected oversized metadata allocated no PCM, and
invalid PCM/cancellation freed partial output. These tests exclude the caller's
encoded fixture and OS/path/error overhead; they are not total-process RSS tests.

Eight reader behavior tests, an internal reservation-failure injection, three
instrumented memory tests, and an actual mono/stereo export-wav round-trip cover
this adapter. See [the implementation report](../progress/etapa-9-lectura-wav.md).
These two per-resource functions do not enforce concurrency or bank budgets;
use the coordinator below for complete-bank requests.

## Implemented bank coordinator (9.3.c)

`BankPreparationCoordinator::default()` creates one shared permit. Clones share
that permit and may be sent to a preparation worker. `prepare(target, selections,
active, cancel)` synchronously validates 0–16 SampleSelection slot/path pairs,
then loads in ascending slot order into a new immutable bank. All selection and
target checks precede file I/O; metadata/compatibility and budget checks precede
each resource's PCM reservation. Only one file is open at a time. Paths are
borrowed for the call; the result retains only owning PCM and its permit.

An admitted request returns PreparedBank or BankPreparationError. Busy wins
before admission; once admitted, cancellation is checked before validation.
The atomic permit never waits or queues and is held throughout preparation and
the result's pending lifetime. Independent coordinators have independent budgets;
reuse one per loading owner. Direct decoder calls and caller-owned banks outside
that coordinator are outside its accounting.

`PreparedBank::bank()` borrows the immutable result. `usage()` returns conservative
accounted payload/buffer high-water ceilings, not measured allocator peaks.
Each reservation checks retained new PCM plus the incoming Vec, a possible
same-size finalization copy, and the full 64 KiB decoder allowance. The supplied
actual active bank is counted by its resource slices, without trusting an external
byte count. Reported retained PCM counts original source channels, with no mono
expansion or deduplication of repeated paths. Empty banks use zero preparation
payload; their coexistence accounting still includes the supplied active bank.

Dropping PreparedBank destroys PCM before releasing its permit on that thread.
`install(&mut Option<SampleBank>)` consumes the result, moves it into a caller-owned
bank holder, and destroys the old bank before releasing the permit. This is a
library ownership handoff: it opens no stream, sends no command, and does not
install into Application or the audio service. Use it outside processing after
any stream using the destination has shut down. There is no method to extract
an untracked pending raw bank. A cancellation after successful return does not
revoke the prepared value or block this handoff.

Basic validation/order/ownership/concurrency/cancellation tests and real-file
maximum-bank memory tests pass in debug/release. Exact aggregate ceilings and
over-limit/overflow rejection also have checked-arithmetic unit coverage; an
oversized valid bank cannot be constructed through the fixed engine limits.
See [the bank report](../progress/etapa-9-preparacion-banco.md) and the following
fault/cleanup verification.

## Fault, cancellation and cleanup verification (9.3.d)

Private loader/reservation/bank-construction seams keep deterministic fault and
synchronization controls out of the public API. Production still uses the same
concrete decoder and SampleBank::new, with the final cancellation check after
bank construction. There is no public fault injector or diagnostic command.

A 17-case second-asset matrix injects open/metadata/read/seek/truncation/length,
allocation and invalid-PCM failures, and cancellation at header, reservation,
decode, final-length and reader-close checkpoints. Each case verifies slot/error
context, no third asset attempted, at most one open reader, matching open/drop
thread IDs, Busy while the reader is closing, preserved active PCM and permit
reuse. Open/metadata and allocation failures are simulated adapter outcomes,
not induced OS failures or physical OOM. Other faults run the real bounded decoder
against an owned instrumented Read + Seek reader.

Channel handshakes pause after bank construction or inside the second data read.
Cancellation before the final publication check rejects the complete new bank;
cancellation after successful return preserves its pending ownership. A paused
Read is resumed explicitly: cancellation does not interrupt blocked I/O. When
that read returns success or an error, the post-I/O cancellation check wins.
An adapter error already returned before another checkpoint is preserved; these
tests do not imply a total order for all concurrent errors/cancellations.

Thread-local allocator tests separately verify representative real-file failures
after several successful assets, partial decode cancellation on a worker, and
worker-created results discarded/installed on the receiving thread. They measure
PCM cleanup on those threads, excluding worker creation, fixtures and old-bank
allocation. They complement reader drop traces; they do not instrument every
injected matrix case, native callbacks, total RSS or arbitrary scheduler races.
See the [fault/cleanup report](../progress/etapa-9-fallos-cancelacion.md).

## Inputs, ownership, and processing boundary

A preparation request supplies an explicit output EngineConfig and 0–16 occupied
slot/path selections. Empty banks remain valid as in SampleBank. Slot indices
must be 0–15 and unique; two distinct slots may name the same file, in which
case they own separate decoded resources and both count toward the budget.
Validate target mono/stereo configuration and all selections before file I/O.
Process files in ascending slot order, one open input/decoder at a time.

daw-app coordinates preparation policy. A concrete I/O adapter opens seekable
regular files, reads metadata, and decodes PCM on the preparation caller/worker,
outside the audio service/controller/callback and outside React. Reader-based
tests may supply seekable in-memory streams. No decoder, paths, or large buffers
enter realtime command queues. No new crate/trait abstraction is required by
this contract; choose the smallest concrete adapter at implementation.

The operation returns a complete owning SampleBank or an error. It never edits
an existing bank, document, stream, or voices. The caller owns the successful
result until installation or discard. A future installer must shut down the
sampler stream before replacing its immutable bank; preparation is not hot-swap.
Source paths are transient request/error context, not project persistence.

## Supported WAV subset

Accept little-endian RIFF/WAVE with signed 16-bit integer PCM or IEEE float32,
in one or two interleaved channels. Accept corresponding WAVEFORMATEXTENSIBLE
headers, including the float32 files produced by the current export-wav command.
Extension subformat must identify one of these two encodings. Require matching
container/valid precision (16/16 or 32/32); valid-bits zero may explicitly inherit
the container size. Extensible mono masks 0, 1, or 4 and stereo masks 0 or 3 are
accepted; reject other assignments instead of remapping speakers.

WAVEFORMATEXTENSIBLE distinguishes container precision, valid precision, speaker
assignment, and subformat; see [Microsoft's structure reference](https://learn.microsoft.com/en-us/windows/win32/api/mmreg/ns-mmreg-waveformatextensible).
The mask whitelist and valid-bits-zero compatibility rule above are application
policy. hound 3.5.1's current writer uses mask 1 for mono and 3 for stereo;
retain both mono/stereo export round-trip fixtures as implementation acceptance.

Reject PCM8/24/32, float64, compressed audio, RIFX, RF64, Wave64, playlists and
multi-data layouts. No resampling, downmix, peak normalization, clipping, looping,
gain, or envelope is applied during loading. Mono-to-stereo duplication belongs
to the existing playback kernel, not a second decoded buffer.

Metadata must specify a positive sample rate equal to the request's target.
Stereo assets require stereo output; mono assets allow mono or stereo output.
Block alignment must equal channels × encoded sample width, and byte rate must
equal sample rate × block alignment using checked arithmetic. Require 1–1048576
complete frames and exactly aligned data length before allocating PCM.

Conversion is fixed: PCM16 sample s becomes f32(s) / 32768.0, so -32768 becomes
-1 and 32767 becomes 0.999969482421875. Float32 preserves finite input values and
signed zero. Reject NaN, infinities and values outside -1..=1 with the slot and
interleaved sample index. Zero-frame WAV is rejected even though export-wav
supports an empty output file; a playable SampleBuffer cannot be empty.

## Container preflight before PCM allocation

RIFF is chunk-based and chunk bodies have word-aligned padding, as documented
in [Microsoft's RIFF reference](https://learn.microsoft.com/en-us/windows/win32/xaudio2/resource-interchange-file-format--riff-).
The following stricter supported subset is application policy:

1. Measure the opened input length; require at most 16 MiB and RIFF size + 8
   equal to that length. Use checked u64 offsets and a bounded seekable view.
2. Walk at most 4096 top-level chunks. Require exactly one fmt before exactly
   one data chunk. Supported fmt payload sizes are 16/18 bytes for classic
   headers (18-byte forms have cbSize zero) and 40 bytes for extensible headers
   (cbSize 22). Reject inconsistent format/size combinations.
3. Check each header/body/padding boundary against the measured RIFF extent.
   Skip unknown/fact/LIST metadata without retaining bodies, using bounded seeks;
   do not recursively parse metadata. Odd-size chunks consume their pad byte.
   Scan after data as well, detecting duplicates or malformed trailing chunks.
4. Validate encoding, configuration, aligned frame count and decoded byte budget.
   Then seek to the recorded data offset and decode exactly the declared frames.
   A short read or detectable size/metadata inconsistency fails; never silently
   accept partial samples, expand storage, or read outside the bounded extent.

This is a bounded read, not a filesystem snapshot. Concurrent in-place rewrites
that preserve length are not universally detectable; callers must provide a
coherent source. No whole-file consistency/locking guarantee is claimed.

No full-file Vec is created by the adapter. The implementation uses no third-party
decoder or new dependency. Strict length/chunk rules intentionally reject files outside
this supported subset, even if another WAV player accepts them.

## Fixed sample-data budgets

These are ceilings for loader-managed audio payload and parser/read/decode
buffers, not total process RSS. MiB is 1048576 bytes; KiB is 1024 bytes.

| Limit                               | Bytes or count | Meaning                                                                                |
| ----------------------------------- | -------------- | -------------------------------------------------------------------------------------- |
| MAX_ENCODED_FILE_BYTES              | 16777216       | 16 MiB physical/stream length per input, including all metadata and audio.             |
| MAX_RIFF_CHUNKS                     | 4096           | Top-level chunks per file, including fmt/data/ignored chunks.                          |
| MAX_RESOURCE_FRAMES                 | 1048576        | Existing SampleBuffer frame bound.                                                     |
| MAX_RESOURCE_PCM_BYTES              | 8388608        | 8 MiB stereo f32; mono max is 4194304 bytes.                                           |
| MAX_BANK_PCM_BYTES                  | 134217728      | 128 MiB retained PCM across the 16 prepared slots.                                     |
| MAX_DECODER_BUFFER_BYTES            | 65536          | 64 KiB combined parser/read/decode buffers per active preparation.                     |
| MAX_PREPARATION_PCM_EXTRA_BYTES     | 8388608        | One resource-sized extra PCM allocation while Vec is finalized into immutable storage. |
| MAX_PREPARATION_DATA_BYTES          | 142671872      | 136 MiB + 64 KiB: complete new-bank payload + one extra resource + decoder buffers.    |
| MAX_ACTIVE_AND_PREPARING_DATA_BYTES | 276889600      | 264 MiB + 64 KiB: one full active bank plus one preparation under the preceding cap.   |
| CANCEL_CHECK_FRAMES                 | 256            | Maximum decoded frames between cooperative cancellation checks.                        |

Before each allocation, checked sample/frame/byte arithmetic must fit usize and
these budgets. Reserve the decoded Vec once from validated metadata; fill it
without geometric growth or retaining a second integer/float sample array.
Streaming file content does not add a 16 MiB encoded-file allocation. The
decoder-buffer limit includes temporary conversion scratch. The extra PCM
allowance covers possible Vec-to-Box relocation by SampleBuffer::new; the
finished Box already counts toward bank payload. A decoder that cannot fit
these limits is unsuitable without an explicit contract revision.

Preparing the final maximum-size resource can coexist with 120 MiB of earlier
slots, its 8 MiB Vec, a possible 8 MiB final Box, and 64 KiB decoder buffers:
142671872 bytes. With one old 128 MiB bank the ceiling becomes 276889600 bytes.
Only one preparation is allowed per coordinator; concurrent requests return
Busy without entering a queue. A completed, unconsumed prepared bank holds that
permit until installed/discarded, preventing retained results accumulating.

Exclude allocator bookkeeping/fragmentation, request paths/errors, fixed structs,
thread stacks, OS file cache, application/frontend/CPAL/driver memory, and any
caller-owned copies outside this coordinator. Do not describe these payload
ceilings as a 264 MiB cap on the whole app. Per-input bounds and PCM high-water
tests exist in 9.3.b; 9.3.c adds aggregate bank/coexistence/permit enforcement and
instrumented real-file maximum-bank tests. Checked budgets are conservative
ceilings; allocator measurements supplement them without claiming process RSS.

## Errors, cancellation, and cleanup

Use typed application/adapter errors, not callback-formatted strings. Distinguish
Busy, Cancelled, invalid request/duplicate slot, open/read/seek failures, malformed
WAV, unsupported encoding/channel assignment, rate/channel incompatibility,
limit/size arithmetic failure, invalid PCM (slot/index), and allocation failure.
LoadError implements the per-input categories. BankPreparationError adds Busy,
request/duplicate slot errors and Asset errors with slot and nested LoadError.
Cancellation at the bank boundary is BankPreparationError::Cancelled. No IPC
codes were added. Error source chains preserve underlying I/O failures.
Once admitted, check cancellation, validate request, then process slots in
ascending order. For each slot: open/measure, preflight, reserve, decode/validate,
freeze resource. Stop at the first failure; do not publish a partial bank.

Cancellation is cooperative: check before starting, before every file and PCM
reservation, between chunk headers, after at most 256 decoded frames, after I/O
returns, and immediately before publication. A cancellation observed before
publication returns Cancelled and drops partial/new resources on the preparation
thread. Returning the complete bank is the commit point; a later cancellation
does not revoke a value already transferred to its caller. OS I/O can block;
this is not an immediate-cancellation or hard wall-clock timeout guarantee.
At a checkpoint, observed cancellation wins before the next phase; between
checkpoints, the first detected error wins. Do not promise a total ordering of
concurrent cancellation and I/O failures.

All failure/cancellation paths close the active reader, release partial/new PCM
and the coordinator permit, and preserve the prior session bank, transport and
document. Caller discarding a successful result also releases its owned resources
outside processing. [9.4.c](sampler-desktop.md) implements application installation/use through the service and UI.

## Implemented acceptance and local closure

The [acceptance matrix](sample-loading-acceptance.md) maps the following criteria
to concrete suites and their limits. Stage 9.3.e adds four loaded-sampler tests:
18 format/header/channel combinations over four block partitions (72 timelines),
all PCM16 codes at matching 44.1 kHz, empty/full mixed-format banks, and real-file
exact/excess limits. Input files are removed before playback; integer-unit
oracles verify raw sample output without using another decoder/player as a
reference. These tests do not add native transport, envelopes, service or UI.

- Round-trip existing mono/stereo float32 export-wav files with exact nonempty
  samples, plus known PCM16 endpoints and full conversion fixtures.
- Test classic/extensible headers, allowed masks/precision, odd metadata padding,
  unknown/trailing chunks, fmt/data order, duplicates, chunk-count ceiling and
  chunk/RIFF overrun. Zero frames, partial frames and invalid PCM must fail.
- Test exact encoded/decoded/bank limits and one unit above; reject malicious
  lengths and arithmetic overflow before PCM reservation. Enforce buffer/high-water
  limits, including possible finalization copies and old/new banks coexisting.
- Test valid/empty/sparse banks, duplicate/out-of-range slots, same file in two
  slots, rate mismatch and stereo-to-mono rejection; preserve the old bank on failure.
- Inject open/read/seek/truncation/allocation failures and cancellation at all
  checkpoints, including the pre-publication boundary. Observe reader/resource
  cleanup and permit reuse; completed ownership must survive input closure.
- Verify headless/full checks and desktop build for the decoder changes. Existing
  9.2 kernel/CI evidence does not validate this newer implementation.

Preparation 9.3 is closed locally. 9.4.a integrates that bank in AudioEngine
with sampler transport/headroom; see the [sampler contract](sampler-contract.md).
9.4.b connects [audio sessions/outcomes](sampler-service.md). [9.4.c](sampler-desktop.md) adds application/DTO/IPC/UI; the integration audit closes locally in [9.4.d](sampler-integration-acceptance.md); physical native verification follows in 9.4–9.5. Project persistence
remains stage 12. Hosted CI for this worktree remains pending publication and
exact-revision inspection.
