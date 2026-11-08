# AudioMOTORR / Rust DAW

A desktop music application built with Rust, Tauri 2, React, TypeScript, and Vite.
The project is building a foundation for a four-channel, 16-step groovebox that
can later evolve into a clip-based DAW.

## Current status

Stages 1–8 and increments 9.1–9.2 are complete. The engine library now includes
immutable PCM resources, a fixed 16-slot bank, and 16 one-shot sampler voices.
Stage 9 remains in progress: file loading is next (9.3), followed by native
sampler transport/service/UI integration (9.4) and final verification (9.5).

Desktop opens audio on an explicit click, offers silence or a sine source with
5 ms gain/transport ramps, controls BPM, and displays observed state. Browser
mode disables native operations. Sampler playback currently belongs to the
engine library; desktop has no sample-loading or sampler controls yet.

See the [current kernel report](progress/etapa-9-voces.md),
[implementation log](progress/detalles.md), [handoff](progress/continuidad-etapa-6.md),
and dated [stage 6 requirements audit](progress/cierre-etapa-6.md).
The progress documents are in Spanish; this README and technical docs remain in
English. Git history identifies published revisions; the
[Actions page](https://github.com/YoZaiKK/AudioMOTORR/actions) reports hosted checks.
Local verification alone does not establish remote CI success.

Technical documentation:

- [Architecture and control/observation diagrams](docs/architecture.md).
- [Realtime contract](docs/realtime-contract.md): ownership, limits, overflow,
  applied state, formats, errors, and restart.
- [ADR 0001: core and desktop boundaries](docs/adr/0001-core-and-desktop-boundaries.md).
- [Development, platform evidence, and CI](docs/development.md).
- [Ordered roadmap and milestone acceptance](docs/roadmap.md).
- [Offline oscillator and WAV contract](docs/offline-oscillator.md).
- [Native audition, gain ramps, and measurement scope](docs/native-audition.md).
- [Sampler PCM resources, ownership, voice policy, and fixtures](docs/sampler-contract.md).

Stage 7 is complete (2026-11-07): the opt-in oscillator, bounded application render,
and CLI float32 WAV export are published in `c6cf627` and passed
[Linux/Windows CI](https://github.com/YoZaiKK/AudioMOTORR/actions/runs/42419963899).
See the [closure report](progress/cierre-etapa-7.md) and
[Spanish execution plan](progress/plan-etapas-7-13.md). Stages 8–13 cover native signal/ramps, sampler,
mixer, sequencer, persistence/beat export, and clips/scenes, in that order.
Stage 8 is complete and published in `1e6bca8`, with
[Linux/Windows CI success](https://github.com/YoZaiKK/AudioMOTORR/actions/runs/42524752652).
The sampler kernel is published in `7acc1db` and passed
[Linux/Windows CI](https://github.com/YoZaiKK/AudioMOTORR/actions/runs/42530699397).
It renders exact mono/stereo one-shots; local debug/release tests measured zero
allocations and deallocations during processing. See the
[contract](docs/sampler-contract.md) and [kernel report](progress/etapa-9-voces.md).
File loading, AudioEngine/service
integration, and sampler UI remain future work; stages 10–13 remain planned.
Desktop defaults to
silence and offers an explicit sine source, frequency, and volume control.
Ordinary smoke commands retain silence; export-wav writes a sine to disk without hardware.
The [export report](progress/etapa-7-exportacion.md) records evidence and limits;
the [oscillator report](progress/etapa-7-oscilador.md) preserves the prior increment.

See the [Spanish practical guides](howToRunThis/README.md) for setup, feature
walkthroughs, tests, troubleshooting, nine executable CLI scenarios, and seven
real UI screenshots. Run the scenarios with `node howToRunThis/run-cli-cases.mjs`.
They are test fixtures, not importable projects. The [verification report](howToRunThis/resultados.md)
records actual results and unchanged lockfile/binding hashes.

Implemented capabilities:

- A minimal project document with a validated ID, title, and tempo.
- Silence rendering into caller-owned, interleaved `f32` buffers.
- Optional engine-library sine rendering with validated frequency/amplitude,
  continuous phase across blocks, and defined Play/Pause/Stop behavior.
- Bounded mono/stereo offline rendering and exact-frame float32 WAV export from CLI.
- Play, Pause, Stop, and tempo changes between processing blocks.
- Separate device-frame and transport-position counters.
- Small, copyable engine snapshots and explicit configuration/render errors.
- Validated immutable mono/stereo PCM resources for the sampler kernel, with
  frame limits and explicit output compatibility checks.
- A fixed 16-slot bank and 16 one-shot voices in the engine library: deterministic
  gain/summing, reject-newest exhaustion, and retained PCM after completion/reset.
- A bounded control thread, realtime SPSC queues, and accepted/applied command IDs.
- Device enumeration, supported output configurations, and explicit device selection.
- Audio initialization, shutdown, failure reporting, and restart.
- CLI smoke checks and behavioral tests for rendering, queues, and service lifecycle.
- Eleven IPC commands with Rust DTOs and generated TypeScript declarations.
- Audio-device selection, initialization/shutdown, Play/Pause/Stop, and BPM controls.
- Explicit native sine audition, volume control, and per-frame 5 ms attack/release ramps.
- State published below 15 Hz, with pending commands separated from applied state.

Verified on Windows: reproducible npm installation, prerequisite diagnostics,
frontend checks and build, core tests, workspace Clippy, and a debug desktop build
without an installer. Native CLI checks opened the Realtek WASAPI output at
48 kHz, stereo, `f32`, confirmed callbacks and applied Play/Pause/Stop, closed it,
and reopened it by explicit device ID. These results are recorded in the
implementation log. A real WebView2 UI smoke also confirmed initialization,
transport, 137.5 BPM, shutdown/restart, and normal window exit. Its screenshot is
saved locally at `target/desktop-stage4.png`. Hosted Ubuntu headless checks passed
for baseline `1ef9bd0`; native Linux and all macOS builds remain unverified.

Project tracking documents are currently maintained in Spanish:

- [Implementation checklist and musical stages](progress/checklist.md).
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

This checks generated bindings, TypeScript, ESLint, frontend tests, Prettier, and
the frontend build, then runs nine offline CLI scenarios, Rust formatting,
headless tests/Clippy, and workspace tests/Clippy with warnings treated as errors.
It stops at the first failure and requires desktop prerequisites for workspace
checks. The desktop package enables native audio; automated tests do not open hardware.

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

## Desktop IPC and bindings

DTOs live in `src-tauri/src/ipc/dto.rs`. Serde defines the camelCase wire contract
and ts-rs **12.0.1** generates `src/shared/api/generated/api.ts`:

```powershell
npm run bindings:generate
npm run bindings:check
npm run test:frontend
```

The check compares generated text without rewriting files. Generated TypeScript
is excluded from Prettier so regeneration stays deterministic. Frames, callback
counts, sequences, and IDs cross JSON as decimal strings; the UI compares them
with `BigInt`. Components use domain hooks and a central `desktop-client.ts` rather
than invoking Tauri directly. They register snapshot listeners before querying
initial state, reject old sequences, and dispose asynchronous subscriptions.

Eleven application commands are registered through `ipc::configure`; build.rs
generates their ACL permissions, granted only to the local `main` window along
with event listen/unlisten. Generated permissions are build artifacts ignored by
Git. Production CSP stays explicit; no remote capability or plugin is enabled.
Application calls run on blocking workers, while a separate publisher sends
observed snapshots at intervals of at least 67 ms. Closing main stops and joins
the publisher and closes the audio service. The engine callback performs no IPC.

On Windows/MSVC, integration-test executables receive a Common Controls v6
manifest for Tauri's imported APIs. Tauri builds the normal binary manifests.
Mock-runtime tests may need access to the application's data directories outside
a restricted filesystem sandbox; this does not mean an audio device is opened.

To run the optional real desktop smoke after a debug desktop build:

```powershell
powershell -NoProfile -File scripts/desktop-smoke.ps1
```

Run `npm run check` first, then build desktop and run this smoke. Workspace tests
may rebuild the main executable for development; rebuild through the Tauri CLI
before the smoke if Cargo commands ran after the desktop build.

This Windows-only check opens the built app, uses WebView2's local debugging port
9223 to exercise the real React UI and audio service, captures a screenshot in
`target/desktop-stage4.png`, shuts audio down, and closes the native window. It
needs real hardware and is not part of `npm run check`. Browser-debugging arguments
are scoped to the launched process and restored afterward; no global settings are
changed. Use an available local port and close other copies of the app first.

The real desktop smoke verifies a structured missing-device error in the UI,
recovery, and callbacks after restarting audio. Historical counts and hardware
evidence are preserved in the [stage 5 report](howToRunThis/resultados.md);
the latest complete local checks are recorded in the implementation log.

## Audio CLI commands

Export a low-amplitude 440 Hz sine to a new WAV file, without opening a device:

```powershell
cargo run -p daw-cli --no-default-features --locked -- export-wav --output target/tone.wav --frames 48001 --block-size 127
```

The parent directory must exist and the filename must be new. Defaults are
48000 Hz, stereo, peak amplitude 0.1, and block size 256. Options also include
`--sample-rate`, `--channels` (1 or 2), `--frequency`, and `--amplitude`.
The command validates parameters/container limits before creation, preserves
existing files, and reports success only after finalization. A later I/O failure
can leave a partial new file. See the [contract](docs/offline-oscillator.md) and
[Spanish CLI guide](howToRunThis/03-cli.md) for exact limits and examples.

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
| `src/`               | React panels, domain hooks, central IPC client, and generated API types.      |
| `src-tauri/`         | Tauri composition, IPC DTOs/commands, ACLs, and snapshot publisher.           |
| `crates/daw-domain/` | Minimal project document, IDs, and validated tempo.                           |
| `crates/daw-engine/` | Silence/sine rendering, gain ramps, transport, snapshots, and immutable PCM.  |
| `crates/daw-audio/`  | Audio service, bounded queues, manual rendering, optional CPAL backend.       |
| `crates/daw-app/`    | Project coordination and audio/transport use cases shared by desktop and CLI. |
| `crates/daw-cli/`    | Offline smoke, WAV export, device/native checks, and release diagnostics.     |
| `docs/`              | Architecture, realtime contract, ADR, development/CI, and roadmap.            |
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
`f32` sample per channel in interleaved order. The default engine overwrites every sample
with zero and accepts variable block sizes, including an empty buffer. It does
not allocate memory, acquire locks, perform I/O, or log during rendering.

Buffers must contain complete frames. A misaligned buffer or counter overflow
returns `RenderError`, silences the entire buffer, and leaves the engine state
unchanged. Counter increments use checked arithmetic.

`AudioEngine::with_sine(config, tempo, frequency_hz, amplitude)` explicitly
constructs a sine-rendering engine. Frequency must be finite, positive, and below
half the sample rate; peak amplitude must be finite and within 0–1 inclusive.
Invalid values return `OscillatorError`. Phase advances once per playing frame,
is shared across channels, survives Pause, and resets on Stop. BPM does not retune
the signal. See the [oscillator contract and API example](docs/offline-oscillator.md).
The service can explicitly initialize the same oscillator with a ramped envelope;
ordinary smoke commands retain silence. See the [native contract](docs/native-audition.md).

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

See the [realtime contract](docs/realtime-contract.md) for queue capacities,
timeouts, supported sample formats, callback constraints, and ownership.

## Verification coverage and next steps

Core tests cover project and tempo validation, silence across different
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

Optional release instrumentation has measured the processing path on the local
Realtek WASAPI device with zero allocations and zero nominal processing-budget
overruns in the recorded short run. These results and their limits are in the
[stage 8 report](progress/etapa-8.md). Physical loopback continuity, device removal,
and other physical output formats remain unverified.

Frontend tests cover initialization errors, pending/applied commands, events that
arrive before invoke replies, stale observations, browser mode, and listener cleanup
under StrictMode. Desktop tests exercise the real registered handler with Tauri's
mock runtime, decimal-string counters beyond JavaScript's safe integer range,
window ACLs, publication rate, and publisher shutdown. Mock-runtime tests do not
replace the separate real WebView2/hardware smoke.

Stage 5 passed 12 frontend tests, 25 headless Rust tests, 29 workspace Rust tests,
and nine CLI scenarios, plus Clippy and the real desktop walkthrough. These suites
overlap. Installation and binding regeneration preserved both lockfiles and generated
types. Use `powershell -NoProfile -File scripts/desktop-smoke.ps1 -UpdateDocs` after
a fresh desktop build to explicitly update documented screenshots and metadata.
Normal checks neither open hardware nor overwrite those images.

Stage 6 adds [GitHub Actions](.github/workflows/ci.yml): explicit headless checks
on Ubuntu 24.04 and frontend/mock IPC/desktop checks on Windows Server 2022.
Official actions use full SHA pins; lockfiles and generated types are checked for
changes. No hardware, screenshot updates, signing, or publishing run in CI.
The workflow is validated locally; inspect its
[hosted runs](https://github.com/YoZaiKK/AudioMOTORR/actions/workflows/ci.yml)
for remote results. Local results are in the [stage 6 closure](progress/cierre-etapa-6.md)
and [implementation log](progress/detalles.md).

Stage 7 is closed with local checks and hosted Linux/Windows success for the
implementation commit `c6cf627`. The [closure report](progress/cierre-etapa-7.md)
maps acceptance to code/tests and records the exact SHA, run, and job links.
The [oscillator](progress/etapa-7-oscilador.md) and
[export](progress/etapa-7-exportacion.md) reports preserve earlier local evidence.
Stage 8 adds native audition and ramps. Select Sine in desktop before initializing,
then Play; volume is 0–1 on a fixed 0.1-peak voice. Pause/Stop release over 5 ms;
snapshots distinguish requested and actual transport until the release completes.
Run the optional real UI check after check and desktop build:

```powershell
powershell -NoProfile -File scripts/desktop-smoke.ps1 -Sine
cargo run -p daw-cli --release --features rt-diagnostics --locked -- tone-check --seconds 2
cargo run -p daw-cli --release --features rt-diagnostics --locked -- engine-bench --block-size 127 --iterations 20000
```

The sine smoke writes target/desktop-stage8.png and preserves the seven historical
screenshots. Release diagnostics are opt-in and excluded from default desktop
builds; hardware checks never run in CI. See the [contract](docs/native-audition.md)
for measurement scope and the [plan](progress/plan-etapas-7-13.md) for stage 9.
Stages 9.1–9.2 implement immutable resources, a fixed bank, and raw voice playback
in the engine library. Next is 9.3: bounded loading/decoding outside processing.
Desktop sampler audition, sequencer, and project persistence are not implemented yet.
