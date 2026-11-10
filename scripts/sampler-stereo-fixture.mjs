import { Buffer } from "node:buffer";
import { log } from "node:console";
import { createHash } from "node:crypto";
import { openSync, writeSync, closeSync, readFileSync } from "node:fs";
import process from "node:process";

const [output, ...extra] = process.argv.slice(2);
if (!output || extra.length)
  throw new Error("Usage: node scripts/sampler-stereo-fixture.mjs OUTPUT.wav");

// Fixed verification input, independent of the engine: ten seconds, distinct
// channels. Exclusive creation preserves earlier measurements and source files.
const rate = 48000;
const frames = 480000;
const header = Buffer.alloc(44);
header.write("RIFF", 0);
header.writeUInt32LE(36 + frames * 8, 4);
header.write("WAVEfmt ", 8);
header.writeUInt32LE(16, 16);
header.writeUInt16LE(3, 20);
header.writeUInt16LE(2, 22);
header.writeUInt32LE(rate, 24);
header.writeUInt32LE(rate * 8, 28);
header.writeUInt16LE(8, 32);
header.writeUInt16LE(32, 34);
header.write("data", 36);
header.writeUInt32LE(frames * 8, 40);
const fd = openSync(output, "wx");
function writeAll(buffer) {
  let offset = 0;
  while (offset < buffer.length) {
    const written = writeSync(fd, buffer, offset, buffer.length - offset);
    if (written === 0) throw new Error("Fixture write made no progress");
    offset += written;
  }
}
try {
  writeAll(header);
  const chunk = Buffer.alloc(256 * 8);
  for (let start = 0; start < frames; start += 256) {
    const count = Math.min(256, frames - start);
    for (let frame = 0; frame < count; frame++) {
      const time = (start + frame) / rate;
      chunk.writeFloatLE(0.1 * Math.sin(2 * Math.PI * 997 * time), frame * 8);
      chunk.writeFloatLE(
        0.05 * Math.sin(2 * Math.PI * 440 * time),
        frame * 8 + 4,
      );
    }
    writeAll(chunk.subarray(0, count * 8));
  }
} finally {
  closeSync(fd);
}
log(
  JSON.stringify({
    output,
    rate,
    channels: 2,
    frames,
    format: "float32",
    sha256: createHash("sha256").update(readFileSync(output)).digest("hex"),
  }),
);
