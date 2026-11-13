import { Buffer } from "node:buffer";
import { log } from "node:console";
import { createHash } from "node:crypto";
import { mkdirSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";
import process from "node:process";

const [directory, ...args] = process.argv.slice(2);
let channels = 2;
let rate = 48000;
const seen = new Set();
if (!directory || args.length % 2)
  throw new Error(
    "Usage: node scripts/sequencer-fixtures.mjs NEW_DIR [--channels 1|2] [--sample-rate 100..192000]",
  );
for (let index = 0; index < args.length; index += 2) {
  const key = args[index];
  if (seen.has(key)) throw new Error("Duplicate fixture option");
  seen.add(key);
  if (key === "--channels") channels = Number(args[index + 1]);
  else if (key === "--sample-rate") rate = Number(args[index + 1]);
  else throw new Error("Unknown fixture option");
}
if (
  ![1, 2].includes(channels) ||
  !Number.isInteger(rate) ||
  rate < 100 ||
  rate > 192000
)
  throw new Error("Invalid fixture channel count/sample rate");

// Exclusive directory and files: never replace earlier input/evidence.
mkdirSync(directory);
const names = ["kick.wav", "snare.wav", "hat.wav", "accent.wav"];
const durations = [0.12, 0.18, 0.06, 0.24];
const files = names.map((name, slot) => {
  const frames = Math.ceil(rate * durations[slot]);
  const bytes = Buffer.alloc(44 + frames * channels * 4);
  bytes.write("RIFF", 0);
  bytes.writeUInt32LE(bytes.length - 8, 4);
  bytes.write("WAVEfmt ", 8);
  bytes.writeUInt32LE(16, 16);
  bytes.writeUInt16LE(3, 20);
  bytes.writeUInt16LE(channels, 22);
  bytes.writeUInt32LE(rate, 24);
  bytes.writeUInt32LE(rate * channels * 4, 28);
  bytes.writeUInt16LE(channels * 4, 32);
  bytes.writeUInt16LE(32, 34);
  bytes.write("data", 36);
  bytes.writeUInt32LE(bytes.length - 44, 40);
  let seed = 0x12345678 + slot;
  let previousNoise = 0;
  for (let frame = 0; frame < frames; frame++) {
    const time = frame / rate;
    seed = (Math.imul(seed, 1664525) + 1013904223) >>> 0;
    const noise = seed / 0x80000000 - 1;
    const envelope =
      Math.min(1, frame / Math.max(1, rate * 0.001)) *
      (1 - frame / (frames - 1)) ** 2;
    const tone = [
      0.12 *
        Math.sin(
          2 * Math.PI * (55 * time + (30 * (1 - Math.exp(-30 * time))) / 30),
        ),
      0.07 * noise + 0.03 * Math.sin(2 * Math.PI * 180 * time),
      0.04 * (noise - previousNoise),
      0.1 * Math.sin(2 * Math.PI * 660 * time),
    ][slot];
    previousNoise = noise;
    for (let side = 0; side < channels; side++) {
      const balance = side === 0 || slot === 0 ? 1 : [1, 0.65, -0.5, 0.8][slot];
      bytes.writeFloatLE(
        tone * envelope * balance,
        44 + (frame * channels + side) * 4,
      );
    }
  }
  writeFileSync(join(directory, name), bytes, { flag: "wx" });
  return {
    name,
    frames,
    sha256: createHash("sha256").update(bytes).digest("hex"),
  };
});
const manifest = {
  format_version: 1,
  generator: "scripts/sequencer-fixtures.mjs",
  output_directory: resolve(directory),
  rate,
  channels,
  encoding: "float32",
  files,
};
writeFileSync(
  join(directory, "manifest.json"),
  JSON.stringify(manifest, null, 2) + "\n",
  { flag: "wx" },
);
log(JSON.stringify(manifest));
