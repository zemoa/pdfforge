import { computed, ref } from "vue";
import { defineStore } from "pinia";

import {
  redactionClient,
  type RedactionPage,
  type RedactionSource,
} from "../../application/redactionClient";
import { addWordRange, removeWord, toggleWord, type SelectionsByPage } from "./selection";

const MIN_ZOOM = 0.75;
const MAX_ZOOM = 2;
const ZOOM_STEP = 0.25;

export const useRedactionStore = defineStore("redaction", () => {
  const source = ref<RedactionSource | null>(null);
  const renderedPage = ref<RedactionPage | null>(null);
  const zoom = ref(1);
  const selections = ref<SelectionsByPage>({});
  const loadingPage = ref(false);
  const errorMessage = ref<string | null>(null);
  let requestedPage = 0;
  let unlistenDrop: (() => void) | undefined;

  const currentPage = computed(() => renderedPage.value?.page ?? 1);
  const selectedWordIndexes = computed(
    () => new Set(Object.keys(selections.value[currentPage.value] ?? {}).map(Number)),
  );
  const selectionCount = computed(() =>
    Object.values(selections.value).reduce((count, words) => count + Object.keys(words).length, 0),
  );
  const selectionSummary = computed(() =>
    Object.entries(selections.value)
      .map(([page, words]) => ({
        page: Number(page),
        words: Object.entries(words)
          .map(([index, text]) => ({ index: Number(index), text }))
          .sort((left, right) => left.index - right.index),
      }))
      .sort((left, right) => left.page - right.page),
  );
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

  function clearSelections() {
    selections.value = {};
  }

  function resetPreparation() {
    requestedPage += 1;
    source.value = null;
    renderedPage.value = null;
    selections.value = {};
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
    clearSelections,
    resetPreparation,
    dispose,
  };
});
