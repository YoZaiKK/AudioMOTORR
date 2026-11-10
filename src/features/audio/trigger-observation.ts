import type {
  ServiceSnapshotDto,
  TriggerOutcomeDto,
} from "../../shared/api/generated/api";

export type TriggerObservation = { commandId: string } & (
  { status: "pending" | "unknown" } | TriggerOutcomeDto
);

// A cumulative acknowledgement confirms processing, never a particular outcome.
export function observeTrigger(
  snapshot: ServiceSnapshotDto | null,
  commandId: string,
): TriggerObservation {
  const exact = snapshot?.sampler?.lastTrigger;
  if (exact?.commandId === commandId) return { commandId, ...exact.outcome };
  if (
    snapshot?.state !== "ready" ||
    !snapshot.sampler ||
    (snapshot.lastAppliedCommand != null &&
      BigInt(snapshot.lastAppliedCommand) >= BigInt(commandId))
  ) {
    return { commandId, status: "unknown" };
  }
  return { commandId, status: "pending" };
}
