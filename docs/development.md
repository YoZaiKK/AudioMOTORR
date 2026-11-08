# Development, platforms, and CI

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
The CLI export-wav command is device-free; hound 3.5.1 is pinned only there.
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

## Stage 9.1 resources

Sample resource tests are discovered by existing headless/workspace checks:

```powershell
cargo test -p daw-engine --no-default-features --locked --test sample_resource_contract
```

Four tests cover exact PCM/metadata, invalid input, mono/stereo frame bounds,
and output compatibility. The [contract](sampler-contract.md) contains reusable
fixtures and future voice oracles; playback and decoding are not implemented.
This increment changes no dependencies, lockfiles, DTOs, or UI. The
[stage report](../progress/etapa-9-sampler.md) separates actual new checks from
the prior implementation's hosted results. Stage 8 closure 80a2121 passed
[run 42525499571](https://github.com/YoZaiKK/AudioMOTORR/actions/runs/42525499571);
that result verifies the starting revision, not these later resource changes.
