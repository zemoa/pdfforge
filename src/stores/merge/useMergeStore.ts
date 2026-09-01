import { computed, ref } from "vue";
import { defineStore } from "pinia";

import {
  mergeClient,
  type InteractiveWarning,
  type MergeInspection,
  type MergeSource,
  type OutputPreview,
} from "../../application/mergeClient";

export const useMergeStore = defineStore("merge", () => {
  const sources = ref<MergeSource[]>([]);
  const pendingInspection = ref<MergeInspection | null>(null);
  const ignoredNonPdfs = ref<string[]>([]);
  const outputName = ref("");
  const destination = ref("");
  const outputPreview = ref<OutputPreview | null>(null);
  const errorMessage = ref<string | null>(null);
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
        errorMessage.value = event.message;
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
    errorMessage.value = null;
    const inspection = await mergeClient.inspectSources(paths);
    ignoredNonPdfs.value = inspection.ignoredNonPdfs;
    if (inspection.incidents.length > 0) {
      pendingInspection.value = inspection;
      return;
    }
    sources.value.push(...inspection.accepted);
    outputPreview.value = null;
  }

  async function choosePdfFiles() {
    await addSelectedPaths(await mergeClient.pickPdfFiles());
  }

  async function chooseSourceFolder() {
    const path = await mergeClient.pickFolder();
    if (path) await addSelectedPaths([path]);
  }

  async function chooseDestinationFolder() {
    const path = await mergeClient.pickFolder();
    if (path) chooseDestination(path);
  }

  function ignoreInvalidSources() {
    if (!pendingInspection.value) return;
    sources.value.push(...pendingInspection.value.accepted);
    pendingInspection.value = null;
    outputPreview.value = null;
  }

  function cancelPreparation() {
    resetPreparation();
    pendingInspection.value = null;
  }

  function removeSource(index: number) {
    sources.value.splice(index, 1);
    outputPreview.value = null;
  }

  function moveSource(index: number, direction: -1 | 1) {
    const target = index + direction;
    if (target < 0 || target >= sources.value.length) return;
    const [source] = sources.value.splice(index, 1);
    sources.value.splice(target, 0, source);
    outputPreview.value = null;
  }

  function reorderSource(from: number, to: number) {
    if (from === to || to < 0 || to >= sources.value.length) return;
    const [source] = sources.value.splice(from, 1);
    sources.value.splice(to, 0, source);
    outputPreview.value = null;
  }

  function renameOutput(name: string) {
    outputName.value = name;
    outputPreview.value = null;
  }

  function chooseDestination(path: string) {
    destination.value = path;
    outputPreview.value = null;
  }

  async function requestSummary() {
    if (!canRequestSummary.value) return null;
    try {
      errorMessage.value = null;
      outputPreview.value = await mergeClient.previewOutput(destination.value, outputName.value);
      return outputPreview.value;
    } catch (error) {
      errorMessage.value = String(error);
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
      errorMessage.value = String(error);
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
    errorMessage,
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
    requestSummary,
    confirmMerge,
    cancelMerge,
    dispose,
  };
});
