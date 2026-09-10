import { computed, ref } from "vue";
import { defineStore } from "pinia";

import {
  mergeClient,
  type InteractiveWarning,
  type MergeInspection,
  type MergeSource,
  type OutputPreview,
} from "../../application/mergeClient";
import { errorCodeFrom, type ErrorCode } from "../../application/error";
import { createPastedDestinationIntent } from "../destination/pasteDestination";

interface MergeEntry extends MergeSource {
  entryId: string;
}

export const useMergeStore = defineStore("merge", () => {
  const sources = ref<MergeEntry[]>([]);
  let nextEntryId = 0;
  const pendingInspection = ref<MergeInspection | null>(null);
  const ignoredNonPdfs = ref<string[]>([]);
  const outputName = ref("");
  const destination = ref("");
  const outputPreview = ref<OutputPreview | null>(null);
  const errorCode = ref<ErrorCode | null>(null);
  const phase = ref<"preparing" | "running">("preparing");
  const progress = ref({ current: 0, total: 0, percent: 0 });
  let unlisten: (() => void) | undefined;
  let unlistenDrop: (() => void) | undefined;
  let unlistenClose: (() => void) | undefined;

  const warnings = computed<InteractiveWarning[]>(() => [
    ...new Set(sources.value.flatMap((source) => source.warnings)),
  ]);
  const canRequestSummary = computed(
    () =>
      phase.value === "preparing" &&
      sources.value.length >= 2 &&
      !!destination.value &&
      !!outputName.value,
  );

  async function initialize() {
    unlisten ??= await mergeClient.onMergeEvent((event) => {
      if (event.type === "progress") progress.value = event;
      if (event.type === "failed") {
        errorCode.value = event.error;
        phase.value = "preparing";
      }
      if (event.type === "cancelled") resetPreparation();
      if (event.type === "succeeded") resetPreparation();
    });
    unlistenDrop ??= await mergeClient.onFileDrop((paths) => {
      void addSelectedPaths(paths);
    });
  }

  async function protectWindowClose(confirmCancellation: () => Promise<boolean>) {
    unlistenClose ??= await mergeClient.onCloseRequest(async () => {
      if (phase.value !== "running") return true;
      if (!(await confirmCancellation())) return false;
      await cancelMerge();
      return true;
    });
  }

  async function addSelectedPaths(paths: string[]) {
    if (paths.length === 0 || phase.value === "running") return;
    errorCode.value = null;
    const inspection = await mergeClient.inspectSources(paths);
    ignoredNonPdfs.value = inspection.ignoredNonPdfs;
    if (inspection.incidents.length > 0) {
      pendingInspection.value = inspection;
      return;
    }
    appendSources(inspection.accepted);
    outputPreview.value = null;
  }

  async function choosePdfFiles() {
    if (phase.value === "running") return;
    await addSelectedPaths(await mergeClient.pickPdfFiles());
  }

  async function chooseSourceFolder() {
    if (phase.value === "running") return;
    const path = await mergeClient.pickFolder();
    if (path) await addSelectedPaths([path]);
  }

  async function chooseDestinationFolder() {
    if (phase.value === "running") return;
    const path = await mergeClient.pickFolder();
    if (path) chooseDestination(path);
  }

  function ignoreInvalidSources() {
    if (phase.value === "running" || !pendingInspection.value) return;
    appendSources(pendingInspection.value.accepted);
    pendingInspection.value = null;
    outputPreview.value = null;
  }

  function cancelPreparation() {
    if (phase.value === "running") return;
    resetPreparation();
    pendingInspection.value = null;
  }

  function removeSource(index: number) {
    if (phase.value === "running" || !isSourceIndex(index)) return;
    sources.value.splice(index, 1);
    outputPreview.value = null;
  }

  function moveSource(index: number, direction: -1 | 1) {
    reorderSource(index, index + direction);
  }

  function reorderSource(from: number, to: number) {
    if (phase.value === "running") return;
    if (from === to || !isSourceIndex(from) || !isSourceIndex(to)) return;
    const [source] = sources.value.splice(from, 1);
    sources.value.splice(to, 0, source);
    outputPreview.value = null;
  }

  function isSourceIndex(index: number) {
    return Number.isInteger(index) && index >= 0 && index < sources.value.length;
  }

  function appendSources(accepted: MergeSource[]) {
    sources.value.push(
      ...accepted.map((source) => ({ ...source, entryId: String(nextEntryId++) })),
    );
  }

  function preparationKey() {
    return JSON.stringify([
      sources.value.map((source) => source.entryId),
      destination.value,
      outputName.value,
    ]);
  }

  function renameOutput(name: string) {
    if (phase.value === "running") return;
    outputName.value = name;
    outputPreview.value = null;
  }

  function chooseDestination(path: string) {
    if (phase.value === "running") return;
    destination.value = path;
    outputPreview.value = null;
  }

  const pasteDestination = createPastedDestinationIntent(
    () => phase.value === "running",
    chooseDestination,
  );

  async function requestSummary() {
    if (!canRequestSummary.value) return null;
    const requestedPreparation = preparationKey();
    try {
      errorCode.value = null;
      const preview = await mergeClient.previewOutput(destination.value, outputName.value);
      if (requestedPreparation !== preparationKey() || phase.value === "running") return null;
      outputPreview.value = preview;
      return outputPreview.value;
    } catch (error) {
      errorCode.value = errorCodeFrom(error, "mergeFailed");
      return null;
    }
  }

  async function confirmMerge() {
    if (!outputPreview.value) return;
    try {
      phase.value = "running";
      progress.value = { current: 0, total: 0, percent: 0 };
      await mergeClient.start(
        sources.value.map((source) => source.path),
        destination.value,
        outputName.value,
      );
    } catch (error) {
      errorCode.value = errorCodeFrom(error, "mergeFailed");
      phase.value = "preparing";
    }
  }

  async function cancelMerge() {
    await mergeClient.cancel();
  }

  function resetPreparation() {
    sources.value = [];
    outputName.value = "";
    destination.value = "";
    outputPreview.value = null;
    pendingInspection.value = null;
    ignoredNonPdfs.value = [];
    progress.value = { current: 0, total: 0, percent: 0 };
    phase.value = "preparing";
  }

  function dispose() {
    unlisten?.();
    unlistenDrop?.();
    unlistenClose?.();
    unlisten = undefined;
    unlistenDrop = undefined;
    unlistenClose = undefined;
  }

  return {
    sources,
    pendingInspection,
    ignoredNonPdfs,
    outputName,
    destination,
    outputPreview,
    errorCode,
    phase,
    progress,
    warnings,
    canRequestSummary,
    initialize,
    protectWindowClose,
    addSelectedPaths,
    choosePdfFiles,
    chooseSourceFolder,
    chooseDestinationFolder,
    ignoreInvalidSources,
    cancelPreparation,
    removeSource,
    moveSource,
    reorderSource,
    renameOutput,
    chooseDestination,
    pasteDestination,
    requestSummary,
    confirmMerge,
    cancelMerge,
    dispose,
  };
});
