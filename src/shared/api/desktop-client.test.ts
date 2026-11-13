import { beforeEach, expect, it, vi } from "vitest";
const transport = vi.hoisted(() => ({
  invoke: vi.fn(),
  isTauri: vi.fn(),
  listen: vi.fn(),
}));
vi.mock("@tauri-apps/api/core", () => ({
  invoke: transport.invoke,
  isTauri: transport.isTauri,
}));
vi.mock("@tauri-apps/api/event", () => ({ listen: transport.listen }));
import { desktopClient } from "./desktop-client";
beforeEach(() => {
  vi.resetAllMocks();
  transport.isTauri.mockReturnValue(true);
  transport.invoke.mockResolvedValue(undefined);
});
it("uses scoped project commands and preserves export options as a single typed request", async () => {
  await desktopClient.document();
  expect(transport.invoke).toHaveBeenLastCalledWith(
    "get_project_document",
    undefined,
  );
  await desktopClient.saveProject("C:\\beats\\a.json", false, "Beat");
  expect(transport.invoke).toHaveBeenLastCalledWith("save_project", {
    path: "C:\\beats\\a.json",
    overwrite: false,
    title: "Beat",
  });
  await desktopClient.openProject("a.json");
  expect(transport.invoke).toHaveBeenLastCalledWith("open_project", {
    path: "a.json",
  });
  await desktopClient.prepareProjectSamples(48000, 2);
  expect(transport.invoke).toHaveBeenLastCalledWith("prepare_project_samples", {
    sampleRate: 48000,
    channels: 2,
  });
  const request = {
    outputPath: "beat.wav",
    sampleRate: 48000,
    channels: 2,
    bars: 3,
    blockSize: 127,
    tail: "finish" as const,
  };
  await desktopClient.exportBeat(request);
  expect(transport.invoke).toHaveBeenLastCalledWith("export_project_beat", {
    request,
  });
});
it("uses Tauri command names, camelCase arguments and the snapshot event", async () => {
  await desktopClient.initialize("device-id");
  expect(transport.invoke).toHaveBeenLastCalledWith("initialize_audio", {
    deviceId: "device-id",
    sineFrequencyHz: null,
  });
  await desktopClient.initialize(null, 997);
  expect(transport.invoke).toHaveBeenLastCalledWith("initialize_audio", {
    deviceId: null,
    sineFrequencyHz: 997,
  });
  await desktopClient.gain(0.25);
  expect(transport.invoke).toHaveBeenLastCalledWith("signal_set_gain", {
    gain: 0.25,
  });
  await desktopClient.tempo(123.5);
  expect(transport.invoke).toHaveBeenLastCalledWith("transport_set_tempo", {
    bpm: 123.5,
  });
  await desktopClient.snapshots(vi.fn());
  expect(transport.listen).toHaveBeenCalledWith(
    "engine-snapshot",
    expect.any(Function),
  );
  const files = [{ slot: 3, path: "C:\\samples\\snare.wav" }];
  await desktopClient.prepareSamples(48000, 2, files);
  expect(transport.invoke).toHaveBeenLastCalledWith("prepare_samples", {
    sampleRate: 48000,
    channels: 2,
    files,
  });
  await desktopClient.preparation();
  expect(transport.invoke).toHaveBeenLastCalledWith(
    "get_sample_preparation",
    undefined,
  );
  await desktopClient.cancelPreparation("9007199254740993");
  expect(transport.invoke).toHaveBeenLastCalledWith(
    "cancel_sample_preparation",
    { preparationId: "9007199254740993" },
  );
  await desktopClient.initializeSampler("9007199254740994", "device-id");
  expect(transport.invoke).toHaveBeenLastCalledWith("initialize_sampler", {
    preparationId: "9007199254740994",
    deviceId: "device-id",
  });
  await desktopClient.triggerSample(3, 0.5);
  expect(transport.invoke).toHaveBeenLastCalledWith("trigger_sample", {
    slot: 3,
    gain: 0.5,
  });
  await desktopClient.mixer();
  expect(transport.invoke).toHaveBeenLastCalledWith(
    "get_mixer_settings",
    undefined,
  );
  await desktopClient.mixerChannel(3, 0.25, -1, true);
  expect(transport.invoke).toHaveBeenLastCalledWith("mixer_set_channel", {
    channel: 3,
    gain: 0.25,
    pan: -1,
    muted: true,
  });
  await desktopClient.mixerRoute(15, 2);
  expect(transport.invoke).toHaveBeenLastCalledWith("mixer_set_route", {
    slot: 15,
    channel: 2,
  });
  await desktopClient.initializeMixedSampler(
    "9007199254740995",
    "headphones-id",
  );
  expect(transport.invoke).toHaveBeenLastCalledWith(
    "initialize_mixed_sampler",
    { preparationId: "9007199254740995", deviceId: "headphones-id" },
  );
});
it("preserves structured errors and normalizes string IPC rejections", async () => {
  transport.invoke.mockRejectedValueOnce({
    code: "queueFull",
    message: "Queue full",
    details: null,
  });
  await expect(desktopClient.play()).rejects.toMatchObject({
    code: "queueFull",
  });
  transport.invoke.mockRejectedValueOnce("invalid invoke argument");
  await expect(desktopClient.play()).rejects.toMatchObject({
    code: "ipcError",
    message: "invalid invoke argument",
  });
});
it("rejects native calls in browser mode without invoking Tauri", async () => {
  transport.isTauri.mockReturnValue(false);
  await expect(desktopClient.play()).rejects.toMatchObject({
    code: "nativeAudioDisabled",
  });
  expect(transport.invoke).not.toHaveBeenCalled();
});

it("passes pattern payloads and exact preparation identifiers to the new commands", async () => {
  const pattern = {
    steps: Array.from({ length: 4 }, () => Array(16).fill(false)),
    slots: [0, 1, 2, 3],
  } as Parameters<typeof desktopClient.setPattern>[0];
  await desktopClient.pattern();
  expect(transport.invoke).toHaveBeenLastCalledWith("get_pattern", undefined);
  await desktopClient.setPattern(pattern);
  expect(transport.invoke).toHaveBeenLastCalledWith("sequencer_set_pattern", {
    pattern,
  });
  await desktopClient.patternStep(3, 15, true);
  expect(transport.invoke).toHaveBeenLastCalledWith("sequencer_set_step", {
    row: 3,
    step: 15,
    enabled: true,
  });
  await desktopClient.patternRowSlot(3, 15);
  expect(transport.invoke).toHaveBeenLastCalledWith("sequencer_set_row_slot", {
    row: 3,
    slot: 15,
  });
  await desktopClient.initializeSequencedSampler(
    "9007199254740993",
    "device-id",
  );
  expect(transport.invoke).toHaveBeenLastCalledWith(
    "initialize_sequenced_sampler",
    { preparationId: "9007199254740993", deviceId: "device-id" },
  );
  transport.isTauri.mockReturnValue(false);
  await expect(desktopClient.pattern()).rejects.toMatchObject({
    code: "nativeAudioDisabled",
  });
});
