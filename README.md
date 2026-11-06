# AudioMOTORR / Rust DAW

A desktop music application built with Rust, Tauri 2, React, TypeScript, and Vite.
The project is building a foundation for a four-channel, 16-step groovebox that
can later evolve into a clip-based DAW.

## Current status

Stages 1–3 are complete: project setup, a deterministic silence-rendering engine,
and an audio service with manual and native CPAL backends. The CLI can process
exact frame counts offline or open a real output device. The desktop UI currently
displays development status; connecting it through IPC is stage 4.

Implemented capabilities:

- A minimal project document with a validated ID, title, and tempo.
- Silence rendering into caller-owned, interleaved `f32` buffers.
- Play, Pause, Stop, and tempo changes between processing blocks.
- Separate device-frame and transport-position counters.
- Small, copyable engine snapshots and explicit configuration/render errors.
- A bounded control thread, realtime SPSC queues, and accepted/applied command IDs.
- Device enumeration, supported output configurations, and explicit device selection.
- Audio initialization, shutdown, failure reporting, and restart.
- CLI smoke checks and behavioral tests for rendering, queues, and service lifecycle.

Verified on Windows: reproducible npm installation, prerequisite diagnostics,
frontend checks and build, core tests, workspace Clippy, and a debug desktop build
without an installer. Native CLI checks opened the Realtek WASAPI output at
48 kHz, stereo, `f32`, confirmed callbacks and applied Play/Pause/Stop, closed it,
and reopened it by explicit device ID. These results are recorded in the
implementation log. Visual window checks remain pending. Linux and macOS are
intended targets but have not been verified.

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
headless tests and Clippy, then workspace tests and Clippy with warnings treated
as errors. It stops at the first failure and requires desktop prerequisites for
workspace checks. The desktop package enables native audio; automated tests do
not open hardware. Generated-binding checks will be added with IPC.

To check only the core, without Tauri or an audio device:

```powershell
cargo test -p daw-domain -p daw-engine -p daw-audio -p daw-app -p daw-cli --no-default-features --locked
cargo clippy -p daw-domain -p daw-engine -p daw-audio -p daw-app -p daw-cli --no-default-features --all-targets --locked -- -D warnings
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

## Audio CLI

Render and verify silence offline, without a device or Tauri:

```powershell
cargo run -p daw-cli --locked -- smoke --frames 48001 --block-size 127
cargo run -p daw-cli --locked -- smoke --seconds 1 --block-size 4096
```

Smoke uses 48 kHz stereo, checks exact device and transport frame counts, verifies
silence and the applied Play command, and exits with a nonzero status on failure.
Choose either integer `--seconds` or `--frames`; the default is 48000 frames.
Block sizes range from 1 to 4096; a final partial block preserves the exact total.
`--frames 0` verifies an empty run without applying the queued Play command.

Enumerate outputs or run a bounded native smoke check:

```powershell
cargo run -p daw-cli --features native-audio --locked -- devices
cargo run -p daw-cli --features native-audio --locked -- live-smoke --seconds 2
cargo run -p daw-cli --features native-audio --locked -- live-smoke --seconds 2 --device "<ID from devices>"
```

Native smoke confirms callbacks and applied Play/Pause/Stop before shutting down.
Its duration is 1–60 seconds (default 2), plus bounded acknowledgement waits. It
produces silence and requires an available audio device; it is not part of the
automated test suite. Native support is opt-in for `daw-audio`, `daw-app`, and
`daw-cli`, and enabled by the desktop adapter. Without the feature, native APIs
return `NativeAudioDisabled` instead of simulating a device.

## Repository structure

| Path                 | Responsibility                                                                |
| -------------------- | ----------------------------------------------------------------------------- |
| `src/`               | Initial React UI and runtime detection.                                       |
| `src-tauri/`         | Tauri window setup and application composition.                               |
| `crates/daw-domain/` | Minimal project document, IDs, and validated tempo.                           |
| `crates/daw-engine/` | Deterministic silence rendering, transport, and snapshots.                    |
| `crates/daw-audio/`  | Audio service, bounded queues, manual rendering, optional CPAL backend.       |
| `crates/daw-app/`    | Project coordination and audio/transport use cases shared by desktop and CLI. |
| `crates/daw-cli/`    | Offline smoke, output enumeration, and native smoke commands.                 |
| `docs/`              | Audio-service ownership, limits, failure behavior, and restart policy.        |
| `scripts/`           | Windows diagnostics and verification orchestration.                           |
| `progress/`          | Requirements, implementation checklist, and verification log.                 |

The repository has one virtual Cargo workspace, one root `Cargo.lock`, one shared
`target/` directory, and one npm `package-lock.json`. The Rust workspace uses edition
2024 and resolver 3. Without an explicit package selection, `cargo check` checks
the five headless packages; use `--workspace` to include desktop and native audio.

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
the audio backends additionally limit output to 1–32 channels and CPAL uses the
selected device's default supported configuration.
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

The engine's `snapshot()` returns a small copy containing configuration, tempo,
transport state, and both frame counters. Direct engine calls are synchronous.
The audio service instead returns `CommandAccepted { command_id }` when a command
enters its realtime queue. `ServiceSnapshot.last_applied_command` confirms the
latest applied command; accepting a command does not confirm playback state.
Future sequencer notes will require offsets within a processing block.

The service exposes a monotonic observation sequence, callback count, separate
audio-service state, engine snapshot, and errors. Native snapshots also identify
the actual selected output and sample format. Shutdown clears playback state;
reinitialization starts stopped at frame zero with the application's current
document tempo. Command IDs and observation sequences survive restart within the
same service instance. Shut down a ready service before selecting another output.

See the [audio-service contract](docs/audio-service.md) for queue capacities,
timeouts, supported sample formats, callback constraints, and ownership.

## Verification coverage and next steps

The original eight core tests cover project and tempo validation, silence across different
channel counts and consecutive block sizes, identical results across different
block partitions, transport behavior, invalid configuration and partial frames,
counter overflow, and independence between document edits and playback state.
Service and CLI tests additionally cover accepted/applied state, queue overflow,
FIFO processing limits, telemetry recovery, concurrent handles, shutdown/restart,
owner cleanup, simulated stream failure, expired requests, argument errors, and
multiple smoke block sizes. Native-feature tests also check unsigned-format silence.

The hardware lifecycle test is ignored by default. To explicitly check missing-device
recovery, transport acknowledgements, and reopening on the same service:

```powershell
cargo test -p daw-audio --features native-audio --locked --test service_contract native_failure_recovery_and_restart_on_same_service -- --ignored
```

The render path has been inspected for allocations and blocking operations.
Allocator instrumentation and real-time deadline measurements have not been
performed. Native smoke validates lifecycle and callbacks, not audible continuity
or absence of dropouts. Physical device removal and other output formats have
not been tested on hardware.

Stage 4 will add IPC commands and generated TypeScript DTOs, connect the UI to
audio initialization and transport, and publish observed engine state. CI and
the complete integration checks remain scheduled for stages 5 and 6.

Output currently contains silence only. There is no synthesizer, sampler,
sequencer, project persistence, or WAV export; an oscillator is a later milestone.
