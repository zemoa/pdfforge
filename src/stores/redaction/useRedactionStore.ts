import { computed, ref } from "vue";
import { defineStore } from "pinia";

import {
  redactionClient,
  type OutputPreview,
  type PageRedaction,
  type RedactionPage,
  type RedactionSource,
} from "../../application/redactionClient";
import {
  addWordRange,
  addZone,
  clearZones,
  moveZone,
  removeWord,
  removeZone,
  resizeZone,
  toggleWord,
  type NormalizedPoint,
  type NormalizedRect,
  type SelectionsByPage,
  type ZoneResizeHandle,
  type ZonesByPage,
} from "./selection";
import { defaultOutputName, sourceDirectory } from "./output";

const MIN_ZOOM = 0.75;
const MAX_ZOOM = 2;
const ZOOM_STEP = 0.25;

export const useRedactionStore = defineStore("redaction", () => {
  const source = ref<RedactionSource | null>(null);
  const renderedPage = ref<RedactionPage | null>(null);
  const zoom = ref(1);
  const selections = ref<SelectionsByPage>({});
  const zones = ref<ZonesByPage>({});
  const loadingPage = ref(false);
  const errorMessage = ref<string | null>(null);
  const outputName = ref("");
  const destination = ref("");
  const outputPreview = ref<OutputPreview | null>(null);
  const phase = ref<"preparing" | "running">("preparing");
  const progress = ref({ current: 0, total: 0, percent: 0 });
  const outcome = ref<"succeeded" | "cancelled" | null>(null);
  let requestedPage = 0;
  let nextZoneId = 1;
  let closeAfterCancellation = false;
  let unlisten: (() => void) | undefined;
  let unlistenDrop: (() => void) | undefined;
  let unlistenClose: (() => void) | undefined;

  const currentPage = computed(() => renderedPage.value?.page ?? 1);
  const selectedWordIndexes = computed(
    () => new Set(Object.keys(selections.value[currentPage.value] ?? {}).map(Number)),
  );
  const zonesOnCurrentPage = computed(() => zones.value[currentPage.value] ?? []);
  const hasSelectableText = computed(
    () => !!renderedPage.value && renderedPage.value.words.length > 0,
  );
  const canDrawZones = computed(() => !!renderedPage.value);
  const selectionCount = computed(
    () =>
      Object.values(selections.value).reduce(
        (count, words) => count + Object.keys(words).length,
        0,
      ) + Object.values(zones.value).reduce((count, pageZones) => count + pageZones.length, 0),
  );
  const selectionSummary = computed(() => {
    const pages = new Set([...Object.keys(selections.value), ...Object.keys(zones.value)]);
    return [...pages]
      .map(Number)
      .sort((left, right) => left - right)
      .map((page) => ({
        page,
        words: Object.entries(selections.value[page] ?? {})
          .map(([index, selection]) => ({ index: Number(index), text: selection.text }))
          .sort((left, right) => left.index - right.index),
        zones: zones.value[page] ?? [],
      }));
  });
  const redactionSelections = computed<PageRedaction[]>(() => {
    const pages = new Set([...Object.keys(selections.value), ...Object.keys(zones.value)]);
    return [...pages]
      .map(Number)
      .sort((left, right) => left - right)
      .map((page) => ({
        page,
        rectangles: [
          ...Object.values(selections.value[page] ?? {}).flatMap((selection) => selection.bounds),
          ...(zones.value[page] ?? []).map((zone) => zone.rect),
        ],
      }))
      .filter((selection) => selection.rectangles.length > 0);
  });
  const canGoPrevious = computed(() => currentPage.value > 1);
  const canGoNext = computed(() => !!source.value && currentPage.value < source.value.pageCount);
  const canRequestSummary = computed(
    () =>
      phase.value === "preparing" &&
      !!source.value &&
      selectionCount.value > 0 &&
      !!destination.value &&
      !!outputName.value,
  );

  async function initialize() {
    unlisten ??= await redactionClient.onRedactionEvent((event) => {
      if (event.type === "progress") progress.value = event;
      if (event.type === "failed") {
        errorMessage.value = event.message;
        phase.value = "preparing";
      }
      if (event.type === "cancelled") {
        resetPreparation();
        outcome.value = "cancelled";
        if (closeAfterCancellation) {
          closeAfterCancellation = false;
          void redactionClient.destroyWindow();
        }
      }
      if (event.type === "succeeded") {
        resetPreparation();
        outcome.value = "succeeded";
      }
    });
    unlistenDrop ??= await redactionClient.onFileDrop((paths) => {
      void addSelectedPaths(paths);
    });
  }

  async function protectWindowClose(confirmCancellation: () => Promise<boolean>) {
    unlistenClose ??= await redactionClient.onCloseRequest(async () => {
      if (phase.value !== "running") return true;
      if (!(await confirmCancellation())) return false;
      closeAfterCancellation = true;
      await cancelRedaction();
      return false;
    });
  }

  async function addSelectedPaths(paths: string[]) {
    if (paths.length === 0 || phase.value === "running") return;
    try {
      errorMessage.value = null;
      outcome.value = null;
      const nextSource = await redactionClient.inspectSource(paths);
      source.value = nextSource;
      renderedPage.value = null;
      selections.value = {};
      zones.value = {};
      nextZoneId = 1;
      zoom.value = 1;
      outputName.value = defaultOutputName(nextSource.name);
      destination.value = sourceDirectory(nextSource.path);
      outputPreview.value = null;
      await loadPage(1);
    } catch (error) {
      errorMessage.value = String(error);
    }
  }

  async function choosePdfFile() {
    const path = await redactionClient.pickPdfFile();
    if (path) await addSelectedPaths([path]);
  }

  async function loadPage(page: number) {
    if (phase.value === "running" || !source.value || page < 1 || page > source.value.pageCount)
      return;
    const request = ++requestedPage;
    loadingPage.value = true;
    errorMessage.value = null;
    try {
      const pagePreview = await redactionClient.renderPage(source.value.path, page);
      if (request === requestedPage) renderedPage.value = pagePreview;
    } catch (error) {
      if (request === requestedPage) errorMessage.value = String(error);
    } finally {
      if (request === requestedPage) loadingPage.value = false;
    }
  }

  async function goToPage(page: number | null) {
    if (!source.value || page === null || !Number.isInteger(page)) return;
    await loadPage(page);
  }

  async function goToPreviousPage() {
    await goToPage(currentPage.value - 1);
  }

  async function goToNextPage() {
    await goToPage(currentPage.value + 1);
  }

  function zoomIn() {
    zoom.value = Math.min(MAX_ZOOM, zoom.value + ZOOM_STEP);
  }

  function zoomOut() {
    zoom.value = Math.max(MIN_ZOOM, zoom.value - ZOOM_STEP);
  }

  function resetZoom() {
    zoom.value = 1;
  }

  function toggleTextWord(wordIndex: number) {
    if (phase.value === "running") return;
    const word = renderedPage.value?.words.find((candidate) => candidate.index === wordIndex);
    if (!word) return;
    selections.value = toggleWord(
      selections.value,
      currentPage.value,
      word.index,
      word.text,
      word.bounds,
    );
    outputPreview.value = null;
  }

  function selectTextWordRange(first: number, last: number) {
    if (phase.value === "running" || !renderedPage.value) return;
    selections.value = addWordRange(
      selections.value,
      currentPage.value,
      first,
      last,
      renderedPage.value.words,
    );
    outputPreview.value = null;
  }

  function removeTextWord(page: number, wordIndex: number) {
    if (phase.value === "running") return;
    selections.value = removeWord(selections.value, page, wordIndex);
    outputPreview.value = null;
  }

  function addZoneSelection(start: NormalizedPoint, end: NormalizedPoint) {
    if (phase.value === "running" || !canDrawZones.value) return;
    zones.value = addZone(zones.value, currentPage.value, nextZoneId, start, end);
    if (zones.value[currentPage.value]?.some((zone) => zone.id === nextZoneId)) nextZoneId += 1;
    outputPreview.value = null;
  }

  function moveZoneSelection(id: number, original: NormalizedRect, offset: NormalizedPoint) {
    if (phase.value === "running") return;
    zones.value = moveZone(zones.value, currentPage.value, id, original, offset);
    outputPreview.value = null;
  }

  function resizeZoneSelection(
    id: number,
    original: NormalizedRect,
    handle: ZoneResizeHandle,
    point: NormalizedPoint,
  ) {
    if (phase.value === "running") return;
    zones.value = resizeZone(zones.value, currentPage.value, id, original, handle, point);
    outputPreview.value = null;
  }

  function removeZoneSelection(page: number, id: number) {
    if (phase.value === "running") return;
    zones.value = removeZone(zones.value, page, id);
    outputPreview.value = null;
  }

  function clearZoneSelections() {
    if (phase.value === "running") return;
    zones.value = clearZones(zones.value);
    outputPreview.value = null;
  }

  function clearSelections() {
    if (phase.value === "running") return;
    selections.value = {};
    clearZoneSelections();
  }

  async function chooseDestinationFolder() {
    const path = await redactionClient.pickFolder();
    if (path) chooseDestination(path);
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

  async function requestSummary() {
    if (!source.value || !canRequestSummary.value) return null;
    try {
      errorMessage.value = null;
      outputPreview.value = await redactionClient.previewOutput(
        source.value.path,
        redactionSelections.value,
        destination.value,
        outputName.value,
      );
      return outputPreview.value;
    } catch (error) {
      errorMessage.value = String(error);
      return null;
    }
  }

  async function confirmRedaction() {
    if (!source.value || !outputPreview.value) return;
    try {
      phase.value = "running";
      progress.value = { current: 0, total: 0, percent: 0 };
      await redactionClient.start(
        source.value.path,
        redactionSelections.value,
        destination.value,
        outputName.value,
      );
    } catch (error) {
      errorMessage.value = String(error);
      phase.value = "preparing";
    }
  }

  async function cancelRedaction() {
    await redactionClient.cancel();
  }

  function resetPreparation() {
    requestedPage += 1;
    source.value = null;
    renderedPage.value = null;
    selections.value = {};
    zones.value = {};
    nextZoneId = 1;
    zoom.value = 1;
    loadingPage.value = false;
    errorMessage.value = null;
    outputName.value = "";
    destination.value = "";
    outputPreview.value = null;
    progress.value = { current: 0, total: 0, percent: 0 };
    phase.value = "preparing";
  }

  function dismissOutcome() {
    outcome.value = null;
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
    renderedPage,
    zoom,
    loadingPage,
    errorMessage,
    outputName,
    destination,
    outputPreview,
    phase,
    progress,
    outcome,
    currentPage,
    selectedWordIndexes,
    zonesOnCurrentPage,
    hasSelectableText,
    canDrawZones,
    selectionCount,
    selectionSummary,
    redactionSelections,
    canGoPrevious,
    canGoNext,
    canRequestSummary,
    initialize,
    protectWindowClose,
    addSelectedPaths,
    choosePdfFile,
    goToPage,
    goToPreviousPage,
    goToNextPage,
    zoomIn,
    zoomOut,
    resetZoom,
    toggleTextWord,
    selectTextWordRange,
    removeTextWord,
    addZoneSelection,
    moveZoneSelection,
    resizeZoneSelection,
    removeZoneSelection,
    clearZoneSelections,
    clearSelections,
    chooseDestinationFolder,
    renameOutput,
    chooseDestination,
    requestSummary,
    confirmRedaction,
    cancelRedaction,
    resetPreparation,
    dismissOutcome,
    dispose,
  };
});
