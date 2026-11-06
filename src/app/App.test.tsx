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
import type { ServiceSnapshotDto } from "../shared/api/generated/api";

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
  },
};
const listeners = new Set<(value: ServiceSnapshotDto) => void>();
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
  client.snapshots.mockImplementation(async (callback) => {
    listeners.add(callback);
    return () => {
      listeners.delete(callback);
    };
  });
});
afterEach(cleanup);

describe("audio interface", () => {
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
