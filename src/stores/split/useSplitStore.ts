import { computed, ref } from "vue";
import { defineStore } from "pinia";

import {
  splitClient,
  type OutputPreview,
  type SplitMode,
  type SplitSource,
} from "../../application/splitClient";
import { errorCodeFrom, type ErrorCode } from "../../application/error";
import { createPastedDestinationIntent } from "../destination/pasteDestination";

const THUMBNAIL_BATCH_SIZE = 24;

interface PageGroup {
  id: number;
  pages: number[];
}

export const useSplitStore = defineStore("split", () => {
  const source = ref<SplitSource | null>(null);
  const mode = ref<SplitMode>("eachPage");
  const selectedPages = ref<number[]>([]);
  const groups = ref<PageGroup[]>([]);
  const thumbnails = ref<Record<number, string>>({});
  const loadedPageCount = ref(0);
  const thumbnailsLoading = ref(false);
  const outputName = ref("");
  const destination = ref("");
  const outputPreview = ref<OutputPreview | null>(null);
  const errorCode = ref<ErrorCode | null>(null);
  const outcome = ref<"succeeded" | "cancelled" | null>(null);
  const phase = ref<"preparing" | "running">("preparing");
  const progress = ref({ current: 0, total: 0, percent: 0 });
  let nextGroupId = 1;
  let closeAfterCancellation = false;
  let unlisten: (() => void) | undefined;
  let unlistenDrop: (() => void) | undefined;
  let unlistenClose: (() => void) | undefined;

  const assignedPages = computed(() => groups.value.flatMap((group) => group.pages));
  const displayedPages = computed(() =>
    Array.from({ length: loadedPageCount.value }, (_, index) => index + 1),
  );
  const canLoadMoreThumbnails = computed(
    () => !!source.value && loadedPageCount.value < source.value.pageCount,
  );
  const canRequestSummary = computed(() => {
    if (phase.value !== "preparing" || !source.value || !destination.value || !outputName.value) {
      return false;
    }
    if (mode.value === "extract") return selectedPages.value.length > 0;
    if (mode.value === "groups") return groups.value.length > 0;
    return true;
  });

  async function initialize() {
    unlisten ??= await splitClient.onSplitEvent((event) => {
      if (event.type === "progress") progress.value = event;
      if (event.type === "failed") {
        errorCode.value = event.error;
        phase.value = "preparing";
      }
      if (event.type === "cancelled") {
        resetPreparation();
        outcome.value = "cancelled";
        if (closeAfterCancellation) {
          closeAfterCancellation = false;
          void splitClient.destroyWindow();
        }
      }
      if (event.type === "succeeded") {
        resetPreparation();
        outcome.value = "succeeded";
      }
    });
    unlistenDrop ??= await splitClient.onFileDrop((paths) => {
      void addSelectedPaths(paths);
    });
  }

  async function protectWindowClose(confirmCancellation: () => Promise<boolean>) {
    unlistenClose ??= await splitClient.onCloseRequest(async () => {
      if (phase.value !== "running") return true;
      if (!(await confirmCancellation())) return false;
      closeAfterCancellation = true;
      await cancelSplit();
      return false;
    });
  }

  async function addSelectedPaths(paths: string[]) {
    if (paths.length === 0 || phase.value === "running") return;
    try {
      errorCode.value = null;
      outcome.value = null;
      source.value = await splitClient.inspectSource(paths);
      mode.value = "eachPage";
      selectedPages.value = [];
      groups.value = [];
      thumbnails.value = {};
      loadedPageCount.value = 0;
      outputPreview.value = null;
      await loadNextThumbnails();
    } catch (error) {
      errorCode.value = errorCodeFrom(error, "splitFailed");
    }
  }

  async function choosePdfFile() {
    if (phase.value === "running") return;
    const path = await splitClient.pickPdfFile();
    if (path) await addSelectedPaths([path]);
  }

  async function chooseDestinationFolder() {
    if (phase.value === "running") return;
    const path = await splitClient.pickFolder();
    if (path) chooseDestination(path);
  }

  async function loadNextThumbnails() {
    if (
      phase.value === "running" ||
      !source.value ||
      thumbnailsLoading.value ||
      !canLoadMoreThumbnails.value
    )
      return;
    thumbnailsLoading.value = true;
    const firstPage = loadedPageCount.value + 1;
    const lastPage = Math.min(source.value.pageCount, firstPage + THUMBNAIL_BATCH_SIZE - 1);
    try {
      const rendered = await splitClient.renderThumbnails(
        source.value.path,
        Array.from({ length: lastPage - firstPage + 1 }, (_, index) => firstPage + index),
      );
      thumbnails.value = {
        ...thumbnails.value,
        ...Object.fromEntries(rendered.map((thumbnail) => [thumbnail.page, thumbnail.pngDataUrl])),
      };
      loadedPageCount.value = lastPage;
    } catch (error) {
      errorCode.value = errorCodeFrom(error, "splitFailed");
    } finally {
      thumbnailsLoading.value = false;
    }
  }

  function chooseMode(nextMode: SplitMode) {
    if (phase.value === "running") return;
    if (mode.value !== nextMode) {
      selectedPages.value = [];
      groups.value = [];
    }
    mode.value = nextMode;
    outputPreview.value = null;
    outcome.value = null;
  }

  function togglePage(page: number) {
    if (
      phase.value === "running" ||
      mode.value === "eachPage" ||
      assignedPages.value.includes(page)
    )
      return;
    if (selectedPages.value.includes(page)) {
      selectedPages.value = selectedPages.value.filter((candidate) => candidate !== page);
    } else {
      selectedPages.value = [...selectedPages.value, page].sort((left, right) => left - right);
    }
    outputPreview.value = null;
  }

  function clearSelectedPages() {
    if (phase.value === "running") return;
    selectedPages.value = [];
    outputPreview.value = null;
  }

  function createGroupFromSelection() {
    if (phase.value === "running" || selectedPages.value.length === 0) return;
    groups.value.push({ id: nextGroupId, pages: selectedPages.value });
    nextGroupId += 1;
    selectedPages.value = [];
    outputPreview.value = null;
  }

  function removeGroup(id: number) {
    if (phase.value === "running") return;
    groups.value = groups.value.filter((group) => group.id !== id);
    outputPreview.value = null;
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
    if (!source.value || !canRequestSummary.value) return null;
    try {
      errorCode.value = null;
      outputPreview.value = await splitClient.previewOutput(
        source.value.path,
        mode.value,
        selectedPages.value,
        groups.value.map((group) => group.pages),
        destination.value,
        outputName.value,
      );
      return outputPreview.value;
    } catch (error) {
      errorCode.value = errorCodeFrom(error, "splitFailed");
      return null;
    }
  }

  async function confirmSplit() {
    if (!source.value || !outputPreview.value) return;
    try {
      phase.value = "running";
      progress.value = { current: 0, total: 0, percent: 0 };
      await splitClient.start(
        source.value.path,
        mode.value,
        selectedPages.value,
        groups.value.map((group) => group.pages),
        destination.value,
        outputName.value,
      );
    } catch (error) {
      errorCode.value = errorCodeFrom(error, "splitFailed");
      phase.value = "preparing";
    }
  }

  async function cancelSplit() {
    await splitClient.cancel();
  }

  function removeSource() {
    if (phase.value === "running") return;
    resetPreparation();
  }

  function resetPreparation() {
    source.value = null;
    mode.value = "eachPage";
    selectedPages.value = [];
    groups.value = [];
    thumbnails.value = {};
    loadedPageCount.value = 0;
    outputName.value = "";
    destination.value = "";
    outputPreview.value = null;
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
    source,
    mode,
    selectedPages,
    groups,
    thumbnails,
    displayedPages,
    thumbnailsLoading,
    canLoadMoreThumbnails,
    assignedPages,
    outputName,
    destination,
    outputPreview,
    errorCode,
    outcome,
    phase,
    progress,
    canRequestSummary,
    initialize,
    protectWindowClose,
    addSelectedPaths,
    choosePdfFile,
    chooseDestinationFolder,
    loadNextThumbnails,
    chooseMode,
    togglePage,
    clearSelectedPages,
    createGroupFromSelection,
    removeGroup,
    removeSource,
    renameOutput,
    chooseDestination,
    pasteDestination,
    requestSummary,
    confirmSplit,
    cancelSplit,
    resetPreparation,
    dispose,
  };
});
