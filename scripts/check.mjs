import { spawnSync } from "node:child_process";
import process from "node:process";

const npmCli = process.env.npm_execpath;
if (!npmCli) {
  throw new Error("Ejecuta este script mediante npm run check.");
}

const checks = [
  [process.execPath, [npmCli, "run", "bindings:check"]],
  [process.execPath, [npmCli, "run", "typecheck"]],
  [process.execPath, [npmCli, "run", "lint"]],
  [process.execPath, [npmCli, "run", "test:frontend"]],
  [process.execPath, [npmCli, "run", "format:check"]],
  [process.execPath, [npmCli, "run", "build"]],
  [process.execPath, ["howToRunThis/run-cli-cases.mjs"]],
  ["cargo", ["fmt", "--all", "--", "--check"]],
  [
    "cargo",
    [
      "test",
      "-p",
      "daw-domain",
      "-p",
      "daw-engine",
      "-p",
      "daw-audio",
      "-p",
      "daw-app",
      "-p",
      "daw-cli",
      "--no-default-features",
      "--locked",
    ],
  ],
  [
    "cargo",
    [
      "clippy",
      "-p",
      "daw-domain",
      "-p",
      "daw-engine",
      "-p",
      "daw-audio",
      "-p",
      "daw-app",
      "-p",
      "daw-cli",
      "--no-default-features",
      "--all-targets",
      "--locked",
      "--",
      "-D",
      "warnings",
    ],
  ],
  ["cargo", ["test", "--workspace", "--locked"]],
  [
    "cargo",
    [
      "test",
      "-p",
      "daw-audio",
      "--features",
      "native-audio,rt-diagnostics",
      "--locked",
    ],
  ],
  [
    "cargo",
    [
      "clippy",
      "-p",
      "daw-cli",
      "--features",
      "rt-diagnostics",
      "--all-targets",
      "--locked",
      "--",
      "-D",
      "warnings",
    ],
  ],
  [
    "cargo",
    [
      "test",
      "-p",
      "daw-cli",
      "--features",
      "offline-diagnostics",
      "--locked",
      "--bin",
      "daw-cli",
      "--test",
      "diagnostics_contract",
    ],
  ],
  [
    "cargo",
    [
      "test",
      "-p",
      "daw-cli",
      "--features",
      "rt-diagnostics",
      "--locked",
      "--test",
      "sample_loading_memory_contract",
    ],
  ],
  [
    "cargo",
    [
      "clippy",
      "--workspace",
      "--all-targets",
      "--locked",
      "--",
      "-D",
      "warnings",
    ],
  ],
];

for (const [command, args] of checks) {
  const result = spawnSync(command, args, { stdio: "inherit" });
  if (result.error) {
    process.stderr.write(`${result.error.message}\n`);
  }
  if (result.error || result.status !== 0) {
    process.exit(result.status ?? 1);
  }
}
