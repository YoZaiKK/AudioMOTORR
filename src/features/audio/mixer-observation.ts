import type { ServiceSnapshotDto } from "../../shared/api/generated/api";

export type MixerObservation = {
  commandId: string;
  status: "pending" | "processed" | "unknown";
};

// A FIFO watermark confirms processing, not the current target or ramp endpoint.
// Once a session is lost, a later session cannot confirm the old edit.
export function observeMixer(
  snapshot: ServiceSnapshotDto | null,
  observation: MixerObservation,
): MixerObservation {
  if (observation.status !== "pending") return observation;
  if (snapshot?.state !== "ready" || !snapshot.mixer)
    return { ...observation, status: "unknown" };
  if (
    snapshot.lastAppliedCommand != null &&
    BigInt(snapshot.lastAppliedCommand) >= BigInt(observation.commandId)
  )
    return { ...observation, status: "processed" };
  return observation;
}
