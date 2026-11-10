import { invoke, isTauri } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type {
  ApiError,
  AppInfoDto,
  ProjectSummaryDto,
  OutputDeviceDto,
  ServiceSnapshotDto,
  CommandAcceptedDto,
  PreparationSnapshotDto,
  SampleFileDto,
} from "./generated/api";

export function apiError(error: unknown): ApiError {
  if (
    typeof error === "object" &&
    error !== null &&
    "code" in error &&
    "message" in error
  )
    return error as ApiError;
  return {
    code: "ipcError",
    message: error instanceof Error ? error.message : String(error),
    details: null,
  };
}
async function call<T>(
  command: string,
  args?: Record<string, unknown>,
): Promise<T> {
  if (!isTauri())
    throw {
      code: "nativeAudioDisabled",
      message: "Requiere Tauri para audio",
      details: null,
    } satisfies ApiError;
  try {
    return await invoke<T>(command, args);
  } catch (error) {
    throw apiError(error);
  }
}
export const desktopClient = {
  available: isTauri,
  appInfo: () => call<AppInfoDto>("get_app_info"),
  project: () => call<ProjectSummaryDto>("get_project_summary"),
  devices: () => call<OutputDeviceDto[]>("list_output_devices"),
  initialize: (
    deviceId: string | null,
    sineFrequencyHz: number | null = null,
  ) =>
    call<ServiceSnapshotDto>("initialize_audio", { deviceId, sineFrequencyHz }),
  shutdown: () => call<void>("shutdown_audio"),
  snapshot: () => call<ServiceSnapshotDto>("get_engine_snapshot"),
  play: () => call<CommandAcceptedDto>("transport_play"),
  pause: () => call<CommandAcceptedDto>("transport_pause"),
  stop: () => call<CommandAcceptedDto>("transport_stop"),
  tempo: (bpm: number) =>
    call<CommandAcceptedDto>("transport_set_tempo", { bpm }),
  gain: (gain: number) => call<CommandAcceptedDto>("signal_set_gain", { gain }),
  prepareSamples: (
    sampleRate: number,
    channels: number,
    files: SampleFileDto[],
  ) =>
    call<PreparationSnapshotDto>("prepare_samples", {
      sampleRate,
      channels,
      files,
    }),
  preparation: () => call<PreparationSnapshotDto>("get_sample_preparation"),
  cancelPreparation: (preparationId: string) =>
    call<PreparationSnapshotDto>("cancel_sample_preparation", {
      preparationId,
    }),
  initializeSampler: (preparationId: string, deviceId: string | null) =>
    call<ServiceSnapshotDto>("initialize_sampler", { preparationId, deviceId }),
  triggerSample: (slot: number, gain: number) =>
    call<CommandAcceptedDto>("trigger_sample", { slot, gain }),
  async snapshots(callback: (snapshot: ServiceSnapshotDto) => void) {
    if (!isTauri())
      throw {
        code: "nativeAudioDisabled",
        message: "Requiere Tauri para eventos",
        details: null,
      } satisfies ApiError;
    try {
      return await listen<ServiceSnapshotDto>("engine-snapshot", (event) =>
        callback(event.payload),
      );
    } catch (error) {
      throw apiError(error);
    }
  },
};
