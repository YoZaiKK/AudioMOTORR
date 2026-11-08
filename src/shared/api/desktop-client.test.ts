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
