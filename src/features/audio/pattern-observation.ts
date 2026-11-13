import type { ServiceSnapshotDto } from "../../shared/api/generated/api";

export type PatternObservation = {
  commandId: string;
  status: "pending" | "processed" | "unknown";
};

// Confirmation describes command processing; values and position remain observed.
// A new session cannot confirm an old edit once its application became unknown.
export function observePattern(
  snapshot: ServiceSnapshotDto | null,
  observation: PatternObservation,
): PatternObservation {
  if (observation.status !== "pending") return observation;
  if (snapshot?.state !== "ready" || !snapshot.sequencer)
    return { ...observation, status: "unknown" };
  if (
    snapshot.lastAppliedCommand != null &&
    BigInt(snapshot.lastAppliedCommand) >= BigInt(observation.commandId)
  )
    return { ...observation, status: "processed" };
  return observation;
}
