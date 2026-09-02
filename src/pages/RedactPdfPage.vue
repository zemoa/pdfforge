<script setup lang="ts">
import {
  NAlert,
  NButton,
  NCard,
  NEmpty,
  NInputNumber,
  NLayout,
  NLayoutContent,
  NLayoutSider,
  NList,
  NListItem,
  NSpin,
  NSpace,
  NText,
  NThing,
} from "naive-ui";
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import { useI18n } from "vue-i18n";
import { useRouter } from "vue-router";

import { useRedactionStore } from "../stores/redaction/useRedactionStore";

const { t } = useI18n();
const router = useRouter();
const redaction = useRedactionStore();
const dragAnchor = ref<number | null>(null);
const dragEnd = ref<number | null>(null);
const selectionsSidebarCollapsed = ref(false);

const viewerStyle = computed(() => ({
  aspectRatio: `${redaction.renderedPage?.aspectRatio ?? 1} / 1`,
  width: `${redaction.zoom * 100}%`,
}));

onMounted(() => {
  void redaction.initialize();
  window.addEventListener("pointerup", finishWordSelection);
});
onBeforeUnmount(() => {
  window.removeEventListener("pointerup", finishWordSelection);
  redaction.dispose();
});

function beginWordSelection(wordIndex: number) {
  dragAnchor.value = wordIndex;
  dragEnd.value = wordIndex;
}

function extendWordSelection(wordIndex: number) {
  if (dragAnchor.value !== null) dragEnd.value = wordIndex;
}

function finishWordSelection() {
  if (dragAnchor.value === null || dragEnd.value === null) return;
  if (dragAnchor.value === dragEnd.value) {
    redaction.toggleTextWord(dragAnchor.value);
  } else {
    redaction.selectTextWordRange(dragAnchor.value, dragEnd.value);
  }
  dragAnchor.value = null;
  dragEnd.value = null;
}

function isInDragRange(wordIndex: number) {
  if (dragAnchor.value === null || dragEnd.value === null) return false;
  return (
    wordIndex >= Math.min(dragAnchor.value, dragEnd.value) &&
    wordIndex <= Math.max(dragAnchor.value, dragEnd.value)
  );
}
</script>

<template>
  <NLayout has-sider class="application-shell">
    <NLayoutSider
      v-model:collapsed="selectionsSidebarCollapsed"
      bordered
      collapse-mode="width"
      :collapsed-width="48"
      :width="296"
      show-trigger
      class="selections-sidebar"
    >
      <aside class="sidebar-content">
        <NCard
          size="small"
          embedded
          :title="t('redaction.selections', { count: redaction.selectionCount })"
        >
          <NEmpty
            v-if="redaction.selectionCount === 0"
            :description="t('redaction.emptySelections')"
          />
          <NList v-else bordered>
            <template v-for="selection in redaction.selectionSummary" :key="selection.page">
              <NListItem v-for="word in selection.words" :key="word.index">
                <NText>{{
                  t("redaction.selection", { page: selection.page, word: word.text })
                }}</NText>
                <template #suffix>
                  <NButton
                    quaternary
                    type="error"
                    size="small"
                    @click="redaction.removeTextWord(selection.page, word.index)"
                  >
                    {{ t("redaction.remove") }}
                  </NButton>
                </template>
              </NListItem>
            </template>
          </NList>
          <NButton
            v-if="redaction.selectionCount > 0"
            type="error"
            secondary
            class="clear-selections"
            @click="redaction.clearSelections"
          >
            {{ t("redaction.clearSelections") }}
          </NButton>
        </NCard>
      </aside>
    </NLayoutSider>

    <NLayoutContent class="workspace-content">
      <main class="workspace">
        <header class="workspace-heading">
          <div>
            <h1>{{ t("redaction.heading") }}</h1>
            <NText depth="3">{{ t("redaction.intro") }}</NText>
          </div>
        </header>

        <NAlert
          v-if="redaction.errorMessage"
          type="error"
          :title="t('redaction.error')"
          class="workspace-alert"
        >
          {{ redaction.errorMessage }}
        </NAlert>

        <template v-if="redaction.source">
          <div class="viewer-scroll">
            <NSpin :show="redaction.loadingPage">
              <div v-if="redaction.renderedPage" class="viewer-page" :style="viewerStyle">
                <img
                  :src="redaction.renderedPage.pngDataUrl"
                  :alt="t('redaction.pagePreview', { page: redaction.currentPage })"
                />
                <template v-for="word in redaction.renderedPage.words" :key="word.index">
                  <button
                    v-for="(bounds, boundsIndex) in word.bounds"
                    :key="boundsIndex"
                    class="word-hitbox"
                    :class="{
                      selected: redaction.selectedWordIndexes.has(word.index),
                      preview: isInDragRange(word.index),
                    }"
                    :style="{
                      left: `${bounds.left * 100}%`,
                      top: `${bounds.top * 100}%`,
                      width: `${bounds.width * 100}%`,
                      height: `${bounds.height * 100}%`,
                    }"
                    type="button"
                    :aria-label="t('redaction.selectWord', { word: word.text })"
                    @pointerdown.prevent="beginWordSelection(word.index)"
                    @pointerenter="extendWordSelection(word.index)"
                    @keydown.enter.prevent="redaction.toggleTextWord(word.index)"
                    @keydown.space.prevent="redaction.toggleTextWord(word.index)"
                  />
                </template>
              </div>
            </NSpin>
          </div>
          <NAlert
            v-if="redaction.renderedPage && redaction.renderedPage.words.length === 0"
            type="info"
            class="no-text-alert"
          >
            {{ t("redaction.noSelectableText") }}
          </NAlert>
          <NText depth="3" class="selection-hint">{{ t("redaction.selectionHint") }}</NText>
        </template>

        <NEmpty v-else class="empty-workspace" :description="t('redaction.emptySource')">
          <template #extra>
            <NButton type="primary" @click="redaction.choosePdfFile">{{
              t("redaction.addSource")
            }}</NButton>
          </template>
        </NEmpty>
      </main>
    </NLayoutContent>

    <NLayoutSider bordered :width="280" class="details-sidebar">
      <aside class="sidebar-content">
        <NButton block @click="router.push('/')">{{ t("common.home") }}</NButton>

        <NCard size="small" embedded :title="t('redaction.source')">
          <template v-if="redaction.source">
            <NThing :title="redaction.source.name" :description="redaction.source.path">
              <NText depth="3">{{
                t("redaction.pageCount", { count: redaction.source.pageCount })
              }}</NText>
            </NThing>
            <NSpace vertical>
              <NButton block @click="redaction.choosePdfFile">{{
                t("redaction.replaceSource")
              }}</NButton>
              <NButton block type="error" @click="redaction.resetPreparation">{{
                t("redaction.removeSource")
              }}</NButton>
            </NSpace>
          </template>
          <template v-else>
            <NText depth="3">{{ t("redaction.dropHint") }}</NText>
          </template>
        </NCard>

        <template v-if="redaction.source">
          <NCard size="small" embedded :title="t('redaction.viewer')">
            <NSpace align="center" justify="center">
              <NButton
                :disabled="!redaction.canGoPrevious || redaction.loadingPage"
                @click="redaction.goToPreviousPage"
              >
                {{ t("redaction.previousPage") }}
              </NButton>
              <NInputNumber
                :value="redaction.currentPage"
                :min="1"
                :max="redaction.source.pageCount"
                :show-button="false"
                class="page-number"
                @update:value="redaction.goToPage"
              />
              <NButton
                :disabled="!redaction.canGoNext || redaction.loadingPage"
                @click="redaction.goToNextPage"
              >
                {{ t("redaction.nextPage") }}
              </NButton>
            </NSpace>
            <NText depth="3" class="page-count">{{
              t("redaction.ofPages", { count: redaction.source.pageCount })
            }}</NText>
            <NSpace align="center" justify="center" class="zoom-controls">
              <NButton size="small" :disabled="redaction.zoom <= 0.75" @click="redaction.zoomOut"
                >−</NButton
              >
              <NButton size="small" @click="redaction.resetZoom"
                >{{ Math.round(redaction.zoom * 100) }}%</NButton
              >
              <NButton size="small" :disabled="redaction.zoom >= 2" @click="redaction.zoomIn"
                >+</NButton
              >
            </NSpace>
          </NCard>
        </template>
      </aside>
    </NLayoutSider>
  </NLayout>
</template>

<style scoped>
.application-shell {
  height: 100vh;
}

.workspace-content {
  min-width: 0;
  order: 2;
}

.details-sidebar {
  order: 1;
}

.selections-sidebar {
  order: 3;
}

.workspace {
  box-sizing: border-box;
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
  height: 100%;
  min-height: 0;
  padding: 1rem;
}

.sidebar-content {
  display: grid;
  gap: 1rem;
  height: 100%;
  overflow: auto;
  padding: 1rem;
}

h1 {
  margin: 0;
}

.selection-hint {
  display: block;
  margin: 0 0 0.25rem;
}

.page-number {
  width: 4.5rem;
}

.viewer-scroll {
  flex: 1;
  min-height: 0;
  overflow: auto;
}

.viewer-page {
  margin: 0 auto;
  position: relative;
}

.viewer-page img {
  display: block;
  height: auto;
  user-select: none;
  width: 100%;
}

.word-hitbox {
  background: transparent;
  border: 0;
  cursor: text;
  padding: 0;
  position: absolute;
}

.word-hitbox:hover,
.word-hitbox:focus-visible,
.word-hitbox.preview {
  background: rgb(245 158 11 / 65%);
  outline: 1px solid rgb(146 64 14);
}

.word-hitbox.selected {
  background: #000;
  outline: 0;
}

.word-hitbox.selected.preview {
  background: rgb(31 41 55 / 85%);
  outline: 1px solid rgb(245 158 11);
}

.no-text-alert {
  margin: 0;
}

.empty-workspace {
  display: grid;
  flex: 1;
  place-items: center;
}

.clear-selections {
  margin-top: 0.75rem;
}

.page-count {
  display: block;
  margin-top: 0.5rem;
  text-align: center;
}

.zoom-controls {
  margin-top: 1rem;
}
</style>
