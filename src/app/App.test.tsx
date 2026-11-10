import { StrictMode } from "react";
import {
  act,
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type {
  PreparationSnapshotDto,
  ServiceSnapshotDto,
} from "../shared/api/generated/api";

const client = vi.hoisted(() => ({
  available: vi.fn(),
  appInfo: vi.fn(),
  project: vi.fn(),
  devices: vi.fn(),
  initialize: vi.fn(),
  shutdown: vi.fn(),
  snapshot: vi.fn(),
  snapshots: vi.fn(),
  play: vi.fn(),
  pause: vi.fn(),
  stop: vi.fn(),
  tempo: vi.fn(),
  gain: vi.fn(),
  prepareSamples: vi.fn(),
  preparation: vi.fn(),
  cancelPreparation: vi.fn(),
  initializeSampler: vi.fn(),
  triggerSample: vi.fn(),
}));
vi.mock("../shared/api/desktop-client", async (original) => ({
  ...(await original<typeof import("../shared/api/desktop-client")>()),
  desktopClient: client,
}));
import { App } from "./App";

const empty: ServiceSnapshotDto = {
  sequence: "0",
  state: "uninitialized",
  backend: null,
  output: null,
  engine: null,
  callbacks: "0",
  lastAppliedCommand: null,
  error: null,
  sampler: null,
};
const ready: ServiceSnapshotDto = {
  ...empty,
  sequence: "1",
  state: "ready",
  backend: "native",
  engine: {
    sampleRate: 48000,
    channels: 2,
    tempo: 120,
    transport: "stopped",
    deviceFrames: "0",
    transportFrames: "0",
    requestedTransport: "stopped",
    sineFrequencyHz: null,
    sineAmplitude: null,
    gain: 1,
    targetGain: 1,
    rampRemainingFrames: 0,
  },
};
const listeners = new Set<(value: ServiceSnapshotDto) => void>();
const prepared: PreparationSnapshotDto = {
  sequence: "2",
  preparationId: "1",
  state: "ready",
  sampleRate: 48000,
  channels: 2,
  occupiedSlots: 1,
  retainedPcmBytes: "4096",
  error: null,
};
const samplerReady: ServiceSnapshotDto = {
  ...ready,
  sampler: {
    occupiedSlots: 1,
    activeVoices: 0,
    started: "0",
    rejected: "0",
    lastTrigger: null,
  },
};
function emit(value: ServiceSnapshotDto) {
  act(() => {
    for (const listener of listeners) listener(value);
  });
}
async function connected() {
  await waitFor(() => expect(client.devices).toHaveBeenCalled());
}

beforeEach(() => {
  vi.resetAllMocks();
  listeners.clear();
  client.available.mockReturnValue(true);
  client.appInfo.mockResolvedValue({ name: "Rust DAW", version: "0.1.0" });
  client.project.mockResolvedValue({
    id: "1",
    title: "Proyecto sin título",
    tempo: 120,
  });
  client.devices.mockResolvedValue([]);
  client.snapshot.mockResolvedValue(empty);
  client.preparation.mockResolvedValue({
    sequence: "0",
    preparationId: null,
    state: "idle",
    sampleRate: null,
    channels: null,
    occupiedSlots: 0,
    retainedPcmBytes: "0",
    error: null,
  });
  client.snapshots.mockImplementation(async (callback) => {
    listeners.add(callback);
    return () => {
      listeners.delete(callback);
    };
  });
});
afterEach(cleanup);

describe("audio interface", () => {
  it("still queries and discards a prepared bank when device enumeration fails", async () => {
    client.devices.mockRejectedValue({
      code: "streamError",
      message: "Device enumeration failed",
      details: null,
    });
    client.preparation.mockResolvedValue(prepared);
    client.cancelPreparation.mockResolvedValue({
      ...prepared,
      sequence: "3",
      state: "cancelled",
      occupiedSlots: 0,
    });
    render(<App />);
    await screen.findByRole("alert");
    await screen.findByText(/Banco listo/);
    fireEvent.click(
      screen.getByRole("button", { name: "Cancelar / descartar banco" }),
    );
    await screen.findByText("Carga cancelada");
    expect(client.cancelPreparation).toHaveBeenCalledWith("1");
    expect(client.initializeSampler).not.toHaveBeenCalled();
  });

  it.each(["cancelling", "replacement"] as const)(
    "never overlaps a slow poll across %s",
    async (scenario) => {
      client.preparation.mockResolvedValueOnce({
        ...prepared,
        sequence: "1",
        state: "loading",
      });
      let resolve!: (value: PreparationSnapshotDto) => void;
      client.preparation.mockImplementationOnce(
        () =>
          new Promise((done) => {
            resolve = done;
          }),
      );
      client.preparation.mockResolvedValue({
        ...prepared,
        sequence: "5",
        state: "cancelled",
        occupiedSlots: 0,
      });
      client.cancelPreparation.mockResolvedValue({
        ...prepared,
        sequence: "2",
        state: scenario === "cancelling" ? "cancelling" : "cancelled",
        occupiedSlots: 0,
      });
      render(<App />);
      await screen.findByText("Cargando WAV…");
      await waitFor(() => expect(client.preparation).toHaveBeenCalledTimes(2));
      fireEvent.click(
        screen.getByRole("button", { name: "Cancelar / descartar banco" }),
      );
      if (scenario === "replacement") {
        await screen.findByText("Carga cancelada");
        client.prepareSamples.mockResolvedValue({
          ...prepared,
          sequence: "3",
          preparationId: "2",
          state: "loading",
          occupiedSlots: 0,
        });
        fireEvent.change(screen.getByLabelText("Ruta WAV · slot 0"), {
          target: { value: "replacement.wav" },
        });
        fireEvent.click(
          screen.getByRole("button", { name: "Cargar banco WAV" }),
        );
        await screen.findByText("Cargando WAV…");
      } else {
        await screen.findByText("Cancelando carga…");
      }
      try {
        await act(async () => {
          await new Promise((done) => setTimeout(done, 300));
        });
        expect(client.preparation).toHaveBeenCalledTimes(2);
      } finally {
        await act(async () => {
          resolve(prepared);
        });
      }
      await screen.findByText("Carga cancelada");
      expect(screen.queryByText(/Banco listo/)).toBeNull();
      expect(client.preparation).toHaveBeenCalledTimes(3);
    },
  );

  it("loads in the background and initializes and triggers only on explicit clicks", async () => {
    client.prepareSamples.mockResolvedValue({
      ...prepared,
      sequence: "1",
      state: "loading",
      occupiedSlots: 0,
    });
    render(<App />);
    await screen.findByText("Sin banco preparado");
    fireEvent.change(screen.getByLabelText("Ruta WAV · slot 0"), {
      target: { value: "C:\\samples\\kick.wav" },
    });
    expect(client.prepareSamples).not.toHaveBeenCalled();
    client.preparation.mockResolvedValue(prepared);
    fireEvent.click(screen.getByRole("button", { name: "Cargar banco WAV" }));
    await screen.findByText("Cargando WAV…");
    expect(client.prepareSamples).toHaveBeenCalledWith(48000, 2, [
      { slot: 0, path: "C:\\samples\\kick.wav" },
    ]);
    expect(client.initializeSampler).not.toHaveBeenCalled();
    await screen.findByText(/Banco listo/);
    client.preparation.mockResolvedValue({
      ...prepared,
      sequence: "3",
      state: "idle",
      occupiedSlots: 0,
      retainedPcmBytes: "0",
    });
    client.initializeSampler.mockResolvedValue(samplerReady);
    fireEvent.click(
      screen.getByRole("button", { name: "Inicializar sampler" }),
    );
    await screen.findByText(/Sampler one-shot · 16 voces/);
    expect(client.initializeSampler).toHaveBeenCalledWith("1", null);
    expect(client.play).not.toHaveBeenCalled();
    expect(client.triggerSample).not.toHaveBeenCalled();
    client.triggerSample.mockResolvedValue({ commandId: "9" });
    fireEvent.click(screen.getByRole("button", { name: "Disparar sample" }));
    await screen.findByText(/Disparo 9: aceptado/);
    emit({
      ...samplerReady,
      sequence: "2",
      lastAppliedCommand: "9",
      sampler: {
        ...samplerReady.sampler!,
        rejected: "1",
        lastTrigger: {
          commandId: "9",
          outcome: { status: "rejected", reason: "notPlaying" },
        },
      },
    });
    expect(screen.getByTestId("trigger-outcome").textContent).toContain(
      "rechazado (notPlaying)",
    );
    client.triggerSample.mockResolvedValue({ commandId: "10" });
    fireEvent.click(screen.getByRole("button", { name: "Disparar sample" }));
    await screen.findByText(/Disparo 10: aceptado/);
    emit({
      ...samplerReady,
      sequence: "3",
      lastAppliedCommand: "10",
      sampler: {
        ...samplerReady.sampler!,
        started: "1",
        lastTrigger: {
          commandId: "10",
          outcome: { status: "started", voice: 0 },
        },
      },
    });
    expect(screen.getByTestId("trigger-outcome").textContent).toContain(
      "voz 0 iniciada",
    );
  });

  it("discards a ready bank and consumes preparation even if native initialization fails", async () => {
    client.preparation.mockResolvedValue(prepared);
    client.cancelPreparation.mockResolvedValue({
      ...prepared,
      sequence: "3",
      state: "cancelled",
      occupiedSlots: 0,
    });
    render(<App />);
    await screen.findByText(/Banco listo/);
    fireEvent.click(
      screen.getByRole("button", { name: "Cancelar / descartar banco" }),
    );
    await screen.findByText("Carga cancelada");
    expect(client.cancelPreparation).toHaveBeenCalledWith("1");
    client.prepareSamples.mockResolvedValue({
      ...prepared,
      preparationId: "2",
      sequence: "4",
    });
    fireEvent.change(screen.getByLabelText("Ruta WAV · slot 0"), {
      target: { value: "kick.wav" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Cargar banco WAV" }));
    await screen.findByText(/Banco listo/);
    client.initializeSampler.mockRejectedValue({
      code: "unsupportedChannels",
      message: "Unsupported output layout",
      details: null,
    });
    client.preparation.mockResolvedValue({
      ...prepared,
      preparationId: "2",
      sequence: "5",
      state: "idle",
    });
    fireEvent.click(
      screen.getByRole("button", { name: "Inicializar sampler" }),
    );
    await screen.findByText("Sin banco preparado");
    expect(screen.getByRole("alert").textContent).toContain(
      "Unsupported output layout",
    );
    expect(
      (
        screen.getByRole("button", {
          name: "Inicializar sampler",
        }) as HTMLButtonElement
      ).disabled,
    ).toBe(true);
    expect(client.play).not.toHaveBeenCalled();
  });

  it.each(["started", "rejected", "lost", "closed"] as const)(
    "handles %s observation arriving before trigger acceptance",
    async (scenario) => {
      client.snapshot.mockResolvedValue(samplerReady);
      let resolve!: (value: { commandId: string }) => void;
      client.triggerSample.mockImplementation(
        () =>
          new Promise((done) => {
            resolve = done;
          }),
      );
      render(<App />);
      await connected();
      fireEvent.click(screen.getByRole("button", { name: "Disparar sample" }));
      const id = "9007199254740993";
      emit({
        ...samplerReady,
        sequence: "9007199254740993",
        state: scenario === "closed" ? "uninitialized" : "ready",
        lastAppliedCommand: scenario === "lost" ? "9007199254740994" : id,
        sampler:
          scenario === "closed"
            ? null
            : {
                ...samplerReady.sampler!,
                lastTrigger: {
                  commandId: scenario === "lost" ? "9007199254740994" : id,
                  outcome:
                    scenario === "rejected"
                      ? { status: "rejected", reason: "poolFull" }
                      : { status: "started", voice: 15 },
                },
              },
      });
      await act(async () => {
        resolve({ commandId: id });
      });
      const expected =
        scenario === "started"
          ? "voz 15 iniciada"
          : scenario === "rejected"
            ? "rechazado (poolFull)"
            : "resultado desconocido";
      expect(screen.getByTestId("trigger-outcome").textContent).toContain(
        expected,
      );
      emit({ ...samplerReady, sequence: "2" });
      expect(screen.getByTestId("trigger-outcome").textContent).toContain(
        expected,
      );
    },
  );

  it("shows queue admission errors without inventing a trigger outcome", async () => {
    client.snapshot.mockResolvedValue(samplerReady);
    client.triggerSample.mockRejectedValue({
      code: "queueFull",
      message: "Queue full",
      details: null,
    });
    render(<App />);
    await connected();
    fireEvent.click(screen.getByRole("button", { name: "Disparar sample" }));
    await screen.findByRole("alert");
    expect(screen.queryByTestId("trigger-outcome")).toBeNull();
    expect(screen.getByRole("alert").textContent).toContain("Queue full");
  });

  it("ignores a delayed preparation reply after cancellation and stops polling", async () => {
    client.preparation.mockResolvedValueOnce({
      ...prepared,
      sequence: "1",
      state: "loading",
    });
    let resolve!: (value: PreparationSnapshotDto) => void;
    client.preparation.mockImplementationOnce(
      () =>
        new Promise((done) => {
          resolve = done;
        }),
    );
    client.cancelPreparation.mockResolvedValue({
      ...prepared,
      sequence: "3",
      state: "cancelled",
      occupiedSlots: 0,
    });
    const view = render(<App />);
    await screen.findByText("Cargando WAV…");
    await waitFor(() => expect(client.preparation).toHaveBeenCalledTimes(2));
    fireEvent.click(
      screen.getByRole("button", { name: "Cancelar / descartar banco" }),
    );
    await screen.findByText("Carga cancelada");
    await act(async () => {
      resolve(prepared);
    });
    expect(screen.queryByText(/Banco listo/)).toBeNull();
    view.unmount();
    expect(listeners.size).toBe(0);
    expect(client.prepareSamples).not.toHaveBeenCalled();
  });

  it("selects a sine only on explicit initialization and observes gain ramps", async () => {
    const sineReady: ServiceSnapshotDto = {
      ...ready,
      engine: {
        ...ready.engine!,
        sineFrequencyHz: 997,
        sineAmplitude: 0.1,
        gain: 0,
      },
    };
    client.initialize.mockResolvedValue(sineReady);
    client.gain.mockResolvedValue({ commandId: "12" });
    render(<App />);
    await connected();
    fireEvent.change(screen.getByLabelText("Señal"), {
      target: { value: "sine" },
    });
    fireEvent.change(screen.getByLabelText("Frecuencia (Hz)"), {
      target: { value: "997" },
    });
    expect(client.initialize).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole("button", { name: "Inicializar audio" }));
    await waitFor(() =>
      expect(client.initialize).toHaveBeenCalledWith(null, 997),
    );
    await screen.findByLabelText("Volumen (0–1)");
    expect(client.play).not.toHaveBeenCalled();
    fireEvent.change(screen.getByLabelText("Volumen (0–1)"), {
      target: { value: "0.25" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Aplicar volumen" }));
    await screen.findByText(/Comando 12 aceptado/);
    expect(client.gain).toHaveBeenCalledWith(0.25);
    expect(screen.getByText(/Ganancia observada:/).textContent).toContain(
      "0.000",
    );
    emit({
      ...sineReady,
      sequence: "2",
      lastAppliedCommand: "12",
      engine: {
        ...sineReady.engine!,
        transport: "playing",
        requestedTransport: "paused",
        gain: 0.125,
        targetGain: 0.25,
        rampRemainingFrames: 120,
      },
    });
    expect(screen.getByText(/Transición a Pausado/)).toBeTruthy();
    expect(screen.getByText(/Ganancia observada:/).textContent).toContain(
      "0.125",
    );
    emit({
      ...sineReady,
      sequence: "3",
      lastAppliedCommand: "12",
      engine: {
        ...sineReady.engine!,
        transport: "paused",
        requestedTransport: "paused",
        gain: 0,
        targetGain: 0.25,
      },
    });
    expect(screen.queryByText(/Transición a Pausado/)).toBeNull();
  });

  it("recovers from initialization failure without starting playback", async () => {
    client.initialize
      .mockRejectedValueOnce({
        code: "deviceAbsent",
        message: "No output device",
        details: null,
      })
      .mockResolvedValueOnce(ready);
    render(<App />);
    await connected();
    fireEvent.click(screen.getByRole("button", { name: "Inicializar audio" }));
    await screen.findByRole("alert");
    await waitFor(() =>
      expect(
        (
          screen.getByRole("button", {
            name: "Inicializar audio",
          }) as HTMLButtonElement
        ).disabled,
      ).toBe(false),
    );
    fireEvent.click(screen.getByRole("button", { name: "Inicializar audio" }));
    await waitFor(() =>
      expect(
        (screen.getByRole("button", { name: "Play" }) as HTMLButtonElement)
          .disabled,
      ).toBe(false),
    );
    expect(screen.queryByRole("alert")).toBeNull();
    expect(screen.getByTestId("transport-state").textContent).toBe("Detenido");
    expect(client.play).not.toHaveBeenCalled();
  });
  it("keeps browser mode native-free and does not initialize from an effect", async () => {
    client.available.mockReturnValue(false);
    render(<App />);
    expect(
      screen.getByText("Vista de interfaz: requiere Tauri para audio"),
    ).toBeTruthy();
    expect(
      (
        screen.getByRole("button", {
          name: "Inicializar audio",
        }) as HTMLButtonElement
      ).disabled,
    ).toBe(true);
    expect(client.snapshot).not.toHaveBeenCalled();
    expect(client.snapshots).not.toHaveBeenCalled();
    expect(client.initialize).not.toHaveBeenCalled();
  });

  it("shows initialization errors without enabling transport", async () => {
    client.initialize.mockRejectedValue({
      code: "deviceAbsent",
      message: "No output device",
      details: null,
    });
    render(<App />);
    await connected();
    expect(client.initialize).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole("button", { name: "Inicializar audio" }));
    expect((await screen.findByRole("alert")).textContent).toContain(
      "No output device",
    );
    expect(
      (screen.getByRole("button", { name: "Play" }) as HTMLButtonElement)
        .disabled,
    ).toBe(true);
    expect(screen.getByTestId("transport-state").textContent).toBe("Sin motor");
  });

  it("shows accepted commands as pending until the applied snapshot arrives", async () => {
    client.snapshot.mockResolvedValue(ready);
    client.play.mockResolvedValue({ commandId: "9" });
    render(<App />);
    await connected();
    fireEvent.click(screen.getByRole("button", { name: "Play" }));
    await screen.findByText(/Comando 9 aceptado/);
    expect(screen.getByTestId("transport-state").textContent).toBe("Detenido");
    emit({
      ...ready,
      sequence: "3",
      lastAppliedCommand: "9",
      engine: { ...ready.engine!, transport: "playing" },
    });
    expect(screen.getByTestId("transport-state").textContent).toBe(
      "Reproduciendo",
    );
    expect(screen.getByText("Sin comandos pendientes")).toBeTruthy();
  });

  it("handles application before the acceptance response", async () => {
    client.snapshot.mockResolvedValue(ready);
    let resolve!: (value: { commandId: string }) => void;
    client.play.mockImplementation(
      () =>
        new Promise((done) => {
          resolve = done;
        }),
    );
    render(<App />);
    await connected();
    fireEvent.click(screen.getByRole("button", { name: "Play" }));
    emit({
      ...ready,
      sequence: "3",
      lastAppliedCommand: "9",
      engine: { ...ready.engine!, transport: "playing" },
    });
    await act(async () => {
      resolve({ commandId: "9" });
    });
    expect(screen.getByText("Sin comandos pendientes")).toBeTruthy();
  });

  it("clears pending state if the stream fails before acceptance returns", async () => {
    client.snapshot.mockResolvedValue(ready);
    let resolve!: (value: { commandId: string }) => void;
    client.play.mockImplementation(
      () =>
        new Promise((done) => {
          resolve = done;
        }),
    );
    render(<App />);
    await connected();
    fireEvent.click(screen.getByRole("button", { name: "Play" }));
    emit({
      ...ready,
      sequence: "3",
      state: "error",
      error: { code: "streamError", message: "Stream failed", details: null },
    });
    await act(async () => {
      resolve({ commandId: "9" });
    });
    expect(screen.getByText("Sin comandos pendientes")).toBeTruthy();
    expect(screen.getByRole("alert").textContent).toContain("Stream failed");
  });

  it("ignores old initial replies and events using lossless decimal sequences", async () => {
    let resolve!: (value: ServiceSnapshotDto) => void;
    client.snapshot.mockImplementation(
      () =>
        new Promise((done) => {
          resolve = done;
        }),
    );
    render(<App />);
    await waitFor(() => expect(client.snapshot).toHaveBeenCalled());
    emit({
      ...ready,
      sequence: "9007199254740993",
      engine: { ...ready.engine!, transport: "playing" },
    });
    await act(async () => {
      resolve({ ...ready, sequence: "9007199254740992" });
    });
    emit({ ...ready, sequence: "9007199254740992" });
    expect(screen.getByTestId("transport-state").textContent).toBe(
      "Reproduciendo",
    );
  });

  it("releases listeners when StrictMode registration completes after unmount", async () => {
    const registrations: Array<(release: () => void) => void> = [];
    client.snapshots.mockImplementation(
      () =>
        new Promise((resolve) => {
          registrations.push(resolve);
        }),
    );
    const view = render(
      <StrictMode>
        <App />
      </StrictMode>,
    );
    expect(registrations).toHaveLength(2);
    view.unmount();
    const releases = registrations.map(() => vi.fn());
    await act(async () => {
      registrations.forEach((resolve, index) => resolve(releases[index]));
    });
    releases.forEach((release) => expect(release).toHaveBeenCalledTimes(1));
    expect(client.snapshot).not.toHaveBeenCalled();
    expect(client.initialize).not.toHaveBeenCalled();
  });

  it("removes an active listener on unmount", async () => {
    const view = render(<App />);
    await connected();
    expect(listeners.size).toBe(1);
    view.unmount();
    expect(listeners.size).toBe(0);
  });
});
