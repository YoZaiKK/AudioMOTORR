import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { fileURLToPath, URL } from "node:url";
import path from "node:path";
import process from "node:process";
import console from "node:console";

const root = fileURLToPath(new URL("../", import.meta.url));
const cases = JSON.parse(
  readFileSync(
    new URL("test-files/cli-cases.json", import.meta.url),
    "utf8",
  ).replace(/^\uFEFF/, ""),
);
if (!Array.isArray(cases) || cases.length === 0)
  throw new Error("Expected nonempty case list");
for (const item of cases) {
  if (
    typeof item.name !== "string" ||
    !Array.isArray(item.args) ||
    item.args[0] !== "smoke" ||
    !item.args.every((arg) => typeof arg === "string") ||
    ![0, 1].includes(item.exit) ||
    (item.stdout !== undefined && typeof item.stdout !== "string") ||
    (item.stderr !== undefined && typeof item.stderr !== "string")
  )
    throw new Error("Invalid CLI case");
}
function run(command, args) {
  const result = spawnSync(command, args, {
    cwd: root,
    encoding: "utf8",
    timeout: 120000,
    maxBuffer: 4 * 1024 * 1024,
  });
  if (result.error) throw result.error;
  return result;
}
const build = run("cargo", [
  "build",
  "-p",
  "daw-cli",
  "--no-default-features",
  "--locked",
]);
if (build.status !== 0) throw new Error(build.stderr);
const metadata = run("cargo", [
  "metadata",
  "--no-deps",
  "--format-version",
  "1",
  "--locked",
]);
if (metadata.status !== 0) throw new Error(metadata.stderr);
const binary = path.join(
  JSON.parse(metadata.stdout).target_directory,
  "debug",
  process.platform === "win32" ? "daw-cli.exe" : "daw-cli",
);
for (const item of cases) {
  const result = run(binary, item.args);
  if (
    result.status !== item.exit ||
    (item.stdout && !result.stdout.includes(item.stdout)) ||
    (item.stderr && !result.stderr.includes(item.stderr))
  ) {
    throw new Error(
      `${item.name}: unexpected result (${result.status})\n${result.stdout}\n${result.stderr}`,
    );
  }
  console.log(`PASS ${item.name}: exit=${result.status}`);
}
console.log(
  `${cases.length} CLI cases passed; manual backend, no audio device opened.`,
);
