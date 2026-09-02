import { computed, ref } from "vue";
import { defineStore } from "pinia";

import {
  redactionClient,
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
  let requestedPage = 0;
  let nextZoneId = 1;
  let unlistenDrop: (() => void) | undefined;

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
          .map(([index, text]) => ({ index: Number(index), text }))
          .sort((left, right) => left.index - right.index),
        zones: zones.value[page] ?? [],
      }));
  });
  const canGoPrevious = computed(() => currentPage.value > 1);
  const canGoNext = computed(() => !!source.value && currentPage.value < source.value.pageCount);

  async function initialize() {
    unlistenDrop ??= await redactionClient.onFileDrop((paths) => {
      void addSelectedPaths(paths);
    });
  }

  async function addSelectedPaths(paths: string[]) {
    if (paths.length === 0) return;
    try {
      errorMessage.value = null;
      const nextSource = await redactionClient.inspectSource(paths);
      source.value = nextSource;
      renderedPage.value = null;
      selections.value = {};
      zones.value = {};
      nextZoneId = 1;
      zoom.value = 1;
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
    if (!source.value || page < 1 || page > source.value.pageCount) return;
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
    const word = renderedPage.value?.words.find((candidate) => candidate.index === wordIndex);
    if (!word) return;
    selections.value = toggleWord(selections.value, currentPage.value, word.index, word.text);
  }

  function selectTextWordRange(first: number, last: number) {
    if (!renderedPage.value) return;
    selections.value = addWordRange(
      selections.value,
      currentPage.value,
      first,
      last,
      renderedPage.value.words,
    );
  }

  function removeTextWord(page: number, wordIndex: number) {
    selections.value = removeWord(selections.value, page, wordIndex);
  }

  function addZoneSelection(start: NormalizedPoint, end: NormalizedPoint) {
    if (!canDrawZones.value) return;
    zones.value = addZone(zones.value, currentPage.value, nextZoneId, start, end);
    if (zones.value[currentPage.value]?.some((zone) => zone.id === nextZoneId)) nextZoneId += 1;
  }

  function moveZoneSelection(id: number, original: NormalizedRect, offset: NormalizedPoint) {
    zones.value = moveZone(zones.value, currentPage.value, id, original, offset);
  }

  function resizeZoneSelection(
    id: number,
    original: NormalizedRect,
    handle: ZoneResizeHandle,
    point: NormalizedPoint,
  ) {
    zones.value = resizeZone(zones.value, currentPage.value, id, original, handle, point);
  }

  function removeZoneSelection(page: number, id: number) {
    zones.value = removeZone(zones.value, page, id);
  }

  function clearZoneSelections() {
    zones.value = clearZones(zones.value);
  }

  function clearSelections() {
    selections.value = {};
    clearZoneSelections();
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
  }

  function dispose() {
    unlistenDrop?.();
    unlistenDrop = undefined;
  }

  return {
    source,
    renderedPage,
    zoom,
    loadingPage,
    errorMessage,
    currentPage,
    selectedWordIndexes,
    zonesOnCurrentPage,
    hasSelectableText,
    canDrawZones,
    selectionCount,
    selectionSummary,
    canGoPrevious,
    canGoNext,
    initialize,
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
    resetPreparation,
    dispose,
  };
});
