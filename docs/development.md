# Development, platforms, and CI

Stage 10.2 adds the [local mixer core](../progress/etapa-10-mezclador-motor.md):
domain/engine fixtures and an instrumented audio-crate memory contract, discovered
by the existing check workflow. Full local check and release verification are
recorded separately from stage 9 CI; no new hosted mixer CI or native audition
is implied. Stage 10.3.a adds [mixed audio sessions](mixer-service.md), queued
controls and complete-callback peaks, with adapter/lifecycle tests.
[Application coordination 10.3.b](mixer-application.md) now adds real-WAV and
selection/acceptance/error fixtures. [Desktop integration 10.3.c](mixer-desktop.md)
adds mock-runtime IPC/ACL and UI ordering/lifecycle/level fixtures, with full
check and debug desktop build recorded in its [report](../progress/etapa-10-desktop-mezclador.md); see the
[10.3.a report](../progress/etapa-10-sesiones-mezclador.md).

[Integrated mixer acceptance 10.3.d](mixer-integration-acceptance.md) adds three
real-WAV application cases (debug/release), three UI regressions and a documented
acceptance matrix. [Mixed diagnostics 10.4.a](mixer-diagnostics.md) add offline and native-feature
release CLI checks and 24 measured offline cases; the same native lifecycle is
tested with manual audio. Physical mixed playback/listening 10.4.b and its own
hosted CI/closure 10.4.c remain pending at that increment. The subsequent
[10.4.b measurement](../progress/etapa-10-medicion-nativa-mezclador.md) records six
real-device mixed cases/reopens on Speakers Realtek.
[implementation e7f543f delivery and exact-revision hosted CI](../progress/publicacion-mezclador-10.4c.md)
are now verified; mixer listening is confirmed and
[stage 10 is closed](../progress/cierre-etapa-10.md) within its recorded scope.

On 2026-11-11 the [9.5.c audit](../progress/auditoria-cierre-9.5c.md) verified
the recorded source/artifact hashes and inspected both successful jobs of
[run 42896571891](https://github.com/YoZaiKK/AudioMOTORR/actions/runs/42896571891)
for implementation `4dc2c6c`, including the Windows full check and desktop build.
The user subsequently confirmed audible headphone output after a repeated native
run; [stage 9 is closed](../progress/cierre-etapa-9.md) within its documented limits.
The increment reports below preserve their original Git/CI state.

Stage 9.5.b now records [six Realtek/WASAPI runs](../progress/etapa-9-medicion-nativa.md)
at 48 kHz stereo f32, with mono/stereo inputs and 0/1/16 voices. Initial/reopened
sessions report zero measured allocation/free/non-finite/processing overruns.
The runner prints exact-ID outcomes and reopened metrics outside processing;
[the fixture helper](../scripts/sampler-stereo-fixture.mjs) reproduces distinct
stereo input without overwriting. Audibility is recorded separately in the closure.
These short native measurements do not establish acoustic continuity by themselves.

Stage 9.5.a (2026-11-10) adds [sampler diagnostics](sampler-diagnostics.md):
offline-diagnostics avoids CPAL while rt-diagnostics retains native audio.
check now runs the diagnostic CLI unit/process tests with offline-diagnostics,
alongside existing instrumented audio and decoder tests. The
[report](../progress/etapa-9-diagnosticos-sampler.md) separates release CPU
measurements from the unperformed native hardware run. Default desktop has no
diagnostic instrumentation/IPC; dependencies, lockfiles and generated types stay fixed.

Stage 9.4.d closes the [integration acceptance audit](sampler-integration-acceptance.md)
locally. It fixes independent startup queries and automatic polling overlap in
React, with three reproduced regressions. The [closure report](../progress/cierre-integracion-9.4.md)
records verification; the [native protocol](sampler-native-verification.md) identifies
the sampler benchmark/runner delivered in 9.5.a. This adds no hardware measurements.

Stage 9.4.c (2026-11-09) adds [application/IPC/UI sampler integration](sampler-desktop.md).
Run the application worker/session and desktop mock contracts without opening hardware:

```powershell
cargo test -p daw-app --no-default-features --locked --lib --test sampler_application_contract
cargo test -p rust-daw-desktop --locked --test ipc_contract
npm run bindings:generate
npm run bindings:check
npm run test:frontend
npm run check
npm run desktop:build -- --debug --no-bundle
```

DTOs/generated TypeScript intentionally change in this increment; dependencies and
lockfiles do not. The [current report](../progress/etapa-9-aplicacion-sampler.md)
separates local check/build evidence from pending hosted CI and physical audition.
Preparation runs on an Application-owned worker outside its mutex. Cancel/poll do
not wait for a running decoder; exit cancels/joins it and cannot interrupt blocked
OS I/O. Read the [practical guide](../howToRunThis/06-sampler.md) for explicit controls.
The reports and manifests below retain their dated verification scope.

Stage 9.3 (2026-11-09) specifies and implements per-resource and complete-bank
[WAV loading](sample-loading-contract.md). The
[reader report](../progress/etapa-9-lectura-wav.md) and
[bank preparation report](../progress/etapa-9-preparacion-banco.md) record new local results
separately from older hosted CI. Complete-bank/permit/aggregate-budget enforcement
is implemented; 9.3.d adds deterministic injected fault/cancellation/publication
tests plus worker/receiving-thread allocator checks. The
[fault/cleanup report](../progress/etapa-9-fallos-cancelacion.md) records their
scope. 9.3.e adds end-to-end WAV/bank/offline-sampler fixtures, documented in the
[acceptance matrix](sample-loading-acceptance.md) and
[loaded-sampler report](../progress/etapa-9-fixtures-integracion.md).
Preparation is closed locally in 9.3.f; the
[closure audit](../progress/cierre-preparacion-9.3.md) and
[source/configuration manifest](../progress/evidencia-preparacion-9.3.json) identify
the evidence. This documentation-only closure reuses 9.3.e check/build results
and verifies that recorded code/configuration files retain their hashes.
The dated 9.3 manifest is retained as historical evidence; its source hashes no
longer match the newer engine. 9.4.a implements AudioEngine sampler integration
offline; see the [engine report](../progress/etapa-9-integracion-motor.md) and
[9.4.a source fingerprint](../progress/evidencia-integracion-9.4a.json).
9.4.b implements audio sessions and queued trigger outcomes; see the
[session contract](sampler-service.md) and [session report](../progress/etapa-9-sesiones-sampler.md).
The [9.4.b fingerprint](../progress/evidencia-sesiones-9.4b.json) records 112
source/configuration files with local check/build results.
The 9.4.a/9.4.b fingerprints are historical. Application/DTO/IPC/frontend are
implemented in 9.4.c; the [9.4.c manifest](../progress/evidencia-desktop-9.4c.json)
identifies that dated code. The 9.4.d closure records the newer hook/tests and
acceptance audit; 9.5.a delivers the diagnostic runner and 9.5.b now records
native processing measurements. The later [stage 9 closure](../progress/cierre-etapa-9.md)
records listening confirmation and exact-implementation CI.
The [current manifest](../progress/evidencia-integracion-9.4d.json) identifies
116 code/configuration files for 9.4.d. The newer
[9.5.a manifest](../progress/evidencia-diagnosticos-9.5a.json) identifies 119 files;
full check/Windows build and offline release diagnostics pass for that dated scope.
The newer [9.5.b report](../progress/etapa-9-medicion-nativa.md) records six native
Realtek runs, exact-ID traces and input/log hashes; its original listening-pending
state is retained as history and resolved by the later closure report.
The existing 9.2 results do not validate this new code.

```powershell
cargo test -p daw-audio --no-default-features --locked --test sampler_service_contract
cargo test -p daw-audio --release --features native-audio,rt-diagnostics --locked
cargo test -p daw-engine --no-default-features --locked --lib --test sampler_engine_contract
cargo test -p daw-engine --release --no-default-features --locked --lib --test sampler_engine_contract
cargo test -p daw-app --no-default-features --locked --test sample_loading_contract
cargo test -p daw-app --no-default-features --locked --lib --test bank_preparation_contract
cargo test -p daw-app --release --no-default-features --locked --lib --test bank_preparation_contract
cargo test -p daw-app --no-default-features --locked --test loaded_sampler_contract
cargo test -p daw-app --release --no-default-features --locked --test loaded_sampler_contract
cargo test -p daw-cli --no-default-features --locked --test export_contract
cargo test -p daw-cli --features rt-diagnostics --locked --test sample_loading_memory_contract
cargo test -p daw-cli --release --features rt-diagnostics --locked --test sample_loading_memory_contract
```

check now runs the CLI feature-only memory suite explicitly, without hardware;
Linux's explicit headless commands discover reader/bank/loaded-sampler/export tests and skip
feature-only memory tests. No workflow/dependency changes are required.
The app library unit suite includes the 17-case second-asset fault matrix and
channel-synchronized publication/blocked-read interleavings. The CLI diagnostic
suite now has eight tests, including allocation/free balance on identified
worker and receiving threads. These are offline tests; none opens hardware.

The repository pins Rust **1.90.0** with rustfmt/Clippy in
`rust-toolchain.toml`, Node **22.23.4** in `.node-version`, and npm **10.9.9**
in `package.json`. One root `Cargo.lock` and `package-lock.json` are authoritative.
Use locked installs/builds; do not regenerate lockfiles to conceal a failed check.
The [Spanish practical guides](../howToRunThis/README.md) include setup and troubleshooting.

## Windows development

Install rustup, MSVC x64 C++ Build Tools, Windows SDK headers/libraries, and
WebView2 Runtime. The repository's doctor only inspects prerequisites; it never
installs tools or changes system settings. See the
[official Tauri prerequisite guide](https://v2.tauri.app/start/prerequisites/).

Run from the repository root in PowerShell:

```powershell
rustup toolchain install 1.90.0 --profile minimal --component rustfmt --component clippy
npm ci
npm run doctor
npm run desktop:dev
```

Desktop dev starts Vite itself. Stop other servers using port 1420 first;
strictPort intentionally rejects conflicts. `npm run dev` starts browser UI only,
where native controls are disabled. Initialize audio explicitly from desktop.

To reproduce the Windows CI sequence after tool setup:

```powershell
npm run doctor
npm ci
npm run bindings:generate
npm run bindings:check
npm run check
npm run desktop:build -- --debug --no-bundle
```

Record hashes of lockfiles and generated bindings before and after this sequence
when local changes predate your check. In clean CI, `git diff --exit-code --
Cargo.lock package-lock.json src/shared/api/generated/api.ts` enforces immutability;
CI also compares bindings immediately after generation. Generated TS is excluded
from Prettier. `npm run check` runs binding comparison, typecheck, lint, frontend
tests, formatting, frontend build, nine CLI scenarios, rustfmt, explicit headless
tests/Clippy, and workspace tests/Clippy. It fails at the first failing command.

The final build creates `target/debug/rust-daw-desktop.exe` without an installer.
Tauri's `beforeBuildCommand` runs the frontend build; it does not recurse into
desktop build. Run desktop build after tests: Cargo tests may replace the main
executable with a development build that expects Vite's devUrl.

## What each verification proves

| Verification                             | Environment and evidence                                                                                                                                                     |
| ---------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Five headless packages and CLI scenarios | No CPAL, Tauri, window, or audio device. Deterministic engine/service/application behavior and CLI exits.                                                                    |
| Frontend tests                           | Vitest/jsdom, mocked desktop client; error display, observed state, command races, browser mode, listener cleanup.                                                           |
| Desktop integration tests                | Real registered Tauri handler and ACL checks with Tauri mock runtime/manual backend. No WebView2 window or audio hardware; Windows test manifests enable Common Controls v6. |
| Desktop debug build                      | Compiles native adapter and packages frontend assets. Does not open audio or demonstrate runtime behavior.                                                                   |
| Optional native CLI and UI smoke         | Requires actual output hardware; UI smoke additionally needs Windows/WebView2 and local debugging port 9223. Confirms lifecycle/callbacks, not absence of dropouts.          |

Mock-runtime tests can access application data directories outside the repository.
A restricted local sandbox may require an execution approval; this is distinct
from opening hardware. Missing `TaskDialogIndirect` in old test builds was resolved
by the test-only Common Controls v6 manifest in `src-tauri/build.rs`.

Optional hardware checks, excluded from CI and ordinary checks:

```powershell
cargo run -p daw-cli --features native-audio --locked -- devices
cargo run -p daw-cli --features native-audio --locked -- live-smoke --seconds 2
cargo test -p daw-audio --features native-audio --locked --test service_contract native_failure_recovery_and_restart_on_same_service -- --ignored
npm run desktop:build -- --debug --no-bundle
powershell -NoProfile -File scripts/desktop-smoke.ps1
```

Normal smoke writes only `target/desktop-stage4.png`. Updating the seven documented
screenshots requires explicitly passing `-UpdateDocs`; stage 6 preserves them and
their [dated metadata](../howToRunThis/screenshots/metadata.json). The
[stage 5 report](../howToRunThis/resultados.md) records actual WebView2/Realtek WASAPI
48 kHz stereo `f32` evidence from 2026-11-06. Stage 6 does not repeat hardware tests.

## Linux and macOS: intended versus verified

The headless contract is intended to compile anywhere the pinned Rust toolchain
runs. Its exact commands, also used by Linux CI, are:

```sh
cargo fmt --all -- --check
cargo test -p daw-domain -p daw-engine -p daw-audio -p daw-app -p daw-cli --no-default-features --locked
cargo clippy -p daw-domain -p daw-engine -p daw-audio -p daw-app -p daw-cli --no-default-features --all-targets --locked -- -D warnings
node howToRunThis/run-cli-cases.mjs
```

Node is needed only for the fixture runner here; npm installation is unnecessary.
`cargo fmt --all` inspects source without compiling desktop. Do not use
`cargo test --workspace` or workspace Clippy to claim dependency-free headless
verification: desktop enables CPAL and requires native dependencies.

No Linux or macOS build was run locally for this delivery. The hosted
[stage 8 run](https://github.com/YoZaiKK/AudioMOTORR/actions/runs/42524752652)
was inspected on 2026-11-08: both jobs passed for `1e6bca8`. This verifies Ubuntu
24.04 headless and Windows desktop checks for that implementation revision only;
later revisions require separate evidence. Linux CI installs no Tauri or CPAL
system dependencies.
Native Linux/macOS compilation, WebView integration, audio hosts, packaging, and
device behavior remain unverified. Consult Tauri's official prerequisites when
adding native coverage; no untested platform dependency recipe is claimed here.

## GitHub Actions

[ci.yml](../.github/workflows/ci.yml) triggers on pushes to `main`, pull requests,
and manual dispatch. It grants only `contents: read`, cancels obsolete runs for
the same workflow/ref, and pins official checkout/setup-node actions by complete
SHA with version comments. Rust is installed with rustup at 1.90.0; Node follows
`.node-version`. Windows verifies the exact npm version plus MSVC/SDK/WebView2
through the read-only doctor on the hosted `windows-2022` image. The image's
software inventory is [maintained by GitHub](https://github.com/actions/runner-images/blob/main/images/windows/Windows2022-Readme.md);
the workflow verifies prerequisites rather than treating that inventory as a test.

- `headless-linux`: formatting, explicit five-package tests/Clippy with
  `--no-default-features --locked`, nine CLI scenarios, and lockfile diff.
- `desktop-windows`: `npm ci`, deterministic binding regeneration/diff,
  `npm run check` including mock IPC tests, then desktop debug build without
  an installer; final lockfile/generated-type diff runs even after a failure.

There are no Cargo caches, third-party actions, hardware smoke, screenshot
updates, signing, uploads, releases, or publishing steps. Results stay in normal
Actions logs. Workflow syntax/expressions are checked locally with actionlint;
this is separate from a successful hosted run. The
[workflow's hosted runs](https://github.com/YoZaiKK/AudioMOTORR/actions/workflows/ci.yml)
show the remote status for each published revision. The stage 6 closure is a
dated record of local verification before the subsequent publication request;
it does not establish remote CI success.

Local closure commands and a criterion-by-criterion audit are recorded in
[stage 6 closure](../progress/cierre-etapa-6.md). Physical device removal, loopback
continuity, other hardware sample formats, and unverified platforms remain pending.
Stage 8 records bounded release processing measurements, which do not establish
a hard real-time guarantee.

## Stage 7 progress and next increment

Stage 7.2 adds six offline oscillator behavior tests in
`crates/daw-engine/tests/oscillator_contract.rs` plus overflow/phase regression
coverage. Existing checks discover these tests automatically. The
[execution plan](../progress/plan-etapas-7-13.md) and
[contract](offline-oscillator.md) describe engine behavior and application/WAV
export (7.3). New offline_contract and export_contract
tests cover block reuse, live-state independence, exact reopened WAV samples,
metadata, path/size errors, and injected writer failures. The nine silence cases
remain regressions. WAV tests use exclusive temporary output directories.
The CLI export-wav command is device-free. Stage 12 additionally pins hound 3.5.1
in daw-app for shared device-free beat export; see [project persistence/export](project-format.md).
Local and remote evidence is separated in the [export report](../progress/etapa-7-exportacion.md).
Stage 7 is closed: implementation c6cf627 passed both hosted jobs in
[run 42419963899](https://github.com/YoZaiKK/AudioMOTORR/actions/runs/42419963899).
The [closure audit](../progress/cierre-etapa-7.md) records that SHA's evidence;
future changes require their own verification.

## Stage 8 verification

Desktop now offers explicit sine initialization, frequency selection, and gain
commands; silence remains the default. See [native audition](native-audition.md)
for ramp semantics and reproducible release diagnostics. Run the real desktop
smoke with `powershell -NoProfile -File scripts/desktop-smoke.ps1 -Sine` after
check and desktop build. It writes target/desktop-stage8.png and rejects -UpdateDocs
to preserve the historical captures.

check additionally tests daw-audio with native-audio,rt-diagnostics and lints the
diagnostic CLI; no hardware is opened. Cargo.lock deliberately adds the optional
allocation-counter 0.8.1 dependency and generated DTOs deliberately change. After
these changes, locked builds and regeneration must preserve their new hashes;
package-lock.json remains unchanged. The [stage report](../progress/etapa-8.md)
records local results, hardware identity, and publication/CI status separately.
The implementation commit 1e6bca8 passed both hosted jobs in run 42524752652;
the later documentation closure preserves that exact evidence without changing DSP.

## Stage 9 resources and voice kernel

Sample resource tests are discovered by existing headless/workspace checks:

```powershell
cargo test -p daw-engine --no-default-features --locked --test sample_resource_contract
cargo test -p daw-engine --no-default-features --locked --test sampler_contract
cargo test -p daw-audio --features rt-diagnostics --locked --test sampler_memory_contract
cargo test -p daw-audio --release --features rt-diagnostics --locked --test sampler_memory_contract
```

Four tests cover exact PCM/metadata, invalid input, mono/stereo frame bounds,
and output compatibility. The [contract](sampler-contract.md) contains reusable
fixtures. Stage 9.2 adds eight kernel behavior tests, an internal overflow test,
and a feature-only allocation/deallocation test with a destruction positive control.
The kernel plays raw one-shots from memory; native sampler integration remains
pending. The separate 9.3.b adapter now decodes single assets. check runs both
the kernel diagnostic suite and the decoder/bank memory suite. 9.3.c adds
ordered bank coordination and real-file coexistence tests with two maximum-size
stereo banks; measured heap balance is distinct from the conservative usage
ceilings returned by PreparedBank and does not measure RSS/native deadlines.
This increment changes no dependencies, lockfiles, DTOs, or UI. The
[stage report](../progress/etapa-9-sampler.md) separates actual new checks from
the prior implementation's hosted results. Stage 8 closure 80a2121 passed
[run 42525499571](https://github.com/YoZaiKK/AudioMOTORR/actions/runs/42525499571);
that result verifies the starting revision, not these later resource changes.
Stage 9.1 revision 5da0920 passed
[run 42528127881](https://github.com/YoZaiKK/AudioMOTORR/actions/runs/42528127881);
the [current kernel report](../progress/etapa-9-voces.md) records 9.2 separately.

Stage 9.2 implementation 7acc1db passed both hosted jobs in
[run 42530699397](https://github.com/YoZaiKK/AudioMOTORR/actions/runs/42530699397),
inspected on 2026-11-08. Ubuntu verified headless behavior/Clippy/CLI scenarios;
Windows verified deterministic bindings, the full check including instrumented
kernel tests, and desktop debug build. Local release tests supplement that
hosted debug coverage; no native sampler stream was opened. Documentation-only
closure revisions have their own hosted runs and do not change the verified DSP.
