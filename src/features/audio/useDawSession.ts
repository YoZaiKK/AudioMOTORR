import { useCallback, useEffect, useRef, useState } from "react";
import { apiError, desktopClient } from "../../shared/api/desktop-client";
import { observeTrigger, type TriggerObservation } from "./trigger-observation";
import { observeMixer, type MixerObservation } from "./mixer-observation";
import { observePattern, type PatternObservation } from "./pattern-observation";
import {
  observeLauncher,
  type LauncherObservation,
} from "./launcher-observation";
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
  PatternDto,
  PatternEditDto,
  ProjectDocumentDto,
  BeatExportDto,
  BeatExportRequestDto,
  LauncherSettingsDto,
  LauncherEditDto,
  LauncherTargetDto,
  LauncherLaunchDto,
  PlaybackModeDto,
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
  const [document, setDocument] = useState<ProjectDocumentDto | null>(null);
  const [documentVersion, setDocumentVersion] = useState(0);
  const [beatExport, setBeatExport] = useState<{
    path: string;
    report: BeatExportDto;
  } | null>(null);
  const documentGeneration = useRef(0);
  const [launcherSettings, setLauncherSettings] =
    useState<LauncherSettingsDto | null>(null);
  const [launcherLaunch, setLauncherLaunch] =
    useState<LauncherObservation | null>(null);
  const launcherGeneration = useRef(0);
  const launcherSelectionGeneration = useRef(0);
  const [devices, setDevices] = useState<OutputDeviceDto[]>([]);
  const [snapshot, setSnapshot] = useState<ServiceSnapshotDto | null>(null);
  const [error, setError] = useState<ApiError | null>(null);
  const [connected, setConnected] = useState(false);
  const [busy, setBusy] = useState(false);
  const [pending, setPending] = useState<string | null>(null);
  const [pattern, setPattern] = useState<PatternDto | null>(null);
  const [patternEdit, setPatternEdit] = useState<
    PatternObservation | "offline" | null
  >(null);
  const patternGeneration = useRef(0);
  const patternSelectionGeneration = useRef(0);
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
  const syncDocument = useCallback(() => {
    const generation = ++documentGeneration.current;
    void desktopClient.document().then(
      (next) => {
        if (mounted.current && generation === documentGeneration.current) {
          setDocument(next);
          setProject(next.summary);
        }
      },
      (reason) => {
        if (mounted.current && generation === documentGeneration.current)
          setError(apiError(reason));
      },
    );
  }, []);
  const observe = useCallback((next: ServiceSnapshotDto) => {
    if (
      current.current &&
      BigInt(next.sequence) <= BigInt(current.current.sequence)
    )
      return;
    current.current = next;
    if (next.state !== "ready" || !next.mixer) mixerGeneration.current += 1;
    if (next.state !== "ready" || !next.sequencer)
      patternGeneration.current += 1;
    if (next.state !== "ready" || !next.launcher) {
      launcherGeneration.current += 1;
      setLauncherLaunch(null);
    } else {
      setLauncherLaunch((previous) =>
        previous ? observeLauncher(next, previous) : null,
      );
    }
    setSnapshot(next);
    setPatternEdit((previous) =>
      previous && previous !== "offline"
        ? observePattern(next, previous)
        : previous,
    );
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
          const [appInfo, summary, selectedDocument] = await Promise.all([
            desktopClient.appInfo(),
            desktopClient.project(),
            desktopClient.document(),
          ]);
          if (disposed) return;
          setInfo(appInfo);
          setProject(summary);
          setDocument(selectedDocument);
          setLauncherSettings(selectedDocument.launcher);
          setDocumentVersion((previous) => previous + 1);
          setConnected(true);
          const failed = (reason: unknown) => {
            if (!disposed) setError(apiError(reason));
          };
          const selectionGeneration = mixerSelectionGeneration.current;
          const patternSelection = patternSelectionGeneration.current;
          await Promise.all([
            desktopClient.pattern().then(
              (selected) => {
                if (
                  !disposed &&
                  patternSelection === patternSelectionGeneration.current
                )
                  setPattern(selected);
              },
              (reason) => {
                if (patternSelection === patternSelectionGeneration.current)
                  failed(reason);
              },
            ),
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
    const stillPreparing = () =>
      preparationCurrent.current?.state === "loading" ||
      preparationCurrent.current?.state === "cancelling";
    const poll = async () => {
      // React may not have committed the final state and cleaned up this effect
      // yet. Consult the synchronously observed state before another IPC call.
      if (disposed || !stillPreparing()) return;
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
      if (!disposed && stillPreparing())
        timer = setTimeout(() => void poll(), 200);
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
        if (!mounted.current) return;
        success(result);
        return result;
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
    syncDocument();
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
  const editPattern = (action: () => Promise<PatternEditDto>) =>
    run(
      async () => {
        const generation = patternGeneration.current;
        return { edit: await action(), generation };
      },
      ({ edit, generation }) => {
        syncDocument();
        patternSelectionGeneration.current += 1;
        setPattern(edit.pattern);
        setPatternEdit(
          edit.accepted
            ? observePattern(current.current, {
                commandId: edit.accepted.commandId,
                status:
                  generation === patternGeneration.current
                    ? "pending"
                    : "unknown",
              })
            : "offline",
        );
      },
    );
  const initializeSampler = (
    deviceId: string | null,
    mode: "raw" | "mixed" | "sequenced" | "project",
  ) =>
    preparation?.preparationId &&
    run(
      async () => {
        try {
          const initialize =
            mode === "project"
              ? desktopClient.initializeProjectSampler
              : mode === "sequenced"
                ? desktopClient.initializeSequencedSampler
                : mode === "mixed"
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
  const applyDocument = (next: ProjectDocumentDto, replace: boolean) => {
    setDocument(next);
    setProject(next.summary);
    setLauncherSettings(next.launcher);
    if (replace) {
      setDocumentVersion((previous) => previous + 1);
      patternSelectionGeneration.current += 1;
      mixerSelectionGeneration.current += 1;
      setPattern(next.pattern);
      setMixerSettings(next.mixer);
      launcherSelectionGeneration.current += 1;
      launcherGeneration.current += 1;
      setLauncherLaunch(null);
      setPatternEdit(null);
      setMixerEdit(null);
      setBeatExport(null);
    }
  };
  return {
    launcherSettings,
    launcherLaunch,
    editLauncher: (request: LauncherEditDto) =>
      run(
        () => {
          launcherSelectionGeneration.current += 1;
          return desktopClient.launcherEdit(request);
        },
        (selected) => {
          setLauncherSettings(selected);
          syncDocument();
        },
      ),
    setPlaybackMode: (mode: PlaybackModeDto) =>
      run(
        () => {
          launcherSelectionGeneration.current += 1;
          return desktopClient.playbackMode(mode);
        },
        (selected) => {
          setLauncherSettings(selected);
          setLauncherLaunch(null);
          syncDocument();
        },
      ),
    refreshLauncher: () =>
      run(
        async () => {
          const selection = launcherSelectionGeneration.current;
          const generation = launcherGeneration.current;
          return {
            status: await desktopClient.launcherStatus(),
            selection,
            generation,
          };
        },
        ({ status, selection, generation }) => {
          if (selection === launcherSelectionGeneration.current)
            setLauncherSettings(status.selected);
          observe(status.observed);
          if (generation === launcherGeneration.current)
            setLauncherLaunch((previous) => {
              const next =
                status.lastLaunch && status.progress
                  ? observeLauncher(current.current, {
                      receipt: status.lastLaunch,
                      progress: status.progress,
                    })
                  : null;
              // A slow query can outlive an exact processing event and a newer
              // report. Keep that proven outcome for the same ID and session.
              if (
                next &&
                previous &&
                next.receipt.accepted.commandId ===
                  previous.receipt.accepted.commandId &&
                (previous.progress.status === "processed" ||
                  previous.progress.status === "rejected") &&
                (next.progress.status === "accepted" ||
                  next.progress.status === "unknown")
              )
                return { ...next, progress: previous.progress };
              return next;
            });
        },
      ),
    launchLauncher: (target: LauncherTargetDto) =>
      run(
        async () => {
          const generation = launcherGeneration.current;
          const receipt: LauncherLaunchDto =
            await desktopClient.launcherLaunch(target);
          return { receipt, generation };
        },
        ({ receipt, generation }) => {
          if (generation === launcherGeneration.current)
            setLauncherLaunch(
              observeLauncher(current.current, {
                receipt,
                progress: { status: "accepted" },
              }),
            );
        },
      ),
    native,
    connected,
    info,
    project,
    document,
    documentVersion,
    beatExport,
    refreshDocument: () =>
      run(
        () => {
          documentGeneration.current += 1;
          return desktopClient.document();
        },
        (next) => applyDocument(next, false),
      ),
    saveProject: (path: string, overwrite: boolean, title: string) =>
      run(
        () => {
          documentGeneration.current += 1;
          return desktopClient.saveProject(path, overwrite, title);
        },
        (next) => applyDocument(next, false),
      ),
    openProject: (path: string) =>
      run(
        () => {
          documentGeneration.current += 1;
          return desktopClient.openProject(path);
        },
        (next) => applyDocument(next, true),
      ),
    prepareProjectSamples: (sampleRate: number, channels: number) =>
      run(
        () => desktopClient.prepareProjectSamples(sampleRate, channels),
        observePreparation,
      ),
    exportBeat: (request: BeatExportRequestDto) =>
      run(
        () => desktopClient.exportBeat(request),
        (report) => setBeatExport({ path: request.outputPath, report }),
      ),
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
    pattern,
    patternEdit,
    refreshPattern: () =>
      run(() => {
        patternSelectionGeneration.current += 1;
        return desktopClient.pattern();
      }, setPattern),
    setPattern: (pattern: PatternDto) =>
      editPattern(() => desktopClient.setPattern(pattern)),
    setPatternStep: (row: number, step: number, enabled: boolean) =>
      editPattern(() => desktopClient.patternStep(row, step, enabled)),
    setPatternRowSlot: (row: number, slot: number) =>
      editPattern(() => desktopClient.patternRowSlot(row, slot)),
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
        (next) => {
          observePreparation(next);
          syncDocument();
        },
      ),
    cancelPreparation: () =>
      preparation?.preparationId &&
      run(
        () => desktopClient.cancelPreparation(preparation.preparationId!),
        observePreparation,
      ),
    initializeSampler: (deviceId: string | null) =>
      initializeSampler(deviceId, "raw"),
    initializeMixedSampler: (deviceId: string | null) =>
      initializeSampler(deviceId, "mixed"),
    initializeSequencedSampler: (deviceId: string | null) =>
      initializeSampler(deviceId, "sequenced"),
    initializeProjectSampler: (deviceId: string | null) =>
      initializeSampler(deviceId, "project"),
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
    setTempo: (bpm: number) =>
      run(
        async () =>
          current.current?.state === "uninitialized"
            ? (await desktopClient.projectTempo(bpm)).accepted
            : desktopClient.tempo(bpm),
        (reply) => {
          if (reply) accepted(reply);
          syncDocument();
        },
      ),
    setGain: (gain: number) => command(() => desktopClient.gain(gain)),
  };
}
export type DawSession = ReturnType<typeof useDawSession>;
