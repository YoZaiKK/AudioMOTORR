// Optional real WebView2/hardware smoke. Not part of automated npm run check.
import process from "node:process";
import console from "node:console";
import { Buffer } from "node:buffer";
import { setTimeout, clearTimeout } from "node:timers";
import { writeFile, mkdir } from "node:fs/promises";
import { setTimeout as delay } from "node:timers/promises";
import os from "node:os";

const port = Number(process.argv[2] ?? 9223);
const updateDocs = process.argv.includes("--update-docs");
const sine = process.argv.includes("--sine");
const screenshotPath = sine
  ? "target/desktop-stage8.png"
  : "target/desktop-stage4.png";
const captures = [];
if (!Number.isInteger(port) || port < 1024 || port > 65535)
  throw new Error("Invalid local debug port");
async function until(check, description) {
  const deadline = Date.now() + 10000;
  while (Date.now() < deadline) {
    const result = await check();
    if (result) return result;
    await delay(50);
  }
  throw new Error(`Timed out: ${description}`);
}

const page = await until(async () => {
  try {
    const response = await globalThis.fetch(
      `http://127.0.0.1:${port}/json/list`,
    );
    const pages = await response.json();
    return pages.find(
      (page) => page.type === "page" && page.url.includes("tauri.localhost"),
    );
  } catch {
    return null;
  }
}, "WebView2 debug target");
const socket = new globalThis.WebSocket(page.webSocketDebuggerUrl);
await new Promise((resolve, reject) => {
  socket.addEventListener("open", resolve, { once: true });
  socket.addEventListener("error", reject, { once: true });
});
let nextId = 0;
const pending = new Map();
socket.addEventListener("message", (event) => {
  const message = JSON.parse(String(event.data));
  const request = pending.get(message.id);
  if (!request) return;
  pending.delete(message.id);
  clearTimeout(request.timeout);
  if (message.error) request.reject(new Error(JSON.stringify(message.error)));
  else request.resolve(message.result);
});
function send(method, params = {}) {
  const id = ++nextId;
  return new Promise((resolve, reject) => {
    const timeout = setTimeout(() => {
      pending.delete(id);
      reject(new Error(`CDP timeout: ${method}`));
    }, 10000);
    pending.set(id, { resolve, reject, timeout });
    socket.send(JSON.stringify({ id, method, params }));
  });
}
async function evaluate(expression) {
  const result = await send("Runtime.evaluate", {
    expression,
    awaitPromise: true,
    returnByValue: true,
  });
  if (result.exceptionDetails)
    throw new Error(JSON.stringify(result.exceptionDetails));
  return result.result.value;
}
function click(label) {
  return evaluate(
    `(() => { const button = [...document.querySelectorAll('button')].find(b => b.textContent === ${JSON.stringify(label)}); if (!button || button.disabled) throw new Error('Button unavailable'); button.click(); return true; })()`,
  );
}
const technical = (label) =>
  evaluate(
    `(() => { const row = [...document.querySelectorAll('dl > div')].find(r => r.querySelector('dt').textContent === ${JSON.stringify(label)}); return row?.querySelector('dd').textContent; })()`,
  );
const transport = () =>
  evaluate(
    "document.querySelector('[data-testid=transport-state]')?.textContent",
  );
const snapshot = () =>
  evaluate("window.__TAURI_INTERNALS__.invoke('get_engine_snapshot')");

async function capture(name, description) {
  if (!updateDocs) return;
  const layout = await send("Page.getLayoutMetrics");
  const png = await send("Page.captureScreenshot", {
    format: "png",
    captureBeyondViewport: true,
    clip: {
      x: 0,
      y: 0,
      width: layout.cssContentSize.width,
      height: layout.cssContentSize.height,
      scale: 1,
    },
  });
  await mkdir("howToRunThis/screenshots", { recursive: true });
  await writeFile(
    `howToRunThis/screenshots/${name}.png`,
    Buffer.from(png.data, "base64"),
  );
  captures.push({
    file: `${name}.png`,
    description,
    capturedAt: new Date().toISOString(),
    observed: await snapshot(),
  });
}

try {
  await until(
    () =>
      evaluate(
        "[...document.querySelectorAll('button')].some(b => b.textContent === 'Inicializar audio' && !b.disabled)",
      ),
    "UI connection",
  );
  await capture(
    "01-sin-inicializar",
    "Servicio sin inicializar; transporte deshabilitado.",
  );
  const missing = await evaluate(
    "window.__TAURI_INTERNALS__.invoke('initialize_audio', { deviceId: 'missing-device-for-desktop-smoke' }).then(() => null, error => error)",
  );
  if (missing?.code !== "deviceAbsent")
    throw new Error("Missing-device IPC error was not structured");
  await until(
    () =>
      evaluate(
        "document.querySelector('[role=alert]')?.textContent.includes('deviceAbsent')",
      ),
    "initialization error in UI",
  );
  await capture(
    "02-error-dispositivo",
    "Error real de IPC por ID inexistente; recuperación disponible.",
  );
  if (sine) {
    await evaluate(
      "(() => { const input = document.querySelector('#signal-source'); input.value = 'sine'; input.dispatchEvent(new Event('change', { bubbles: true })); return true; })()",
    );
  }
  await click("Inicializar audio");
  await until(
    async () => (await technical("Servicio")) === "ready",
    "audio ready in UI",
  );
  await click("Play");
  await until(
    async () => (await transport()) === "Reproduciendo",
    "applied Play in UI",
  );
  await until(
    async () => BigInt((await snapshot()).engine.deviceFrames) > 0n,
    "real callback frames",
  );
  await evaluate(
    "(() => { const input = document.querySelector('#tempo'); input.value = '137.5'; input.form.requestSubmit(); return true; })()",
  );
  await until(
    () =>
      evaluate(
        "document.querySelector('#tempo').form.textContent.includes('Aplicado: 137.5 BPM')",
      ),
    "applied BPM in UI",
  );
  await capture(
    "03-reproduciendo",
    "Play y 137.5 BPM aplicados; contadores reales.",
  );
  if (sine) {
    if ((await snapshot()).engine.sineFrequencyHz !== 440)
      throw new Error("Native sine source was not applied");
    await evaluate(
      "(() => { const input = document.querySelector('#signal-gain'); input.value = '0.25'; input.form.requestSubmit(); return true; })()",
    );
    await until(
      () =>
        evaluate(
          "document.querySelector('#signal-gain')?.form.textContent.includes('Ganancia observada: 0.250')",
        ),
      "observed native gain ramp in UI",
    );
  }
  await click("Pause");
  await until(
    async () => (await transport()) === "Pausado",
    "applied Pause in UI",
  );
  await capture("04-pausado", "Pause aplicado; posición conservada.");
  await click("Stop");
  await until(
    async () =>
      (await transport()) === "Detenido" &&
      (await technical("Posición (frames)")) === "0",
    "applied Stop in UI",
  );
  await capture("05-detenido", "Stop aplicado; posición musical en cero.");
  const observed = await snapshot();
  await click("Cerrar audio");
  await until(
    async () => (await technical("Servicio")) === "uninitialized",
    "audio shutdown in UI",
  );
  await click("Inicializar audio");
  await until(
    async () => (await technical("Servicio")) === "ready",
    "audio restart in UI",
  );
  await until(
    async () => BigInt(await technical("Callbacks")) > 0n,
    "restarted callbacks in UI",
  );
  const restarted = await snapshot();
  if (
    restarted.engine.tempo !== 137.5 ||
    restarted.engine.transport !== "stopped"
  )
    throw new Error("Restart state mismatch");
  if (
    sine &&
    (restarted.engine.sineFrequencyHz !== 440 || restarted.engine.gain !== 0)
  )
    throw new Error("Sine restart must be stopped at zero effective gain");
  await capture(
    "06-reiniciado",
    "Audio reiniciado, detenido y con callbacks; tempo conservado.",
  );
  const layout = await send("Page.getLayoutMetrics");
  const screenshot = await send("Page.captureScreenshot", {
    format: "png",
    captureBeyondViewport: true,
    clip: {
      x: 0,
      y: 0,
      width: layout.cssContentSize.width,
      height: layout.cssContentSize.height,
      scale: 1,
    },
  });
  await mkdir("target", { recursive: true });
  await writeFile(screenshotPath, Buffer.from(screenshot.data, "base64"));
  await click("Cerrar audio");
  await until(
    async () => (await technical("Servicio")) === "uninitialized",
    "final shutdown",
  );
  await capture(
    "07-audio-cerrado",
    "Servicio cerrado; controles de transporte deshabilitados.",
  );
  if (updateDocs)
    await writeFile(
      "howToRunThis/screenshots/metadata.json",
      JSON.stringify(
        {
          platform: os.platform(),
          release: os.release(),
          node: process.version,
          app: await evaluate(
            "window.__TAURI_INTERNALS__.invoke('get_app_info')",
          ),
          note: "Snapshots taken after screenshots; counters may differ slightly.",
          captures,
        },
        null,
        2,
      ) + "\n",
    );
  console.log(
    JSON.stringify({
      result: "desktop smoke OK",
      callbacks: observed.callbacks,
      output: observed.output,
      lastAppliedCommand: observed.lastAppliedCommand,
      restartTempo: restarted.engine.tempo,
      screenshot: screenshotPath,
      source: sine ? "sine" : "silence",
    }),
  );
} finally {
  try {
    await evaluate("window.__TAURI_INTERNALS__.invoke('shutdown_audio')");
  } catch {
    /* Cleanup after a lost target is handled by the native window lifecycle. */
  }
  socket.close();
}
