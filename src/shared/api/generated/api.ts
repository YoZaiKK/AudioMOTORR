// Generated from Rust DTOs by ts-rs. Run npm run bindings:generate.
export type ErrorCodeDto = "invalidArgument" | "uninitialized" | "queueFull" | "busy" | "deviceAbsent" | "unsupportedSampleFormat" | "unsupportedChannels" | "nativeAudioDisabled" | "streamError" | "timeout" | "serviceClosed" | "ipcError";
export type ApiError = { code: ErrorCodeDto, message: string, details: string | null, };
export type AppInfoDto = { name: string, version: string, };
export type ProjectSummaryDto = { id: string, title: string, tempo: number, };
export type CommandAcceptedDto = { commandId: string, };
export type ServiceStateDto = "uninitialized" | "ready" | "error" | "shuttingDown";
export type TransportStateDto = "stopped" | "playing" | "paused";
export type BackendDto = "manual" | "native";
export type EngineSnapshotDto = { sampleRate: number, channels: number, tempo: number, transport: TransportStateDto, deviceFrames: string, transportFrames: string, };
export type OutputSelectionDto = { deviceId: string, name: string, sampleFormat: string, };
export type ServiceSnapshotDto = { sequence: string, state: ServiceStateDto, backend: BackendDto | null, output: OutputSelectionDto | null, engine: EngineSnapshotDto | null, callbacks: string, lastAppliedCommand: string | null, error: ApiError | null, };
export type OutputConfigDto = { channels: number, minSampleRate: number, maxSampleRate: number, sampleFormat: string, };
export type OutputDeviceDto = { id: string, name: string, isDefault: boolean, configs: Array<OutputConfigDto>, };
