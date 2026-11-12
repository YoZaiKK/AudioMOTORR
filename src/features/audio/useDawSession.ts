import { useCallback, useEffect, useRef, useState } from "react";
import { apiError, desktopClient } from "../../shared/api/desktop-client";
import { observeTrigger, type TriggerObservation } from "./trigger-observation";
import { observeMixer, type MixerObservation } from "./mixer-observation";
import type {
  ApiError,
  AppInfoDto,
  CommandAcceptedDto,
  OutputDeviceDto,
  ProjectSummaryDto,
  ServiceSnapshotDto,
  PreparationSnapshotDto,
  SampleFileDto,
  MixerSettingsDto,
  MixerEditDto,
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
  const [mixerSettings, setMixerSettings] = useState<MixerSettingsDto | null>(
    null,
  );
  const [mixerEdit, setMixerEdit] = useState<
    MixerObservation | "offline" | null
  >(null);
  const [preparation, setPreparation] = useState<PreparationSnapshotDto | null>(
    null,
  );
  const preparationCurrent = useRef<PreparationSnapshotDto | null>(null);
  // Effect cleanup cannot cancel an invoke that already crossed IPC. Keep this
  // guard across cancellation/replacement effects until that invoke settles.
  const preparationPolling = useRef(false);
  const [trigger, setTrigger] = useState<TriggerObservation | null>(null);
  const current = useRef<ServiceSnapshotDto | null>(null);
  const mixerGeneration = useRef(0);
  // Startup reads can outlive an explicit refresh and a later edit.
  const mixerSelectionGeneration = useRef(0);
  const mounted = useRef(false);
  const working = useRef(false);
  const observe = useCallback((next: ServiceSnapshotDto) => {
    if (
      current.current &&
      BigInt(next.sequence) <= BigInt(current.current.sequence)
    )
      return;
    current.current = next;
    if (next.state !== "ready" || !next.mixer) mixerGeneration.current += 1;
    setSnapshot(next);
    setMixerEdit((previous) =>
      previous && previous !== "offline"
        ? observeMixer(next, previous)
        : previous,
    );
    setTrigger((previous) =>
      previous?.status === "pending"
        ? observeTrigger(next, previous.commandId)
        : previous,
    );
    setPending((id) =>
      next.state !== "ready" || (id && acknowledged(next, id)) ? null : id,
    );
  }, []);
  const observePreparation = useCallback((next: PreparationSnapshotDto) => {
    if (
      preparationCurrent.current &&
      BigInt(next.sequence) <= BigInt(preparationCurrent.current.sequence)
    )
      return;
    preparationCurrent.current = next;
    setPreparation(next);
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
          const failed = (reason: unknown) => {
            if (!disposed) setError(apiError(reason));
          };
          const selectionGeneration = mixerSelectionGeneration.current;
          await Promise.all([
            desktopClient.mixer().then(
              (settings) => {
                if (
                  !disposed &&
                  selectionGeneration === mixerSelectionGeneration.current
                )
                  setMixerSettings(settings);
              },
              (reason) => {
                if (selectionGeneration === mixerSelectionGeneration.current)
                  failed(reason);
              },
            ),
            desktopClient.devices().then((outputs) => {
              if (!disposed) setDevices(outputs);
            }, failed),
            desktopClient.preparation().then((prepared) => {
              if (!disposed) observePreparation(prepared);
            }, failed),
          ]);
        } catch (reason) {
          if (!disposed) setError(apiError(reason));
        }
      })();
    return () => {
      disposed = true;
      mounted.current = false;
      unlisten?.();
    };
  }, [native, observe, observePreparation]);
  useEffect(() => {
    if (
      !native ||
      !connected ||
      (preparation?.state !== "loading" && preparation?.state !== "cancelling")
    )
      return;
    let disposed = false;
    let timer: ReturnType<typeof setTimeout>;
    const poll = async () => {
      if (preparationPolling.current) {
        timer = setTimeout(() => void poll(), 200);
        return;
      }
      preparationPolling.current = true;
      try {
        const next = await desktopClient.preparation();
        if (!disposed) observePreparation(next);
      } catch (reason) {
        if (!disposed) setError(apiError(reason));
      } finally {
        preparationPolling.current = false;
      }
      if (!disposed) timer = setTimeout(() => void poll(), 200);
    };
    timer = setTimeout(() => void poll(), 200);
    return () => {
      disposed = true;
      clearTimeout(timer);
    };
  }, [native, connected, preparation?.state, observePreparation]);
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
  const edited = (edit: MixerEditDto, generation: number) => {
    mixerSelectionGeneration.current += 1;
    setMixerSettings(edit.settings);
    setMixerEdit(
      edit.accepted
        ? observeMixer(current.current, {
            commandId: edit.accepted.commandId,
            status:
              generation === mixerGeneration.current ? "pending" : "unknown",
          })
        : "offline",
    );
  };
  const editMixer = (action: () => Promise<MixerEditDto>) =>
    run(
      async () => {
        const generation = mixerGeneration.current;
        return { edit: await action(), generation };
      },
      ({ edit, generation }) => edited(edit, generation),
    );
  const initializeSampler = (deviceId: string | null, mixed: boolean) =>
    preparation?.preparationId &&
    run(
      async () => {
        try {
          const initialize = mixed
            ? desktopClient.initializeMixedSampler
            : desktopClient.initializeSampler;
          return await initialize(preparation.preparationId!, deviceId);
        } finally {
          try {
            const next = await desktopClient.preparation();
            if (mounted.current) observePreparation(next);
          } catch (reason) {
            if (mounted.current) setError(apiError(reason));
          }
        }
      },
      (next) => {
        setTrigger(null);
        observe(next);
      },
    );
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
    preparation,
    trigger,
    mixerSettings,
    mixerEdit,
    refreshMixer: () =>
      run(() => {
        mixerSelectionGeneration.current += 1;
        return desktopClient.mixer();
      }, setMixerSettings),
    setMixerChannel: (
      channel: number,
      gain: number,
      pan: number,
      muted: boolean,
    ) => editMixer(() => desktopClient.mixerChannel(channel, gain, pan, muted)),
    setMixerRoute: (slot: number, channel: number) =>
      editMixer(() => desktopClient.mixerRoute(slot, channel)),
    prepareSamples: (
      sampleRate: number,
      channels: number,
      files: SampleFileDto[],
    ) =>
      run(
        () => desktopClient.prepareSamples(sampleRate, channels, files),
        observePreparation,
      ),
    cancelPreparation: () =>
      preparation?.preparationId &&
      run(
        () => desktopClient.cancelPreparation(preparation.preparationId!),
        observePreparation,
      ),
    initializeSampler: (deviceId: string | null) =>
      initializeSampler(deviceId, false),
    initializeMixedSampler: (deviceId: string | null) =>
      initializeSampler(deviceId, true),
    triggerSample: (slot: number, gain: number) =>
      run(
        () => desktopClient.triggerSample(slot, gain),
        (command) =>
          setTrigger(observeTrigger(current.current, command.commandId)),
      ),
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
