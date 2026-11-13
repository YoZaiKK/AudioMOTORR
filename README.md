# AudioMOTORR / Rust DAW

A desktop music application built with Rust, Tauri 2, React, TypeScript, and Vite.
The project is building a foundation for a four-channel, 16-step groovebox that
can later evolve into a clip-based DAW.

## Current status

Updated 2026-11-12. **Stages 1–10 are complete within their recorded scope.**
The sampler includes WAV preparation, engine/service/desktop integration and
diagnostics. Six native matrix cases pass on Realtek/WASAPI; the user confirmed
audible headphone output after a repeat. Source/evidence audit and exact-revision
CI are verified; see [stage 9 closure](progress/cierre-etapa-9.md).
The [four-channel mixer core (10.2)](progress/etapa-10-mezclador-motor.md) is
implemented locally: document settings, opt-in engine mixing, gain/pan/mute,
ramps and per-render meters. [Audio service integration (10.3.a)](progress/etapa-10-sesiones-mezclador.md)
adds mixed manual/native sessions, queued controls and complete-callback peaks.
[Application coordination (10.3.b)](progress/etapa-10-aplicacion-mezclador.md) now
installs current document settings and distinguishes document-only edits, queue
acceptance and applied state. [Desktop integration (10.3.c)](progress/etapa-10-desktop-mezclador.md)
adds generated DTOs/IPC/ACL, four control strips, 16 slot routes and measured
channel/master levels. [Integrated acceptance (10.3.d)](progress/cierre-integracion-mezclador-10.3d.md)
adds decoded four-channel references, lifecycle coverage and two UI corrections.
[Mixed diagnostics (10.4.a)](progress/etapa-10-diagnosticos-mezclador.md) now add
explicit benchmark/native commands and a native verification protocol. Offline
measurements pass. [Native measurements (10.4.b)](progress/etapa-10-medicion-nativa-mezclador.md)
now record six successful mixed cases and six reopens on Speakers Realtek.
The [local delivery audit (10.4.c)](progress/auditoria-cierre-mezclador-10.4c.md)
verifies source/artifact integrity and prepares a [reviewable delivery](progress/entrega-mezclador-etapa-10.md).
[Delivery and CI](progress/publicacion-mezclador-10.4c.md) are verified for
implementation `e7f543f`: both Linux and Windows jobs pass. The user confirmed
‘Se escucha’ after the requested mono/stereo replay; [stage 10 is closed](progress/cierre-etapa-10.md)
within its recorded limits. **11.1 is implemented locally:** the
[sequencer contract](docs/sequencer-contract.md) and editable four-row, 16-step
domain pattern, with ordered one-shot events and independent project state.
See the [11.1 report](progress/etapa-11-modelo-patron.md).
**11.2 is implemented locally:** explicit sequenced mixed playback in the engine,
exact fractional phase, within-block scheduling, transport and live edits;
[engine report](progress/etapa-11-planificador.md).
**11.3.a is implemented locally:** [sequenced audio sessions](docs/sequencer-service.md),
bounded pattern commands and observed state across whole callbacks;
[service report](progress/etapa-11-sesiones-secuenciador.md).
11.1–11.2 are committed in `1effd48`; 11.3.a is delivered in `2d4cc28`.
**11.3.b is implemented locally:** [application pattern coordination](docs/sequencer-application.md),
selected versus accepted/applied edits and current settings at prepared-bank installation;
[application report](progress/etapa-11-aplicacion-secuenciador.md).
**11.3.c is implemented locally:** [desktop grid and IPC](docs/sequencer-desktop.md),
64 step buttons, row slots, explicit sequenced initialization and observed position.
See the [walkthrough](howToRunThis/08-secuenciador.md) and
[real WebView2 capture](docs/screenshots/stage11-sequencer-wide.png).
**11.3.d is accepted locally:** [decoded beat and integration coverage](docs/sequencer-integration-acceptance.md),
with 42 reference combinations and current check/release/build results.
**11.4.a is implemented locally:** [sequencer diagnostics](docs/sequencer-diagnostics.md),
72 offline measurements and a prepared native runner/fixture/protocol.
**11.4.b processing is measured:** six native cases and six reopens on Realtek/WASAPI;
[listening remains pending](progress/etapa-11-medicion-nativa-secuenciador.md).
**Stage 12 is implemented locally:** [portable project files and beat WAV export](docs/project-format.md),
with [desktop usage](howToRunThis/09-proyectos.md), CLI and real WebView2 verification.
**Stage 12 is delivered and closed within scope:** [`6c8c52e`](https://github.com/YoZaiKK/AudioMOTORR/commit/6c8c52e820081249a62ab5b664f94853568a4af6),
with [both CI jobs successful](https://github.com/YoZaiKK/AudioMOTORR/actions/runs/43348424541).
See the [closure](progress/cierre-etapa-12.md). Stage 11 listening remains pending;
clips/scenes are next in stage 13.

Desktop explicitly loads/cancels WAV banks, initializes silence/sine/sampler
sessions, controls transport/tempo/gain and shows exact-ID trigger outcomes or
Unknown for lost observations. The sampler has 16 slots/voices, fixed 1/16
headroom and 5 ms master ramps. Browser mode disables native operations.
See the [sampler walkthrough](howToRunThis/06-sampler.md).

sampler-bench provides device-free mono/stereo CPU measurements with
offline-diagnostics, without CPAL. sampler-native-check is implemented and built
in release and now measured on physical Realtek hardware with mono/stereo inputs
and 0/1/16 voices; see the [native report](progress/etapa-9-medicion-nativa.md). Existing
engine-bench/tone-check exercise the sine. The CLI exports a sine WAV; exporting
a sampler project or a beat is not implemented.

The sampler baseline is published in [4dc2c6c](https://github.com/YoZaiKK/AudioMOTORR/commit/4dc2c6c2e0882b8f131b68088feffa8def84ba83),
including the CLI trace patch and stereo fixture helper. On 2026-11-11, GitHub's
API confirmed remote main at this revision and both Linux/Windows jobs successful
in [run 42896571891](https://github.com/YoZaiKK/AudioMOTORR/actions/runs/42896571891).
The 120-file stage 9 worktree fingerprint matched at its closure audit, with four
Git/worktree differences limited to line endings. Mixer 10.2 adds new local source
changes and its own manifest; 10.3.a records a newer service snapshot in its
[manifest](progress/evidencia-sesiones-mezclador-10.3a.json). Stage 9 reports retain their original Git/CI state.
The mixer implementation is delivered in `e7f543f` with its own successful
[Linux/Windows CI](https://github.com/YoZaiKK/AudioMOTORR/actions/runs/43147510037).
The earlier manifests preserve their capture-time Git state. The
[9.5.a report](progress/etapa-9-diagnosticos-sampler.md) records 108 headless,
114 workspace, 29 instrumented-audio, nine diagnostic-CLI, eight decoder-memory,
24 frontend tests and nine CLI scenarios, plus Windows debug build. Suites overlap.
Its [119-file fingerprint](progress/evidencia-diagnosticos-9.5a.json) remains a
dated snapshot. The [native evidence](progress/evidencia-nativa-9.5b.json) records
the measured runner, inputs, exact-ID outcomes and current source snapshot.
The [10.2 report](progress/etapa-10-mezclador-motor.md) separates new mixer
verification from that historical sampler evidence.

See the [configuration review](progress/revision-readme-configuracion.md),
[status/objectives audit](progress/estado-y-siguiente-etapa.md),
[implementation log](progress/detalles.md) and [handoff](progress/continuidad-etapa-6.md).
README/technical documents are in English; progress and practical guides are in Spanish.

## Next steps

| Stage                        | Next deliverable                                                                           | Acceptance / boundary                                                                                                                                                    |
| ---------------------------- | ------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| 9 — closed                   | Source/evidence audit, native measurements, user-confirmed audible output and CI recorded. | Both jobs successful for 4dc2c6c; acoustic continuity, unplug/stress and platform limits remain explicit.                                                                |
| 10.1–10.2 — delivered core   | Four-channel document settings and opt-in deterministic engine mixing implemented.         | [Contract](docs/mixer-contract.md), reference samples, partition invariance and zero processing allocation/free checks.                                                  |
| 10.3.a — delivered           | Mixed sessions, shared FIFO controls and complete-callback meter windows implemented.      | [Service contract](docs/mixer-service.md), adapter/lifecycle fixtures and processing memory checks.                                                                      |
| 10.3.b — delivered           | Application selection/acceptance and prepared-bank installation coordinated.               | [Application contract](docs/mixer-application.md), rollback and lifecycle fixtures.                                                                                      |
| 10.3.c — delivered           | Selected settings, receipts, applied observations and levels exposed through four strips.  | Generated bindings, ACL and application/IPC/UI ordering tests.                                                                                                           |
| 10.3.d — delivered           | Integrated references, lifecycle and UI query ordering verified locally.                   | End-to-end reference cases, UI/IPC ordering/lifecycle, coherent check/build evidence.                                                                                    |
| 10.4.a — delivered           | Mixed diagnostics and native protocol implemented; offline evidence recorded.              | Release gates, 24 offline benchmark cases and a compiled native runner; hardware remains 10.4.b.                                                                         |
| **10.4.b–c — closed**        | Native measurements, delivery/CI and user-confirmed audible mixer output recorded.         | Explicit mixed runner, own hardware evidence, real listening and CI of delivered revision.                                                                               |
| 11.1 — implemented locally   | Domain pattern and sequencer contract.                                                     | Fixed 4 × 16 state, validated slots, ordered events and independent project copies; [report](progress/etapa-11-modelo-patron.md).                                        |
| 11.2 — implemented locally   | Engine scheduling and sequenced mixed rendering.                                           | Exact fractional phase, independent event/PCM references, transport/edits and processing memory checks; [report](progress/etapa-11-planificador.md).                     |
| 11.3.a — implemented locally | Sequenced manual/native sessions and bounded pattern commands.                             | Applied phase/outcomes, FIFO budget/loss/restart, native chunk/conversion references and processing memory checks; [report](progress/etapa-11-sesiones-secuenciador.md). |
| 11.3.b — implemented locally | Selected document pattern, queue acceptance and applied state coordinated.                 | Current pattern/mix/tempo at installation, shared FIFO rollback/lifecycle and real-WAV fixtures; [report](progress/etapa-11-aplicacion-secuenciador.md).                 |
| 11.3.c — implemented locally | Desktop grid, generated DTOs and scoped IPC.                                               | Exact strings, receipt/order/loss tests and real WebView2 editing/layout; [report](progress/etapa-11-desktop-secuenciador.md).                                           |
| 11.3.d — accepted locally    | Decoded beat reference and integrated acceptance.                                          | Two new cases, 42 combinations, lifecycle/UI/IPC audit, check/release/build; [report](progress/cierre-integracion-secuenciador-11.3d.md).                                |
| 11.4.a — implemented locally | Explicit sequencer diagnostics and native protocol.                                        | 72 offline measurements, independent reference/memory checks and device-free lifecycle tests; [report](progress/etapa-11-diagnosticos-secuenciador.md).                  |
| 11.4.b–c                     | Native sequencer verification and delivery closure.                                        | Physical measurements/listening, evidence audit and exact-revision CI.                                                                                                   |
| 12 — completed within scope  | Portable projects, explicit preparation and full beat export.                              | Versioned files, relative asset copies, exact beat/tails and desktop/CLI; delivered in 6c8c52e with both CI jobs successful; [closure](progress/cierre-etapa-12.md).     |
| 13                           | Clips and scenes.                                                                          | Clip/scene model, quantized launch and predictable transitions.                                                                                                          |

The [roadmap](docs/roadmap.md) and [Spanish execution plan](progress/plan-etapas-7-13.md)
define scope and dependencies. The [execution plan](progress/plan-siguientes-pasos.md)
breaks down the remaining deliveries. Stage 9 closure combines implementation,
measurements, audit/CI and user-confirmed audibility; it does not establish
acoustic continuity. This documentation update created no commit/push.

Technical documentation:

- [Mixed sampler diagnostics](docs/mixer-diagnostics.md) and [native verification protocol](docs/mixer-native-verification.md).

- [Integrated mixer acceptance and verification boundaries](docs/mixer-integration-acceptance.md).

- [Desktop mixer controls, routing and measured levels](docs/mixer-desktop.md).
- [Practical mixer guide (Spanish)](howToRunThis/07-mezclador.md).

- [Architecture and control/observation diagrams](docs/architecture.md).
- [Realtime contract](docs/realtime-contract.md): ownership, limits, overflow,
  applied state, formats, errors, and restart.
- [ADR 0001: core and desktop boundaries](docs/adr/0001-core-and-desktop-boundaries.md).
- [Development, platform evidence, and CI](docs/development.md).
- [Ordered roadmap and milestone acceptance](docs/roadmap.md).
- [Offline oscillator and WAV contract](docs/offline-oscillator.md).
- [Native audition, gain ramps, and measurement scope](docs/native-audition.md).
- [Sampler PCM resources, ownership, voice policy, and fixtures](docs/sampler-contract.md).
- [Sampler sessions, trigger outcomes and stream-owner teardown](docs/sampler-service.md).
- [Sampler application, DTO/IPC, preparation lifecycle and desktop controls](docs/sampler-desktop.md).
- [Integrated sampler acceptance matrix and local closure](docs/sampler-integration-acceptance.md).
- [Sampler benchmark and opt-in native diagnostic runner](docs/sampler-diagnostics.md).
- [Native sampler verification protocol](docs/sampler-native-verification.md).
- [Specified WAV loading formats, memory budgets, errors, and cancellation](docs/sample-loading-contract.md).
- [WAV preparation acceptance matrix and verification limits](docs/sample-loading-acceptance.md).

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
The application library now prepares complete banks from WAV with fault/cleanup
and end-to-end playback fixtures. AudioEngine integration is implemented offline;
service integration is implemented in the Rust audio library.
Sampler UI and four-channel mixing are implemented and delivered. Mixer
listening is confirmed and stage 10 is closed; sequencer, persistence and clips are stages 11–13. See the
[bank preparation report](progress/etapa-9-preparacion-banco.md) and
[fault/cleanup report](progress/etapa-9-fallos-cancelacion.md).
The [loaded-sampler report](progress/etapa-9-fixtures-integracion.md) records 9.3.e.
The [9.3 closure](progress/cierre-preparacion-9.3.md) and
[source/configuration fingerprint](progress/evidencia-preparacion-9.3.json) identify
the dated preparation scope and its original evidence boundary. The
[9.4.a report](progress/etapa-9-integracion-motor.md) records dated engine integration;
the historical 9.3 fingerprint does not match this newer engine. The
[9.4.a fingerprint](progress/evidencia-integracion-9.4a.json) identifies that dated
engine integration before inclusion in 0c346d5.
The [9.4.b report](progress/etapa-9-sesiones-sampler.md) records dated session
integration and its evidence; older fingerprints remain dated snapshots.
The [9.4.b fingerprint](progress/evidencia-sesiones-9.4b.json) identifies that dated
code/configuration. The [9.4.c report](progress/etapa-9-aplicacion-sampler.md) records
that desktop integration before its inclusion in Phase 9.5.a; its earlier publication/CI
status is historical. Hosted CI for 0c346d5 remains uninspected in this review.
The [9.4.c fingerprint](progress/evidencia-desktop-9.4c.json) identifies its dated
116-file state. The [9.4.d closure](progress/cierre-integracion-9.4.md) records the
audit, UI fixes and new local verification separately from pending native measurements/CI.
The [9.4.d fingerprint](progress/evidencia-integracion-9.4d.json) identifies the
dated 116-file state; full check and Windows debug build passed, with 24 frontend tests.
The [9.5.a report](progress/etapa-9-diagnosticos-sampler.md) records newer sampler
diagnostics and device-free measurements before the native runs. The
[9.5.b report](progress/etapa-9-medicion-nativa.md) records the later Realtek
measurements; hosted CI for the current worktree remains unverified.
The [9.5.a fingerprint](progress/evidencia-diagnosticos-9.5a.json) identifies the
dated 119-file state. Full check and Windows debug build passed locally; release
benchmarks report zero allocations/frees in processing across 24 mono/stereo cases.
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
- AudioEngine sampler mode with bank ownership, fixed headroom, 5 ms transport/gain
  ramps and typed direct trigger results, verified offline without an audio device.
- Manual/CPAL sampler sessions and shared-FIFO triggers with exact-ID Copy outcomes,
  cumulative counters and owner-side resource reclamation after stream closure.
- Individual PCM16/float32 WAV loading in daw-app, with bounded preflight,
  exact PCM reservation, typed errors and cooperative cancellation outside audio processing.
- Complete-bank preparation in daw-app: validated slot selections, sequential
  decoding, one shared preparation/pending-result permit, and aggregate payload budgets.
- A bounded control thread, realtime SPSC queues, and accepted/applied command IDs.
- Device enumeration, supported output configurations, and explicit device selection.
- Audio initialization, shutdown, failure reporting, and restart.
- CLI smoke checks and behavioral tests for rendering, queues, and service lifecycle.
- Twenty IPC commands with Rust DTOs and generated TypeScript declarations.
- Worker-based WAV loading/cancellation and explicit sampler initialization and
  triggers, with exact-ID Started/rejected/Unknown UI observations.
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

## Repository configuration

| Area                   | Configuration and reviewed settings                                                                                                                                                                                                                                       |
| ---------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Toolchain/dependencies | [rust-toolchain.toml](rust-toolchain.toml): Rust 1.90.0, rustfmt/Clippy; [.node-version](.node-version): Node 22.23.4; [package.json](package.json): npm 10.9.9. [.npmrc](.npmrc) enables strict engine checks and exact dependency saves. Keep both lockfiles committed. |
| Rust workspace         | [Cargo.toml](Cargo.toml): six members, five headless defaults, edition 2024, unsafe forbidden. Native audio/diagnostics remain opt-in; desktop enables native audio.                                                                                                      |
| Frontend               | [vite.config.ts](vite.config.ts): 127.0.0.1:1420, strict port; [tsconfig.json](tsconfig.json): strict/noEmit; [vitest.config.ts](vitest.config.ts): jsdom/frontend tests.                                                                                                 |
| Desktop                | [tauri.conf.json](src-tauri/tauri.conf.json) uses the same dev URL and root dist output. CSP and [capabilities](src-tauri/capabilities/default.json) restrict production access to local main-window commands; 20 application commands plus event listen/unlisten.        |
| CI                     | [Workflow](.github/workflows/ci.yml): pinned toolchains, SHA-pinned official actions, explicit headless packages on Ubuntu 24.04, full check/debug desktop on Windows 2022, no hardware/publishing steps.                                                                 |
| Generated/local files  | [.gitignore](.gitignore) excludes dependencies, build output, Tauri generated permissions/schemas, coverage, logs, local env values and TypeScript build cache. [.prettierignore](.prettierignore) and [ESLint](eslint.config.js) exclude generated coverage.             |

Real .env/.env.* files stay local; .env.example and .env.*.example templates remain
eligible for version control. Cargo.lock, package-lock.json, generated TypeScript
bindings, source, documentation and historical screenshots remain tracked.
Formatter exclusions for bindings preserve deterministic generation; they do not
mean those files should be ignored by Git. Store new measurements/temporary WAVs
under target; keep dated reports and approved assets in their documented locations.

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

Twenty application commands are registered through `ipc::configure`; build.rs
generates their ACL permissions, granted only to the local `main` window along
with event listen/unlisten. Generated permissions are build artifacts ignored by
Git. Production CSP stays explicit; no remote capability or plugin is enabled.
Application calls run on blocking workers, while a separate publisher sends
observed snapshots at intervals of at least 67 ms. Closing main stops and joins
the publisher and closes the audio service. The engine callback performs no IPC.
Sampler preparation uses its own worker and 200 ms status polling outside audio;
exit cancels and joins that worker. See the [desktop contract](docs/sampler-desktop.md).

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

| Path                 | Responsibility                                                                  |
| -------------------- | ------------------------------------------------------------------------------- |
| `src/`               | React panels, domain hooks, central IPC client, and generated API types.        |
| `src-tauri/`         | Tauri composition, IPC DTOs/commands, ACLs, and snapshot publisher.             |
| `crates/daw-domain/` | Project document, IDs, validated tempo and fixed mixer settings.                |
| `crates/daw-engine/` | Silence/sine/sampler rendering, optional four-channel mixing, ramps and meters. |
| `crates/daw-audio/`  | Audio service, bounded queues, manual rendering, optional CPAL backend.         |
| `crates/daw-app/`    | Project coordination and audio/transport use cases shared by desktop and CLI.   |
| `crates/daw-cli/`    | Offline smoke, WAV export, device/native checks, and release diagnostics.       |
| `docs/`              | Architecture, realtime contract, ADR, development/CI, and roadmap.              |
| `scripts/`           | Windows diagnostics and verification orchestration.                             |
| `progress/`          | Requirements, implementation checklist, and verification log.                   |

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

The document also owns validated four-channel `MixerSettings`: gain/pan/mute and
16 slot routes, initially `slot % 4`. Copying them into a new
`AudioEngine::with_mixed_sampler` explicitly installs the mixed mode; editing the
document later does not implicitly change processing. Legacy sampler mode stays available.

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
in the engine library. 9.3.b adds bounded reading/decoding of individual WAV assets
outside processing. 9.3.c adds complete-bank preparation and aggregate budgets;
9.3.d verifies faults/cancellation/cleanup and 9.3.e verifies end-to-end loading
and offline playback. Preparation 9.3 is closed locally and 9.4.a integrates the
sampler into AudioEngine. 9.4.b adds audio sessions/queued outcomes; 9.4.c adds
application/DTO/IPC and desktop controls. 9.4.d closes integration locally;
9.5.b records six native cases using the [sampler diagnostics](docs/sampler-diagnostics.md).
The [9.5.c audit](progress/auditoria-cierre-9.5c.md) verifies evidence and exact-revision CI.
The user confirmed audible headphone output; [stage 9 closure](progress/cierre-etapa-9.md)
records that observation under the [native protocol](docs/sampler-native-verification.md).
Sequencer and project persistence are now implemented; see the current
[project format/export contract](docs/project-format.md). Clips/scenes remain stage 13.
