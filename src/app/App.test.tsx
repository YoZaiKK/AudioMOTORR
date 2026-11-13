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
  MixerSettingsDto,
  MixerEditDto,
  PatternDto,
  ProjectDocumentDto,
} from "../shared/api/generated/api";

const client = vi.hoisted(() => ({
  available: vi.fn(),
  appInfo: vi.fn(),
  project: vi.fn(),
  document: vi.fn(),
  saveProject: vi.fn(),
  openProject: vi.fn(),
  prepareProjectSamples: vi.fn(),
  exportBeat: vi.fn(),
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
  mixer: vi.fn(),
  mixerChannel: vi.fn(),
  mixerRoute: vi.fn(),
  initializeMixedSampler: vi.fn(),
  pattern: vi.fn(),
  setPattern: vi.fn(),
  patternStep: vi.fn(),
  patternRowSlot: vi.fn(),
  initializeSequencedSampler: vi.fn(),
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
  mixer: null,
  sequencer: null,
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
function emptyPattern(): PatternDto {
  return {
    steps: Array.from({ length: 4 }, () =>
      Array(16).fill(false),
    ) as PatternDto["steps"],
    slots: [0, 1, 2, 3],
  };
}
function sequenceReady(
  sequence = "1",
  pattern = emptyPattern(),
): ServiceSnapshotDto {
  return {
    ...mixedReady,
    sequence,
    sequencer: {
      pattern,
      musicalFrames: "0",
      loopIndex: "0",
      step: 0,
      phaseNumerator: "0",
      phaseDenominator: "202661983231672320000",
      lastBoundary: null,
      lastEvent: null,
      counters: {
        boundaries: "0",
        attempted: "0",
        started: "0",
        missingAsset: "0",
        poolFull: "0",
      },
    },
  };
}
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
const mixerSettings: MixerSettingsDto = {
  channels: [
    { gain: 1, pan: 0, muted: false },
    { gain: 1, pan: 0, muted: false },
    { gain: 1, pan: 0, muted: false },
    { gain: 1, pan: 0, muted: false },
  ],
  routes: [0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3],
};
const mixedReady: ServiceSnapshotDto = {
  ...samplerReady,
  mixer: {
    settings: mixerSettings,
    channels: [0, 1, 2, 3].map(() => ({
      effectiveGain: 1,
      left: 1,
      right: 1,
      gainRemainingFrames: 0,
      panRemainingFrames: 0,
    })) as NonNullable<ServiceSnapshotDto["mixer"]>["channels"],
    voiceChannels: Array(16).fill(null) as NonNullable<
      ServiceSnapshotDto["mixer"]
    >["voiceChannels"],
    meters: {
      sequence: "0",
      frames: "0",
      channels: [
        { left: 0, right: 0 },
        { left: 0, right: 0 },
        { left: 0, right: 0 },
        { left: 0, right: 0 },
      ],
      master: { left: 0, right: 0 },
    },
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
  client.document.mockResolvedValue({
    path: null,
    dirty: true,
    summary: { id: "1", title: "Proyecto sin título", tempo: 120 },
    pattern: emptyPattern(),
    mixer: mixerSettings,
    assets: Array(16).fill(null),
  });
  client.snapshot.mockResolvedValue(empty);
  client.mixer.mockResolvedValue(mixerSettings);
  client.pattern.mockResolvedValue(emptyPattern());
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

function documentFixture(): ProjectDocumentDto {
  return {
    path: null,
    dirty: true,
    summary: { id: "1", title: "Proyecto sin título", tempo: 120 },
    pattern: emptyPattern(),
    mixer: structuredClone(mixerSettings),
    assets: Array(16).fill(null) as ProjectDocumentDto["assets"],
  };
}
describe("project persistence and beat export", () => {
  it("saves title/path explicitly and only replaces the current saved destination automatically", async () => {
    const saved = {
      ...documentFixture(),
      path: "C:\\beats\\one.json",
      dirty: false,
      summary: { id: "9007199254740993", title: "Mi beat", tempo: 120 },
    };
    client.saveProject.mockResolvedValue(saved);
    render(<App />);
    await connected();
    fireEvent.change(screen.getByLabelText("Archivo de proyecto"), {
      target: { value: saved.path },
    });
    fireEvent.change(screen.getByLabelText("Título del proyecto"), {
      target: { value: "Mi beat" },
    });
    fireEvent.change(screen.getByLabelText("Archivo WAV del beat"), {
      target: { value: "chosen.wav" },
    });
    fireEvent.change(screen.getByLabelText("Compases a exportar"), {
      target: { value: "3" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Guardar proyecto" }));
    await waitFor(() =>
      expect(client.saveProject).toHaveBeenCalledWith(
        saved.path,
        false,
        "Mi beat",
      ),
    );
    await waitFor(() =>
      expect(screen.getByTestId("project-file-state").textContent).toBe(
        "Proyecto guardado",
      ),
    );
    expect(
      (screen.getByLabelText("Archivo WAV del beat") as HTMLInputElement).value,
    ).toBe("chosen.wav");
    expect(
      (screen.getByLabelText("Compases a exportar") as HTMLInputElement).value,
    ).toBe("3");
    fireEvent.click(screen.getByRole("button", { name: "Guardar proyecto" }));
    await waitFor(() =>
      expect(client.saveProject).toHaveBeenLastCalledWith(
        saved.path,
        true,
        "Mi beat",
      ),
    );
    expect(client.play).not.toHaveBeenCalled();
    expect(client.initializeSequencedSampler).not.toHaveBeenCalled();
  });
  it("opens one selected document atomically and prepares its assets only on an explicit click", async () => {
    const opened = documentFixture();
    opened.path = "C:\\beats\\saved.json";
    opened.dirty = false;
    opened.summary.title = "Beat abierto";
    opened.pattern.steps[2][15] = true;
    opened.assets[2] = "assets/hat.wav";
    opened.mixer.channels[2].muted = true;
    client.openProject.mockResolvedValue(opened);
    client.prepareProjectSamples.mockResolvedValue(prepared);
    render(<App />);
    await connected();
    fireEvent.change(screen.getByLabelText("Archivo de proyecto"), {
      target: { value: opened.path },
    });
    fireEvent.click(screen.getByRole("button", { name: "Abrir proyecto" }));
    await waitFor(() =>
      expect(
        screen
          .getByRole("button", { name: "Fila 3, paso 16" })
          .getAttribute("aria-pressed"),
      ).toBe("true"),
    );
    expect(
      (screen.getByLabelText("Título del proyecto") as HTMLInputElement).value,
    ).toBe("Beat abierto");
    expect(client.prepareProjectSamples).not.toHaveBeenCalled();
    expect(client.play).not.toHaveBeenCalled();
    fireEvent.click(
      screen.getByRole("button", { name: "Preparar samples del proyecto" }),
    );
    await waitFor(() =>
      expect(client.prepareProjectSamples).toHaveBeenCalledWith(48000, 2),
    );
    await waitFor(() =>
      expect(
        screen
          .getByRole("button", { name: "Abrir proyecto" })
          .hasAttribute("disabled"),
      ).toBe(true),
    );
    expect(client.initializeSequencedSampler).not.toHaveBeenCalled();
  });
  it("exports exact string frame counts without changing transport and disables opening during playback", async () => {
    const playing = sequenceReady();
    playing.engine = {
      ...playing.engine!,
      transport: "playing",
      requestedTransport: "playing",
    };
    client.snapshot.mockResolvedValue(playing);
    client.exportBeat.mockResolvedValue({
      musicalFrames: "9007199254740993",
      tailFrames: "7",
      totalFrames: "9007199254741000",
      counters: {
        boundaries: "16",
        attempted: "18",
        started: "18",
        missingAsset: "0",
        poolFull: "0",
      },
    });
    render(<App />);
    await connected();
    expect(
      screen
        .getByRole("button", { name: "Abrir proyecto" })
        .hasAttribute("disabled"),
    ).toBe(true);
    fireEvent.change(screen.getByLabelText("Archivo WAV del beat"), {
      target: { value: "C:\\beats\\one.wav" },
    });
    fireEvent.change(screen.getByLabelText("Compases a exportar"), {
      target: { value: "2" },
    });
    fireEvent.change(screen.getByLabelText("Final del beat"), {
      target: { value: "cut" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Exportar beat" }));
    await waitFor(() =>
      expect(client.exportBeat).toHaveBeenCalledWith({
        outputPath: "C:\\beats\\one.wav",
        sampleRate: 48000,
        channels: 2,
        bars: 2,
        blockSize: 256,
        tail: "cut",
      }),
    );
    await waitFor(() =>
      expect(screen.getByTestId("beat-export-result").textContent).toContain(
        "9007199254741000",
      ),
    );
    expect(client.stop).not.toHaveBeenCalled();
    expect(client.pause).not.toHaveBeenCalled();
  });
  it("preserves visible selection on failed save/open/export and does not invent output", async () => {
    client.saveProject.mockRejectedValue({
      code: "invalidArgument",
      message: "destination exists",
    });
    client.openProject.mockRejectedValue({
      code: "invalidArgument",
      message: "missing asset",
    });
    client.exportBeat.mockRejectedValue({
      code: "invalidArgument",
      message: "WAV already exists",
    });
    render(<App />);
    await connected();
    fireEvent.click(screen.getByRole("button", { name: "Guardar proyecto" }));
    await waitFor(() =>
      expect(screen.getByRole("alert").textContent).toContain(
        "destination exists",
      ),
    );
    fireEvent.click(screen.getByRole("button", { name: "Abrir proyecto" }));
    await waitFor(() =>
      expect(screen.getByRole("alert").textContent).toContain("missing asset"),
    );
    expect(
      (screen.getByLabelText("Título del proyecto") as HTMLInputElement).value,
    ).toBe("Proyecto sin título");
    fireEvent.click(screen.getByRole("button", { name: "Exportar beat" }));
    await waitFor(() =>
      expect(screen.getByRole("alert").textContent).toContain(
        "WAV already exists",
      ),
    );
    expect(screen.queryByTestId("beat-export-result")).toBeNull();
  });
  it("ignores a background document reply superseded by opening another project", async () => {
    let reply!: (value: ProjectDocumentDto) => void;
    client.document
      .mockResolvedValueOnce(documentFixture())
      .mockImplementationOnce(
        () =>
          new Promise<ProjectDocumentDto>((resolve) => {
            reply = resolve;
          }),
      );
    const selected = emptyPattern();
    selected.steps[0][0] = true;
    client.patternStep.mockResolvedValue({ pattern: selected, accepted: null });
    const opened = documentFixture();
    opened.path = "new.json";
    opened.summary.title = "New document";
    client.openProject.mockResolvedValue(opened);
    render(<App />);
    await connected();
    fireEvent.click(screen.getByRole("button", { name: "Fila 1, paso 1" }));
    await waitFor(() => expect(client.document).toHaveBeenCalledTimes(2));
    fireEvent.click(screen.getByRole("button", { name: "Abrir proyecto" }));
    await waitFor(() =>
      expect(
        (screen.getByLabelText("Título del proyecto") as HTMLInputElement)
          .value,
      ).toBe("New document"),
    );
    await act(async () => reply(documentFixture()));
    expect(
      (screen.getByLabelText("Título del proyecto") as HTMLInputElement).value,
    ).toBe("New document");
    expect(
      (screen.getByLabelText("Archivo de proyecto") as HTMLInputElement).value,
    ).toBe("new.json");
  });
  it("consulting the document preserves unsaved title and destination drafts", async () => {
    render(<App />);
    await connected();
    fireEvent.change(screen.getByLabelText("Título del proyecto"), {
      target: { value: "Unsaved title" },
    });
    fireEvent.change(screen.getByLabelText("Archivo de proyecto"), {
      target: { value: "save-as.json" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Consultar proyecto" }));
    await waitFor(() => expect(client.document).toHaveBeenCalledTimes(2));
    await waitFor(() =>
      expect(
        screen
          .getByRole("button", { name: "Consultar proyecto" })
          .hasAttribute("disabled"),
      ).toBe(false),
    );
    expect(
      (screen.getByLabelText("Título del proyecto") as HTMLInputElement).value,
    ).toBe("Unsaved title");
    expect(
      (screen.getByLabelText("Archivo de proyecto") as HTMLInputElement).value,
    ).toBe("save-as.json");
  });
  it("disables native project actions in browser mode", async () => {
    client.available.mockReturnValue(false);
    render(<App />);
    for (const name of [
      "Guardar proyecto",
      "Abrir proyecto",
      "Consultar proyecto",
      "Preparar samples del proyecto",
      "Exportar beat",
    ])
      expect(
        screen.getByRole("button", { name }).hasAttribute("disabled"),
      ).toBe(true);
    expect(client.document).not.toHaveBeenCalled();
    expect(client.exportBeat).not.toHaveBeenCalled();
  });
  it("ignores startup pattern and mixer replies after an opened document replaced them", async () => {
    let oldPattern!: (value: PatternDto) => void;
    let oldMixer!: (value: MixerSettingsDto) => void;
    client.pattern.mockImplementationOnce(
      () =>
        new Promise<PatternDto>((resolve) => {
          oldPattern = resolve;
        }),
    );
    client.mixer.mockImplementationOnce(
      () =>
        new Promise<MixerSettingsDto>((resolve) => {
          oldMixer = resolve;
        }),
    );
    const opened = documentFixture();
    opened.path = "opened.json";
    opened.pattern.steps[3][15] = true;
    opened.mixer.channels[0].gain = 0.25;
    client.openProject.mockResolvedValue(opened);
    render(<App />);
    await connected();
    fireEvent.click(screen.getByRole("button", { name: "Abrir proyecto" }));
    await waitFor(() =>
      expect(
        screen
          .getByRole("button", { name: "Fila 4, paso 16" })
          .getAttribute("aria-pressed"),
      ).toBe("true"),
    );
    await act(async () => {
      oldPattern(emptyPattern());
      oldMixer(mixerSettings);
    });
    expect(
      screen
        .getByRole("button", { name: "Fila 4, paso 16" })
        .getAttribute("aria-pressed"),
    ).toBe("true");
    expect(
      (screen.getByLabelText("Archivo de proyecto") as HTMLInputElement).value,
    ).toBe("opened.json");
  });
});

describe("sixteen-step sequencer", () => {
  it("edits all four rows offline and clears the pattern with one atomic replacement", async () => {
    const selected = emptyPattern();
    selected.steps[3][15] = true;
    selected.slots[3] = 15;
    client.patternStep.mockResolvedValue({ pattern: selected, accepted: null });
    client.patternRowSlot.mockResolvedValue({
      pattern: selected,
      accepted: null,
    });
    render(<App />);
    await connected();
    const step = screen.getByRole("button", { name: "Fila 4, paso 16" });
    await waitFor(() => expect(step.hasAttribute("disabled")).toBe(false));
    expect(document.querySelectorAll(".step-grid tbody button")).toHaveLength(
      64,
    );
    fireEvent.click(step);
    await waitFor(() => expect(step.getAttribute("aria-pressed")).toBe("true"));
    expect(client.patternStep).toHaveBeenCalledWith(3, 15, true);
    expect(step.getAttribute("data-applied")).toBe("none");
    expect(screen.getByTestId("pattern-outcome").textContent).toContain(
      "sin enviar comandos",
    );
    fireEvent.change(screen.getByLabelText("Fila 4"), {
      target: { value: "15" },
    });
    await waitFor(() =>
      expect(client.patternRowSlot).toHaveBeenCalledWith(3, 15),
    );
    const cleared = structuredClone(selected);
    cleared.steps.forEach((row) => row.fill(false));
    client.setPattern.mockResolvedValue({ pattern: cleared, accepted: null });
    await waitFor(() =>
      expect(
        screen
          .getByRole("button", { name: "Vaciar patrón" })
          .hasAttribute("disabled"),
      ).toBe(false),
    );
    fireEvent.click(screen.getByRole("button", { name: "Vaciar patrón" }));
    await waitFor(() =>
      expect(step.getAttribute("aria-pressed")).toBe("false"),
    );
    expect(client.setPattern).toHaveBeenCalledWith(cleared);
    expect(client.setPattern).toHaveBeenCalledTimes(1);
    expect(client.initializeSequencedSampler).not.toHaveBeenCalled();
    expect(client.play).not.toHaveBeenCalled();
  });

  it("initializes a prepared sequenced session explicitly without Play and keeps offline receipts truthful", async () => {
    const selected = emptyPattern();
    selected.steps[0][0] = true;
    client.patternStep.mockResolvedValue({ pattern: selected, accepted: null });
    client.preparation.mockResolvedValueOnce(prepared).mockResolvedValue({
      ...prepared,
      sequence: "3",
      state: "idle",
      occupiedSlots: 0,
    });
    client.initializeSequencedSampler.mockResolvedValue(
      sequenceReady("1", selected),
    );
    render(<App />);
    await connected();
    const step = screen.getByRole("button", { name: "Fila 1, paso 1" });
    await waitFor(() => expect(step.hasAttribute("disabled")).toBe(false));
    fireEvent.click(step);
    await screen.findByTestId("pattern-outcome");
    const initialize = screen.getByRole("button", {
      name: "Inicializar secuenciador",
    });
    await waitFor(() =>
      expect(initialize.hasAttribute("disabled")).toBe(false),
    );
    fireEvent.click(initialize);
    await waitFor(() =>
      expect(client.initializeSequencedSampler).toHaveBeenCalledWith("1", null),
    );
    await waitFor(() => expect(step.getAttribute("data-applied")).toBe("true"));
    expect(screen.getByTestId("pattern-outcome").textContent).toContain(
      "sin enviar comandos",
    );
    expect(screen.getByTestId("pattern-position").textContent).toBe(
      "Sin paso observado",
    );
    expect(client.play).not.toHaveBeenCalled();
  });

  it("keeps selected and applied steps separate and uses exact observed IDs and position", async () => {
    const initial = sequenceReady();
    initial.lastAppliedCommand = "9007199254740992";
    client.snapshot.mockResolvedValue(initial);
    const selected = emptyPattern();
    selected.steps[0][0] = true;
    client.patternStep.mockResolvedValue({
      pattern: selected,
      accepted: { commandId: "9007199254740993" },
    });
    render(<App />);
    await connected();
    const step = screen.getByRole("button", { name: "Fila 1, paso 1" });
    await waitFor(() => expect(step.hasAttribute("disabled")).toBe(false));
    fireEvent.click(step);
    await waitFor(() => expect(step.getAttribute("aria-pressed")).toBe("true"));
    expect(step.getAttribute("data-applied")).toBe("false");
    expect(screen.getByTestId("pattern-outcome").textContent).toContain(
      "esperando procesamiento",
    );
    const processed = sequenceReady("9007199254740993", selected);
    processed.lastAppliedCommand = "9007199254740993";
    processed.sequencer!.lastBoundary = {
      loopIndex: "9007199254740993",
      step: 7,
      musicalFrame: "9007199254740993",
    };
    emit(processed);
    expect(step.getAttribute("data-applied")).toBe("true");
    expect(screen.getByTestId("pattern-outcome").textContent).toContain(
      "procesado;",
    );
    expect(screen.getByTestId("pattern-position").textContent).toContain(
      "8 · vuelta 9007199254740994",
    );
    expect(
      document.querySelectorAll('.step-grid [aria-current="step"]'),
    ).toHaveLength(4);
    emit({ ...initial, sequence: "9007199254740992" });
    vi.useFakeTimers();
    try {
      act(() => vi.advanceTimersByTime(10000));
    } finally {
      vi.useRealTimers();
    }
    expect(screen.getByTestId("pattern-position").textContent).toContain(
      "8 · vuelta 9007199254740994",
    );
    expect(client.play).not.toHaveBeenCalled();
  });

  it("handles a processed snapshot that arrives before the edit reply", async () => {
    client.snapshot.mockResolvedValue(sequenceReady());
    let reply!: (value: {
      pattern: PatternDto;
      accepted: { commandId: string };
    }) => void;
    client.patternStep.mockImplementation(
      () =>
        new Promise((resolve) => {
          reply = resolve;
        }),
    );
    render(<App />);
    await connected();
    const step = screen.getByRole("button", { name: "Fila 1, paso 1" });
    await waitFor(() => expect(step.hasAttribute("disabled")).toBe(false));
    fireEvent.click(step);
    const selected = emptyPattern();
    selected.steps[0][0] = true;
    emit({ ...sequenceReady("2", selected), lastAppliedCommand: "19" });
    await act(async () =>
      reply({ pattern: selected, accepted: { commandId: "19" } }),
    );
    expect(screen.getByTestId("pattern-outcome").textContent).toContain(
      "procesado;",
    );
    expect(step.getAttribute("aria-pressed")).toBe("true");
  });

  it.each([false, true])(
    "never confirms an old edit after session loss (late reply %s)",
    async (late) => {
      client.snapshot.mockResolvedValue(sequenceReady());
      const selected = emptyPattern();
      selected.steps[0][0] = true;
      let reply!: (value: {
        pattern: PatternDto;
        accepted: { commandId: string };
      }) => void;
      if (late)
        client.patternStep.mockImplementation(
          () =>
            new Promise((resolve) => {
              reply = resolve;
            }),
        );
      else
        client.patternStep.mockResolvedValue({
          pattern: selected,
          accepted: { commandId: "9" },
        });
      render(<App />);
      await connected();
      const step = screen.getByRole("button", { name: "Fila 1, paso 1" });
      await waitFor(() => expect(step.hasAttribute("disabled")).toBe(false));
      fireEvent.click(step);
      await waitFor(() => expect(client.patternStep).toHaveBeenCalled());
      if (!late) await screen.findByTestId("pattern-outcome");
      emit({ ...empty, sequence: "2" });
      emit({ ...sequenceReady("3"), lastAppliedCommand: "99" });
      if (late)
        await act(async () =>
          reply({ pattern: selected, accepted: { commandId: "9" } }),
        );
      expect(screen.getByTestId("pattern-outcome").textContent).toContain(
        "desconocida",
      );
      expect(step.getAttribute("data-applied")).toBe("false");
    },
  );

  it.each(["reply", "error"])(
    "ignores a superseded startup pattern %s after refresh and editing",
    async (outcome) => {
      let resolve!: (value: PatternDto) => void;
      let reject!: (reason: unknown) => void;
      client.pattern.mockImplementationOnce(
        () =>
          new Promise((yes, no) => {
            resolve = yes;
            reject = no;
          }),
      );
      client.pattern.mockResolvedValueOnce(emptyPattern());
      const selected = emptyPattern();
      selected.steps[2][4] = true;
      client.patternStep.mockResolvedValue({
        pattern: selected,
        accepted: null,
      });
      render(<App />);
      await connected();
      fireEvent.click(screen.getByRole("button", { name: "Consultar patrón" }));
      const step = screen.getByRole("button", { name: "Fila 3, paso 5" });
      await waitFor(() => expect(step.hasAttribute("disabled")).toBe(false));
      fireEvent.click(step);
      await waitFor(() =>
        expect(step.getAttribute("aria-pressed")).toBe("true"),
      );
      await act(async () => {
        if (outcome === "reply") resolve(emptyPattern());
        else
          reject({
            code: "timeout",
            message: "Old pattern query",
            details: null,
          });
      });
      expect(step.getAttribute("aria-pressed")).toBe("true");
      expect(screen.queryByRole("alert")).toBeNull();
    },
  );

  it("preserves selection on queue full and shows observed note rejections separately", async () => {
    client.snapshot.mockResolvedValue(sequenceReady());
    client.patternStep.mockRejectedValue({
      code: "queueFull",
      message: "Pattern queue full",
      details: null,
    });
    render(<App />);
    await connected();
    const step = screen.getByRole("button", { name: "Fila 1, paso 1" });
    await waitFor(() => expect(step.hasAttribute("disabled")).toBe(false));
    fireEvent.click(step);
    await screen.findByRole("alert");
    expect(step.getAttribute("aria-pressed")).toBe("false");
    expect(screen.queryByTestId("pattern-outcome")).toBeNull();
    const snapshot = sequenceReady("2");
    snapshot.sequencer!.counters.missingAsset = "9007199254740993";
    snapshot.sequencer!.lastEvent = {
      boundary: { loopIndex: "0", step: 0, musicalFrame: "0" },
      row: 0,
      slot: 15,
      deviceFrame: "0",
      outcome: { status: "rejected", reason: "missingAsset" },
    };
    emit(snapshot);
    expect(screen.getByTestId("pattern-counts").textContent).toContain(
      "9007199254740993 sin sample",
    );
    expect(screen.getByTestId("pattern-event").textContent).toContain(
      "rechazada: slot vacío",
    );
  });

  it("does not compensate a timed-out edit even if the engine later observes a new pattern", async () => {
    client.snapshot.mockResolvedValue(sequenceReady());
    client.patternStep.mockRejectedValue({
      code: "timeout",
      message: "Acceptance unknown",
      details: null,
    });
    render(<App />);
    await connected();
    const step = screen.getByRole("button", { name: "Fila 1, paso 1" });
    await waitFor(() => expect(step.hasAttribute("disabled")).toBe(false));
    fireEvent.click(step);
    await screen.findByRole("alert");
    const applied = emptyPattern();
    applied.steps[0][0] = true;
    emit({ ...sequenceReady("2", applied), lastAppliedCommand: "1" });
    expect(step.getAttribute("aria-pressed")).toBe("false");
    expect(step.getAttribute("data-applied")).toBe("true");
    expect(screen.queryByTestId("pattern-outcome")).toBeNull();
    expect(client.patternStep).toHaveBeenCalledTimes(1);
    expect(client.setPattern).not.toHaveBeenCalled();
  });

  it.each(["loading", "cancelling"] as const)(
    "allows document editing during preparation %s",
    async (state) => {
      client.preparation.mockResolvedValue({ ...prepared, state });
      const selected = emptyPattern();
      selected.steps[1][3] = true;
      client.patternStep.mockResolvedValue({
        pattern: selected,
        accepted: null,
      });
      render(<App />);
      await connected();
      const step = screen.getByRole("button", { name: "Fila 2, paso 4" });
      await waitFor(() => expect(step.hasAttribute("disabled")).toBe(false));
      fireEvent.click(step);
      await waitFor(() =>
        expect(step.getAttribute("aria-pressed")).toBe("true"),
      );
      expect(
        screen
          .getByRole("button", { name: "Inicializar secuenciador" })
          .hasAttribute("disabled"),
      ).toBe(true);
    },
  );

  it("disables pattern edits in browser mode and in legacy sessions", async () => {
    client.available.mockReturnValue(false);
    const view = render(<App />);
    expect(
      screen
        .getByRole("button", { name: "Fila 1, paso 1" })
        .hasAttribute("disabled"),
    ).toBe(true);
    expect(client.pattern).not.toHaveBeenCalled();
    view.unmount();
    client.available.mockReturnValue(true);
    client.snapshot.mockResolvedValue(mixedReady);
    render(<App />);
    await connected();
    expect(
      screen
        .getByRole("button", { name: "Fila 1, paso 1" })
        .hasAttribute("disabled"),
    ).toBe(true);
    expect(
      screen.getByText(/Cierra audio e inicializa el secuenciador/),
    ).toBeTruthy();
  });

  it("refreshes consumed preparation after initialization fails and preserves selected pattern", async () => {
    const selected = emptyPattern();
    selected.steps[0][0] = true;
    client.pattern.mockResolvedValue(selected);
    client.preparation.mockResolvedValueOnce(prepared).mockResolvedValue({
      ...prepared,
      sequence: "3",
      state: "idle",
      occupiedSlots: 0,
      retainedPcmBytes: "0",
    });
    client.initializeSequencedSampler.mockRejectedValue({
      code: "deviceAbsent",
      message: "Output missing",
      details: null,
    });
    render(<App />);
    await connected();
    const initialize = screen.getByRole("button", {
      name: "Inicializar secuenciador",
    });
    await waitFor(() =>
      expect(initialize.hasAttribute("disabled")).toBe(false),
    );
    fireEvent.click(initialize);
    await screen.findByRole("alert");
    await waitFor(() => expect(initialize.hasAttribute("disabled")).toBe(true));
    expect(
      screen
        .getByRole("button", { name: "Fila 1, paso 1" })
        .getAttribute("aria-pressed"),
    ).toBe("true");
    expect(client.play).not.toHaveBeenCalled();
  });
});

describe("four-channel mixer", () => {
  it("ignores a superseded startup settings error after a successful refresh", async () => {
    let startupError!: (reason: unknown) => void;
    client.mixer.mockImplementationOnce(
      () =>
        new Promise((_resolve, reject) => {
          startupError = reject;
        }),
    );
    const refreshed = structuredClone(mixerSettings);
    refreshed.channels[0].gain = 0.125;
    client.mixer.mockResolvedValueOnce(refreshed);
    render(<App />);
    await connected();
    fireEvent.click(
      screen.getByRole("button", { name: "Consultar ajustes de mezcla" }),
    );
    await waitFor(() =>
      expect(
        (screen.getByLabelText("Ganancia · Canal 1") as HTMLInputElement).value,
      ).toBe("0.125"),
    );
    await act(async () =>
      startupError({
        code: "timeout",
        message: "Superseded startup query",
        details: null,
      }),
    );
    expect(screen.queryByRole("alert")).toBeNull();
    expect(screen.getByTestId("mixer-selected-0").textContent).toContain(
      "G 0.125",
    );
  });

  it("ignores a startup settings reply superseded by refresh and an accepted edit", async () => {
    let startupReply!: (settings: MixerSettingsDto) => void;
    client.mixer.mockImplementationOnce(
      () =>
        new Promise((resolve) => {
          startupReply = resolve;
        }),
    );
    const refreshed = structuredClone(mixerSettings);
    refreshed.channels[0].gain = 0.25;
    client.mixer.mockResolvedValueOnce(refreshed);
    const edited = structuredClone(refreshed);
    edited.channels[0].gain = 0.5;
    client.mixerChannel.mockResolvedValue({ settings: edited, accepted: null });
    render(<App />);
    await connected();
    fireEvent.click(
      screen.getByRole("button", { name: "Consultar ajustes de mezcla" }),
    );
    await waitFor(() =>
      expect(
        (screen.getByLabelText("Ganancia · Canal 1") as HTMLInputElement).value,
      ).toBe("0.25"),
    );
    fireEvent.change(screen.getByLabelText("Ganancia · Canal 1"), {
      target: { value: "0.5" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Aplicar canal 1" }));
    await waitFor(() =>
      expect(client.mixerChannel).toHaveBeenCalledWith(0, 0.5, 0, false),
    );
    await screen.findByTestId("mixer-outcome");
    await act(async () => startupReply(mixerSettings));
    expect(screen.getByTestId("mixer-selected-0").textContent).toContain(
      "G 0.5",
    );
    expect(
      (screen.getByLabelText("Ganancia · Canal 1") as HTMLInputElement).value,
    ).toBe("0.5");
  });

  it("keeps an offline receipt truthful after explicit mixed initialization", async () => {
    client.preparation.mockResolvedValueOnce(prepared).mockResolvedValue({
      ...prepared,
      sequence: "3",
      state: "idle",
      preparationId: null,
    });
    const selected = structuredClone(mixerSettings);
    selected.channels[0].gain = 0.25;
    client.mixerChannel.mockResolvedValue({
      settings: selected,
      accepted: null,
    });
    const installed = structuredClone(mixedReady);
    installed.mixer!.settings = selected;
    installed.mixer!.channels[0].effectiveGain = 0.25;
    client.initializeMixedSampler.mockResolvedValue(installed);
    render(<App />);
    await connected();
    fireEvent.change(screen.getByLabelText("Ganancia · Canal 1"), {
      target: { value: "0.25" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Aplicar canal 1" }));
    await screen.findByTestId("mixer-outcome");
    fireEvent.click(
      screen.getByRole("button", { name: "Inicializar con mezcla" }),
    );
    await screen.findByText("Sampler · mezcla de cuatro canales · 16 voces");
    expect(screen.getByTestId("mixer-outcome").textContent).not.toContain(
      "audio sin inicializar",
    );
    expect(screen.getByTestId("mixer-outcome").textContent).not.toContain(
      "próxima sesión",
    );
    expect(screen.getByTestId("mixer-applied-0").textContent).toContain(
      "G 0.250",
    );
    expect(client.play).not.toHaveBeenCalled();
  });

  it("waits for selected settings and still loads them when device enumeration fails", async () => {
    let resolve!: (settings: MixerSettingsDto) => void;
    client.mixer.mockImplementation(
      () =>
        new Promise((done) => {
          resolve = done;
        }),
    );
    client.devices.mockRejectedValue({
      code: "streamError",
      message: "Devices unavailable",
      details: null,
    });
    render(<App />);
    await screen.findByRole("alert");
    expect(
      (screen.getByLabelText("Ganancia · Canal 1") as HTMLInputElement)
        .disabled,
    ).toBe(true);
    const settings = structuredClone(mixerSettings);
    settings.channels[0].gain = 0.125;
    await act(async () => resolve(settings));
    expect(
      (screen.getByLabelText("Ganancia · Canal 1") as HTMLInputElement).value,
    ).toBe("0.125");
    expect(
      (
        screen.getByLabelText("Ganancia · Canal 1") as HTMLInputElement
      ).checkValidity(),
    ).toBe(true);
    expect(
      (screen.getByLabelText("Ganancia · Canal 1") as HTMLInputElement)
        .disabled,
    ).toBe(false);
    expect(client.mixerChannel).not.toHaveBeenCalled();
  });

  it("never revives an unconfirmed edit after session loss and a later watermark", async () => {
    client.snapshot.mockResolvedValue(mixedReady);
    const selected = structuredClone(mixerSettings);
    selected.routes[0] = 3;
    client.mixerRoute.mockResolvedValue({
      settings: selected,
      accepted: { commandId: "17" },
    });
    render(<App />);
    await connected();
    fireEvent.click(screen.getByText("Rutas de los 16 slots"));
    fireEvent.change(screen.getByLabelText("Ruta · slot 0"), {
      target: { value: "3" },
    });
    await screen.findByText(/Cambio 17: aceptado/);
    emit({
      ...empty,
      sequence: "2",
      state: "error",
      error: { code: "streamError", message: "Device lost", details: null },
    });
    expect(screen.getByTestId("mixer-outcome").textContent).toContain(
      "aplicación desconocida",
    );
    emit({ ...mixedReady, sequence: "3", lastAppliedCommand: "18" });
    expect(screen.getByTestId("mixer-outcome").textContent).toContain(
      "aplicación desconocida",
    );
    expect(screen.getByTestId("mixer-selected-0").textContent).toContain("G 1");
    expect(
      (screen.getByLabelText("Ruta · slot 0") as HTMLSelectElement).value,
    ).toBe("3");
  });

  it("refreshes a consumed bank after mixed initialization fails and preserves selection", async () => {
    client.preparation.mockResolvedValueOnce(prepared).mockResolvedValue({
      ...prepared,
      sequence: "3",
      state: "idle",
      preparationId: null,
      occupiedSlots: 0,
    });
    client.initializeMixedSampler.mockRejectedValue({
      code: "deviceAbsent",
      message: "Headphones missing",
      details: null,
    });
    render(<App />);
    await connected();
    fireEvent.click(
      screen.getByRole("button", { name: "Inicializar con mezcla" }),
    );
    await screen.findByRole("alert");
    expect(screen.getByText("Sin banco preparado")).toBeTruthy();
    expect(
      (
        screen.getByRole("button", {
          name: "Inicializar con mezcla",
        }) as HTMLButtonElement
      ).disabled,
    ).toBe(true);
    expect(screen.getByTestId("mixer-selected-0").textContent).toContain("G 1");
    expect(client.play).not.toHaveBeenCalled();
    expect(client.initializeSampler).not.toHaveBeenCalled();
  });

  it("selects offline channels and routes without inventing commands or opening audio", async () => {
    const selected = structuredClone(mixerSettings);
    selected.channels[0] = { gain: 0.25, pan: -0.5, muted: true };
    client.mixerChannel.mockResolvedValue({
      settings: selected,
      accepted: null,
    });
    render(<App />);
    await waitFor(() =>
      expect(
        (screen.getByLabelText("Ganancia · Canal 1") as HTMLInputElement)
          .disabled,
      ).toBe(false),
    );
    fireEvent.change(screen.getByLabelText("Ganancia · Canal 1"), {
      target: { value: "" },
    });
    expect(
      (
        screen.getByRole("button", {
          name: "Aplicar canal 1",
        }) as HTMLButtonElement
      ).disabled,
    ).toBe(true);
    fireEvent.change(screen.getByLabelText("Ganancia · Canal 1"), {
      target: { value: "0.25" },
    });
    fireEvent.change(screen.getByLabelText("Pan · Canal 1"), {
      target: { value: "-0.5" },
    });
    fireEvent.click(screen.getByLabelText("Mute · Canal 1"));
    fireEvent.click(screen.getByRole("button", { name: "Aplicar canal 1" }));
    await screen.findByText(
      /Selección de mezcla actualizada sin enviar comandos de audio/,
    );
    expect(client.mixerChannel).toHaveBeenCalledWith(0, 0.25, -0.5, true);
    expect(screen.getByTestId("mixer-selected-0").textContent).toContain(
      "G 0.25 · P -0.5 · mute",
    );
    expect(screen.getByTestId("mixer-applied-0").textContent).toBe(
      "Sin mezcla aplicada",
    );
    const routed = structuredClone(selected);
    routed.routes[15] = 1;
    client.mixerRoute.mockResolvedValue({ settings: routed, accepted: null });
    fireEvent.click(screen.getByText("Rutas de los 16 slots"));
    fireEvent.change(screen.getByLabelText("Ruta · slot 15"), {
      target: { value: "1" },
    });
    await waitFor(() => expect(client.mixerRoute).toHaveBeenCalledWith(15, 1));
    expect(client.initializeMixedSampler).not.toHaveBeenCalled();
    expect(client.initialize).not.toHaveBeenCalled();
    expect(client.play).not.toHaveBeenCalled();
    expect(screen.queryByRole("meter")).toBeNull();
  });

  it("keeps selected settings separate until an ordered observation processes a large command ID", async () => {
    client.snapshot.mockResolvedValue(mixedReady);
    const selected = structuredClone(mixerSettings);
    selected.channels[0].gain = 0.5;
    client.mixerChannel.mockResolvedValue({
      settings: selected,
      accepted: { commandId: "9007199254740993" },
    });
    render(<App />);
    await connected();
    fireEvent.change(screen.getByLabelText("Ganancia · Canal 1"), {
      target: { value: "0.5" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Aplicar canal 1" }));
    await screen.findByText(/Cambio 9007199254740993: aceptado/);
    expect(screen.getByTestId("mixer-applied-0").textContent).toContain(
      "G 1.000",
    );
    const applied = structuredClone(mixedReady);
    applied.sequence = "9007199254740993";
    applied.lastAppliedCommand = "9007199254740993";
    applied.mixer!.settings = selected;
    applied.mixer!.channels[0] = {
      effectiveGain: 0.75,
      left: 1,
      right: 1,
      gainRemainingFrames: 120,
      panRemainingFrames: 0,
    };
    emit(applied);
    expect(screen.getByTestId("mixer-outcome").textContent).toContain(
      "procesado; consulta valores y rampas observados",
    );
    expect(screen.getByTestId("mixer-applied-0").textContent).toContain(
      "G 0.750",
    );
    expect(screen.getByTestId("mixer-applied-0").textContent).toContain(
      "rampa G 120",
    );
    emit({ ...mixedReady, sequence: "9007199254740992" });
    expect(screen.getByTestId("mixer-applied-0").textContent).toContain(
      "G 0.750",
    );
    expect(screen.getByTestId("mixer-selected-0").textContent).toContain(
      "G 0.5",
    );
  });

  it.each(["queueFull", "timeout"])(
    "retains the document and retry draft after %s",
    async (code) => {
      client.snapshot.mockResolvedValue(mixedReady);
      client.mixerChannel.mockRejectedValue({
        code,
        message: "Change failed",
        details: null,
      });
      render(<App />);
      await connected();
      fireEvent.change(screen.getByLabelText("Pan · Canal 2"), {
        target: { value: "0.5" },
      });
      fireEvent.click(screen.getByRole("button", { name: "Aplicar canal 2" }));
      await screen.findByRole("alert");
      expect(screen.getByTestId("mixer-selected-1").textContent).toContain(
        "P 0",
      );
      expect(
        (screen.getByLabelText("Pan · Canal 2") as HTMLInputElement).value,
      ).toBe("0.5");
      expect(screen.queryByTestId("mixer-outcome")).toBeNull();
      expect(
        (
          screen.getByRole("button", {
            name: "Aplicar canal 2",
          }) as HTMLButtonElement
        ).disabled,
      ).toBe(false);
    },
  );

  it.each([false, true])(
    "handles observations before the receipt, including a lost session (%s)",
    async (lost) => {
      client.snapshot.mockResolvedValue(mixedReady);
      let resolve!: (edit: MixerEditDto) => void;
      client.mixerRoute.mockImplementation(
        () =>
          new Promise((done) => {
            resolve = done;
          }),
      );
      render(<App />);
      await connected();
      fireEvent.click(screen.getByText("Rutas de los 16 slots"));
      fireEvent.change(screen.getByLabelText("Ruta · slot 0"), {
        target: { value: "3" },
      });
      const selected = structuredClone(mixerSettings);
      selected.routes[0] = 3;
      if (lost) emit({ ...empty, sequence: "2" });
      emit({
        ...mixedReady,
        sequence: "3",
        lastAppliedCommand: "9007199254740995",
      });
      await act(async () =>
        resolve({
          settings: selected,
          accepted: { commandId: "9007199254740993" },
        }),
      );
      expect(screen.getByTestId("mixer-outcome").textContent).toContain(
        lost ? "aplicación desconocida" : "procesado;",
      );
      expect(
        (screen.getByLabelText("Ruta · slot 0") as HTMLSelectElement).value,
      ).toBe("3");
      expect(screen.getAllByText("Observada: canal 1").length).toBe(4);
    },
  );

  it("shows actual whole-window peaks and omits a nonexistent mono right channel", async () => {
    client.snapshot.mockResolvedValue(mixedReady);
    render(<App />);
    await connected();
    expect(screen.queryByRole("meter")).toBeNull();
    const measured = structuredClone(mixedReady);
    measured.sequence = "2";
    measured.mixer!.meters = {
      ...measured.mixer!.meters,
      sequence: "9007199254740993",
      frames: "1001",
      master: { left: 0.25, right: 0.125 },
    };
    measured.mixer!.meters.channels[0] = { left: 0.0625, right: 0.03125 };
    emit(measured);
    expect(screen.getAllByRole("meter")).toHaveLength(10);
    expect(
      screen.getByRole("meter", { name: "Canal 1 L" }).getAttribute("value"),
    ).toBe("0.0625");
    expect(
      screen.getByRole("meter", { name: "Master R" }).getAttribute("value"),
    ).toBe("0.125");
    expect(
      screen.getByText(/1001 frames · secuencia 9007199254740993/),
    ).toBeTruthy();
    const mono = structuredClone(measured);
    mono.sequence = "3";
    mono.mixer!.meters.master.right = null;
    mono.mixer!.meters.channels.forEach((peak) => {
      peak.right = null;
    });
    mono.engine!.channels = 1;
    emit(mono);
    expect(screen.getAllByRole("meter")).toHaveLength(5);
    expect(screen.getByRole("meter", { name: "Master mono" })).toBeTruthy();
    expect(screen.queryByRole("meter", { name: "Master R" })).toBeNull();
    emit({ ...empty, sequence: "4" });
    expect(screen.queryByRole("meter")).toBeNull();
  });

  it("disables editing in a legacy session and browser preview", async () => {
    client.snapshot.mockResolvedValue(samplerReady);
    const first = render(<App />);
    await connected();
    expect(
      (screen.getByLabelText("Ganancia · Canal 1") as HTMLInputElement)
        .disabled,
    ).toBe(true);
    expect(
      (screen.getByLabelText("Ruta · slot 0") as HTMLSelectElement).disabled,
    ).toBe(true);
    first.unmount();
    vi.clearAllMocks();
    client.available.mockReturnValue(false);
    render(<App />);
    expect(
      (screen.getByLabelText("Pan · Canal 4") as HTMLInputElement).disabled,
    ).toBe(true);
    expect(client.mixer).not.toHaveBeenCalled();
    expect(client.mixerChannel).not.toHaveBeenCalled();
  });

  it("explicitly initializes a prepared mixed session without Play and refreshes the consumed bank", async () => {
    client.preparation.mockResolvedValueOnce(prepared).mockResolvedValue({
      ...prepared,
      sequence: "3",
      state: "idle",
      preparationId: null,
      occupiedSlots: 0,
    });
    client.initializeMixedSampler.mockResolvedValue(mixedReady);
    render(<App />);
    await connected();
    fireEvent.click(
      screen.getByRole("button", { name: "Inicializar con mezcla" }),
    );
    await screen.findByText("Sampler · mezcla de cuatro canales · 16 voces");
    expect(client.initializeMixedSampler).toHaveBeenCalledWith("1", null);
    expect(client.initializeSampler).not.toHaveBeenCalled();
    expect(client.play).not.toHaveBeenCalled();
    expect(client.preparation).toHaveBeenCalledTimes(2);
    expect(screen.getByText("Sin banco preparado")).toBeTruthy();
  });
});

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
