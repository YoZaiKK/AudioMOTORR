import { useCallback, useEffect, useRef, useState } from "react";
import { apiError, desktopClient } from "../../shared/api/desktop-client";
import type {
  ApiError,
  AppInfoDto,
  CommandAcceptedDto,
  OutputDeviceDto,
  ProjectSummaryDto,
  ServiceSnapshotDto,
} from "../../shared/api/generated/api";

function acknowledged(snapshot: ServiceSnapshotDto | null, id: string) {
  return (
    snapshot?.lastAppliedCommand != null &&
    BigInt(snapshot.lastAppliedCommand) >= BigInt(id)
  );
}
export function useDawSession() {
  const native = desktopClient.available();
  const [info, setInfo] = useState<AppInfoDto | null>(null);
  const [project, setProject] = useState<ProjectSummaryDto | null>(null);
  const [devices, setDevices] = useState<OutputDeviceDto[]>([]);
  const [snapshot, setSnapshot] = useState<ServiceSnapshotDto | null>(null);
  const [error, setError] = useState<ApiError | null>(null);
  const [connected, setConnected] = useState(false);
  const [busy, setBusy] = useState(false);
  const [pending, setPending] = useState<string | null>(null);
  const current = useRef<ServiceSnapshotDto | null>(null);
  const mounted = useRef(false);
  const working = useRef(false);
  const observe = useCallback((next: ServiceSnapshotDto) => {
    if (
      current.current &&
      BigInt(next.sequence) <= BigInt(current.current.sequence)
    )
      return;
    current.current = next;
    setSnapshot(next);
    setPending((id) =>
      next.state !== "ready" || (id && acknowledged(next, id)) ? null : id,
    );
  }, []);
  useEffect(() => {
    mounted.current = true;
    let disposed = false;
    let unlisten: (() => void) | undefined;
    if (native)
      void (async () => {
        try {
          const release = await desktopClient.snapshots((next) => {
            if (!disposed) observe(next);
          });
          if (disposed) {
            release();
            return;
          }
          unlisten = release;
          const initial = await desktopClient.snapshot();
          if (disposed) return;
          observe(initial);
          const [appInfo, summary] = await Promise.all([
            desktopClient.appInfo(),
            desktopClient.project(),
          ]);
          if (disposed) return;
          setInfo(appInfo);
          setProject(summary);
          setConnected(true);
          const outputs = await desktopClient.devices();
          if (!disposed) setDevices(outputs);
        } catch (reason) {
          if (!disposed) setError(apiError(reason));
        }
      })();
    return () => {
      disposed = true;
      mounted.current = false;
      unlisten?.();
    };
  }, [native, observe]);
  const run = useCallback(
    async <T>(operation: () => Promise<T>, success: (value: T) => void) => {
      if (!native || working.current || !mounted.current) return;
      working.current = true;
      setBusy(true);
      setError(null);
      try {
        const result = await operation();
        if (mounted.current) success(result);
      } catch (reason) {
        if (mounted.current) setError(apiError(reason));
      } finally {
        working.current = false;
        if (mounted.current) setBusy(false);
      }
    },
    [native],
  );
  const accepted = useCallback((command: CommandAcceptedDto) => {
    setPending(
      current.current?.state === "ready" &&
        !acknowledged(current.current, command.commandId)
        ? command.commandId
        : null,
    );
  }, []);
  const command = (action: () => Promise<CommandAcceptedDto>) =>
    run(action, accepted);
  const ready = connected && snapshot?.state === "ready";
  return {
    native,
    connected,
    info,
    project,
    devices,
    snapshot,
    busy,
    pending,
    ready,
    error: error ?? snapshot?.error ?? null,
    refresh: () => run(desktopClient.devices, setDevices),
    initialize: (
      deviceId: string | null,
      sineFrequencyHz: number | null = null,
    ) =>
      run(() => desktopClient.initialize(deviceId, sineFrequencyHz), observe),
    shutdown: () =>
      run(async () => {
        await desktopClient.shutdown();
        return desktopClient.snapshot();
      }, observe),
    play: () => command(desktopClient.play),
    pause: () => command(desktopClient.pause),
    stop: () => command(desktopClient.stop),
    setTempo: (bpm: number) => command(() => desktopClient.tempo(bpm)),
    setGain: (gain: number) => command(() => desktopClient.gain(gain)),
  };
}
export type DawSession = ReturnType<typeof useDawSession>;
