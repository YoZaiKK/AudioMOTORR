# AudioMOTORR / Rust DAW

A desktop music application built with Rust, Tauri 2, React, TypeScript, and Vite.
The project is building a foundation for a four-channel, 16-step groovebox that
can later evolve into a clip-based DAW.

## Current status

Stages 1 and 2 are complete: project setup and a deterministic silence-rendering
engine with transport controls. The engine runs without a window or audio device.
The desktop UI currently displays development status; it does not control the
engine yet.

Implemented capabilities:

- A minimal project document with a validated ID, title, and tempo.
- Silence rendering into caller-owned, interleaved `f32` buffers.
- Play, Pause, Stop, and tempo changes between processing blocks.
- Separate device-frame and transport-position counters.
- Small, copyable engine snapshots and explicit configuration/render errors.
- Eight behavioral tests covering validation, rendering, transport, and errors.

Verified on Windows: reproducible npm installation, prerequisite diagnostics,
frontend checks and build, core tests, workspace Clippy, and a debug desktop build
without an installer. These results are recorded in the implementation log;
visual window checks and real-device audio checks remain pending. Linux and macOS
are intended targets but have not been verified.

Project tracking documents are currently maintained in Spanish:

- [Six-stage implementation checklist](progress/checklist.md).
- [Implementation log, versions, and verification results](progress/detalles.md).
- [Original requirements](progress/rust-daw-bootstrap-prompt.md).

## Windows setup

Requirements:

- MSVC x64 Build Tools and the Windows SDK.
- Microsoft Edge WebView2 Runtime.
- Rust installed through rustup; toolchain **1.90.0** is pinned in `rust-toolchain.toml`.
- Node **22.23.4**, pinned in `.node-version`, and npm **10.9.9**.

See the [official Tauri prerequisites](https://v2.tauri.app/start/prerequisites/)
for system setup. The project does not install system prerequisites automatically.

Run from the repository root in PowerShell:

```powershell
rustup toolchain install 1.90.0 --profile minimal --component rustfmt --component clippy
npm ci
npm run doctor
```

The doctor script only inspects tool versions, MSVC, the SDK, and WebView2 registry
entries. It reports checks it cannot complete and does not install tools or change
system settings.

## Development

Start the browser UI:

```powershell
npm run dev
```

Open `http://127.0.0.1:1420`. The UI currently uses Spanish text and displays a
notice that audio functionality requires Tauri when running in a browser.

Start the desktop application:

```powershell
npm run desktop:dev
```

This starts Vite automatically and opens the application inside Tauri. Stop any
existing `npm run dev` server first: the development port is fixed and startup
fails if it is already occupied.

## Checks and builds

Run the current verification sequence:

```powershell
npm run check
```

This runs TypeScript checks, ESLint, Prettier, the frontend build, Rust formatting,
domain and engine tests, and workspace Clippy with warnings treated as errors.
It stops at the first failure and currently requires desktop prerequisites because
Clippy includes the Tauri package. Generated-binding checks will be added with IPC.

To check only the core, without Tauri or an audio device:

```powershell
cargo test -p daw-domain -p daw-engine --locked
cargo clippy -p daw-domain -p daw-engine --all-targets --locked -- -D warnings
```

Individual checks and builds:

```powershell
npm run typecheck
npm run lint
npm run format:check
npm run build
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
npm run desktop:build -- --debug --no-bundle
```

The final command builds the debug executable at
`target/debug/rust-daw-desktop.exe` without creating an installer. Running
`npm run desktop:build` without options uses Tauri's configured packaging settings.
It does not publish or sign the application.

Use `npm run format` to format frontend files and documentation, and
`cargo fmt --all` to format Rust code.

## Repository structure

| Path                 | Responsibility                                                |
| -------------------- | ------------------------------------------------------------- |
| `src/`               | Initial React UI and runtime detection.                       |
| `src-tauri/`         | Tauri window setup and application composition.               |
| `crates/daw-domain/` | Minimal project document, IDs, and validated tempo.           |
| `crates/daw-engine/` | Deterministic silence rendering, transport, and snapshots.    |
| `crates/daw-app/`    | Tauri-independent application information; future use cases.  |
| `scripts/`           | Windows diagnostics and verification orchestration.           |
| `progress/`          | Requirements, implementation checklist, and verification log. |

The repository has one virtual Cargo workspace, one root `Cargo.lock`, one shared
`target/` directory, and one npm `package-lock.json`. The Rust workspace uses edition
2024 and resolver 3. Without an explicit package selection, `cargo check` checks
`daw-domain`, `daw-engine`, and `daw-app`; use `--workspace` to include desktop.

The engine depends only on the domain crate. Neither crate depends on Tauri, CPAL,
React, or external libraries.

## Core behavior

### Project document and playback state

`Project` stores an editable document independently of `AudioEngine`. Project IDs
are caller-supplied, nonzero `u64` values; global uniqueness is not generated.
Titles are trimmed and must not be empty. The default document has ID `1`, the
title "Proyecto sin título", and a tempo of 120 BPM.

`Tempo::new` accepts finite values from 20 through 300 BPM, including both bounds.
Editing the document does not change playback automatically: the caller must
explicitly apply `EngineCommand::SetTempo` to the engine.

### Rendering

`EngineConfig::new(sample_rate, channels)` accepts positive `u32` sample rates and
positive `u16` channel counts. These are the silence renderer's capabilities;
a future audio backend must also validate device-specific constraints.
`AudioEngine::new(config, tempo)` starts stopped with both counters at zero.

The caller provides a buffer to `render_interleaved`. Each frame contains one
`f32` sample per channel in interleaved order. The engine overwrites every sample
with zero and accepts variable block sizes, including an empty buffer. It does
not allocate memory, acquire locks, perform I/O, or log during rendering.

Buffers must contain complete frames. A misaligned buffer or counter overflow
returns `RenderError`, silences the entire buffer, and leaves the engine state
unchanged. Counter increments use checked arithmetic.

### Transport and snapshots

The caller applies `Play`, `Pause`, `Stop`, or `SetTempo(Tempo)` through
`apply_command` between render calls:

- Device frames advance with every valid rendered block, including while stopped
  or paused.
- Transport frames advance only while playing.
- Pause preserves position; Play resumes it; Stop resets transport position without
  resetting the device counter.
- Tempo changes preserve both counters and do not schedule musical events yet.

`snapshot()` returns a small copy containing configuration, tempo, transport state,
and both frame counters. Applying commands is currently synchronous; there is no
command queue or distinction between command acceptance and application yet.
Future sequencer notes will require offsets within a processing block.

## Verification coverage and next steps

The eight core tests cover project and tempo validation, silence across different
channel counts and consecutive block sizes, identical results across different
block partitions, transport behavior, invalid configuration and partial frames,
counter overflow, and independence between document edits and playback state.

The render path has been inspected for allocations and blocking operations.
Allocator instrumentation, real-time deadline measurements, native audio, and
hardware validation have not been performed.

Stage 3 will add `daw-audio`, deterministic manual and optional CPAL backends,
bounded inter-thread queues, audio-service lifecycle management, application use
cases, and the `daw-cli` smoke command. Stage 4 will connect the UI through IPC.

There is currently no device output, synthesizer, sampler, sequencer, project
persistence, or WAV export. The first complete audio foundation will process
silence; an oscillator is a later milestone.
