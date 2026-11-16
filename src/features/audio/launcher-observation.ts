import type {
  LauncherLaunchDto,
  LauncherProgressDto,
  ServiceSnapshotDto,
} from "../../shared/api/generated/api";

export type LauncherObservation = {
  receipt: LauncherLaunchDto;
  progress: LauncherProgressDto;
};

// A watermark establishes processing, never a particular outcome or transition.
export function observeLauncher(
  snapshot: ServiceSnapshotDto | null,
  previous: LauncherObservation,
): LauncherObservation {
  if (previous.progress.status !== "accepted") return previous;
  if (snapshot?.state !== "ready" || !snapshot.launcher)
    return { ...previous, progress: { status: "unknown" } };
  const report = snapshot.launcher.lastRequest;
  const id = previous.receipt.accepted.commandId;
  if (report?.commandId === id) return { ...previous, progress: report.result };
  if (
    snapshot.lastAppliedCommand !== null &&
    BigInt(snapshot.lastAppliedCommand) >= BigInt(id)
  )
    return { ...previous, progress: { status: "unknown" } };
  return previous;
}
