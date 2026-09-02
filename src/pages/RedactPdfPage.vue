<script setup lang="ts">
import {
  NAlert,
  NButton,
  NCard,
  NEmpty,
  NInput,
  NInputNumber,
  NLayout,
  NLayoutContent,
  NLayoutSider,
  NList,
  NListItem,
  NModal,
  NProgress,
  NSpin,
  NSpace,
  NText,
  NThing,
} from "naive-ui";
import { computed, onBeforeUnmount, onMounted, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { useRouter } from "vue-router";

import {
  rectangleFromPoints,
  type NormalizedPoint,
  type NormalizedRect,
  type ZoneResizeHandle,
  type ZoneSelection,
} from "../stores/redaction/selection";
import { useRedactionStore } from "../stores/redaction/useRedactionStore";

const { t } = useI18n();
const router = useRouter();
const redaction = useRedactionStore();
const dragAnchor = ref<number | null>(null);
const dragEnd = ref<number | null>(null);
const selectionsSidebarCollapsed = ref(false);
const viewerPage = ref<HTMLElement | null>(null);
const draftZone = ref<NormalizedRect | null>(null);
const zoneGesture = ref<ZoneGesture | null>(null);
const selectionMode = ref<SelectionMode>("text");
const showSummary = ref(false);
const resizeHandles: ZoneResizeHandle[] = ["top-left", "top-right", "bottom-left", "bottom-right"];

type ZoneGesture =
  | { kind: "create"; start: NormalizedPoint }
  | { kind: "move"; id: number; start: NormalizedPoint; original: NormalizedRect }
  | {
      kind: "resize";
      id: number;
      start: NormalizedPoint;
      original: NormalizedRect;
      handle: ZoneResizeHandle;
    };

type SelectionMode = "text" | "zone";

const viewerStyle = computed(() => ({
  aspectRatio: `${redaction.renderedPage?.aspectRatio ?? 1} / 1`,
  width: `${redaction.zoom * 100}%`,
}));
const isZoneMode = computed(
  () => redaction.canDrawZones && (!redaction.hasSelectableText || selectionMode.value === "zone"),
);

onMounted(() => {
  void redaction.initialize();
  void redaction.protectWindowClose(async () => window.confirm(t("redaction.closeWhileRunning")));
  window.addEventListener("pointermove", updateZoneGesture);
  window.addEventListener("pointerup", finishSelections);
});

watch(
  () => redaction.outcome,
  (outcome) => {
    if (!outcome) return;
    window.setTimeout(() => redaction.dismissOutcome(), 3500);
  },
);

async function openSummary() {
  if (await redaction.requestSummary()) showSummary.value = true;
}
onBeforeUnmount(() => {
  window.removeEventListener("pointermove", updateZoneGesture);
  window.removeEventListener("pointerup", finishSelections);
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

function finishSelections(event: PointerEvent) {
  finishWordSelection();
  finishZoneGesture(event);
}

function pointFromPointer(event: PointerEvent): NormalizedPoint | null {
  const bounds = viewerPage.value?.getBoundingClientRect();
  if (!bounds || bounds.width === 0 || bounds.height === 0) return null;
  return {
    x: (event.clientX - bounds.left) / bounds.width,
    y: (event.clientY - bounds.top) / bounds.height,
  };
}

function rectangleStyle(rectangle: NormalizedRect) {
  return {
    left: `${rectangle.left * 100}%`,
    top: `${rectangle.top * 100}%`,
    width: `${rectangle.width * 100}%`,
    height: `${rectangle.height * 100}%`,
  };
}

function beginZone(event: PointerEvent) {
  if (!isZoneMode.value || event.button !== 0) return;
  const start = pointFromPointer(event);
  if (!start) return;
  zoneGesture.value = { kind: "create", start };
  draftZone.value = null;
}

function beginZoneMove(event: PointerEvent, zone: ZoneSelection) {
  if (event.button !== 0) return;
  const start = pointFromPointer(event);
  if (!start) return;
  zoneGesture.value = { kind: "move", id: zone.id, start, original: zone.rect };
}

function beginZoneResize(event: PointerEvent, zone: ZoneSelection, handle: ZoneResizeHandle) {
  if (event.button !== 0) return;
  const start = pointFromPointer(event);
  if (!start) return;
  zoneGesture.value = { kind: "resize", id: zone.id, start, original: zone.rect, handle };
}

function updateZoneGesture(event: PointerEvent) {
  const gesture = zoneGesture.value;
  const point = pointFromPointer(event);
  if (!gesture || !point) return;

  if (gesture.kind === "create") {
    draftZone.value = rectangleFromPoints(gesture.start, point);
  } else if (gesture.kind === "move") {
    redaction.moveZoneSelection(gesture.id, gesture.original, {
      x: point.x - gesture.start.x,
      y: point.y - gesture.start.y,
    });
  } else {
    redaction.resizeZoneSelection(gesture.id, gesture.original, gesture.handle, point);
  }
}

function finishZoneGesture(event: PointerEvent) {
  const gesture = zoneGesture.value;
  if (!gesture) return;
  const end = pointFromPointer(event);
  if (gesture.kind === "create" && end) redaction.addZoneSelection(gesture.start, end);
  zoneGesture.value = null;
  draftZone.value = null;
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
              <NListItem v-for="(zone, index) in selection.zones" :key="zone.id">
                <NText>{{
                  t("redaction.zoneSelection", { page: selection.page, zone: index + 1 })
                }}</NText>
                <template #suffix>
                  <NButton
                    quaternary
                    type="error"
                    size="small"
                    @click="redaction.removeZoneSelection(selection.page, zone.id)"
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
        <NAlert
          v-if="redaction.outcome === 'succeeded'"
          type="success"
          :title="t('redaction.success')"
        >
          {{ t("redaction.successBody") }}
        </NAlert>
        <NAlert
          v-if="redaction.outcome === 'cancelled'"
          type="info"
          :title="t('redaction.cancelled')"
        >
          {{ t("redaction.cancelledBody") }}
        </NAlert>

        <NCard v-if="redaction.phase === 'running'" embedded>
          <NThing :title="t('redaction.processing')">
            <NText depth="3">{{ t("redaction.progress", redaction.progress) }}</NText>
          </NThing>
          <NProgress
            :percentage="redaction.progress.percent"
            indicator-placement="inside"
            processing
          />
          <NButton type="error" @click="redaction.cancelRedaction">{{
            t("redaction.cancel")
          }}</NButton>
        </NCard>

        <template v-else-if="redaction.source">
          <NSpace v-if="redaction.hasSelectableText" align="center" class="selection-mode">
            <NText depth="3">{{ t("redaction.selectionMode") }}</NText>
            <NButton
              size="small"
              :type="selectionMode === 'text' ? 'primary' : 'default'"
              @click="selectionMode = 'text'"
            >
              {{ t("redaction.selectTextMode") }}
            </NButton>
            <NButton
              size="small"
              :type="selectionMode === 'zone' ? 'primary' : 'default'"
              @click="selectionMode = 'zone'"
            >
              {{ t("redaction.drawZoneMode") }}
            </NButton>
          </NSpace>
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
                    :tabindex="isZoneMode ? -1 : 0"
                    @pointerdown.prevent="beginWordSelection(word.index)"
                    @pointerenter="extendWordSelection(word.index)"
                    @keydown.enter.prevent="redaction.toggleTextWord(word.index)"
                    @keydown.space.prevent="redaction.toggleTextWord(word.index)"
                  />
                </template>
                <div
                  v-if="redaction.canDrawZones"
                  ref="viewerPage"
                  class="zone-drawing-layer"
                  :class="{ interactive: isZoneMode }"
                  @pointerdown.prevent="beginZone"
                >
                  <div
                    v-for="zone in redaction.zonesOnCurrentPage"
                    :key="zone.id"
                    class="zone-preview"
                    :style="rectangleStyle(zone.rect)"
                    :aria-label="
                      t('redaction.zoneSelection', {
                        page: zone.page,
                        zone: zone.id,
                      })
                    "
                    role="group"
                    @pointerdown.stop.prevent="beginZoneMove($event, zone)"
                  >
                    <button
                      v-for="handle in resizeHandles"
                      :key="handle"
                      class="zone-handle"
                      :class="handle"
                      type="button"
                      :aria-label="t('redaction.resizeZone')"
                      @pointerdown.stop.prevent="beginZoneResize($event, zone, handle)"
                    />
                  </div>
                  <div
                    v-if="draftZone"
                    class="zone-preview zone-draft"
                    :style="rectangleStyle(draftZone)"
                  />
                </div>
              </div>
            </NSpin>
          </div>
          <NAlert
            v-if="redaction.renderedPage && !redaction.hasSelectableText"
            type="info"
            class="no-text-alert"
          >
            {{ t("redaction.noSelectableText") }}
          </NAlert>
          <NText depth="3" class="selection-hint">{{
            isZoneMode ? t("redaction.zoneHint") : t("redaction.selectionHint")
          }}</NText>
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
        <NButton block :disabled="redaction.phase === 'running'" @click="router.push('/')">{{
          t("common.home")
        }}</NButton>

        <NCard size="small" embedded :title="t('redaction.source')">
          <template v-if="redaction.source">
            <NThing :title="redaction.source.name" :description="redaction.source.path">
              <NText depth="3">{{
                t("redaction.pageCount", { count: redaction.source.pageCount })
              }}</NText>
            </NThing>
            <NSpace vertical>
              <NButton
                block
                :disabled="redaction.phase === 'running'"
                @click="redaction.choosePdfFile"
                >{{ t("redaction.replaceSource") }}</NButton
              >
              <NButton
                block
                type="error"
                :disabled="redaction.phase === 'running'"
                @click="redaction.resetPreparation"
                >{{ t("redaction.removeSource") }}</NButton
              >
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
                :disabled="
                  redaction.phase === 'running' || !redaction.canGoPrevious || redaction.loadingPage
                "
                @click="redaction.goToPreviousPage"
              >
                {{ t("redaction.previousPage") }}
              </NButton>
              <NInputNumber
                :value="redaction.currentPage"
                :min="1"
                :max="redaction.source.pageCount"
                :show-button="false"
                :disabled="redaction.phase === 'running'"
                class="page-number"
                @update:value="redaction.goToPage"
              />
              <NButton
                :disabled="
                  redaction.phase === 'running' || !redaction.canGoNext || redaction.loadingPage
                "
                @click="redaction.goToNextPage"
              >
                {{ t("redaction.nextPage") }}
              </NButton>
            </NSpace>
            <NText depth="3" class="page-count">{{
              t("redaction.ofPages", { count: redaction.source.pageCount })
            }}</NText>
            <NSpace align="center" justify="center" class="zoom-controls">
              <NButton
                size="small"
                :disabled="redaction.phase === 'running' || redaction.zoom <= 0.75"
                @click="redaction.zoomOut"
                >−</NButton
              >
              <NButton
                size="small"
                :disabled="redaction.phase === 'running'"
                @click="redaction.resetZoom"
                >{{ Math.round(redaction.zoom * 100) }}%</NButton
              >
              <NButton
                size="small"
                :disabled="redaction.phase === 'running' || redaction.zoom >= 2"
                @click="redaction.zoomIn"
                >+</NButton
              >
            </NSpace>
          </NCard>

          <NCard size="small" embedded :title="t('redaction.destination')">
            <label>
              {{ t("redaction.outputName") }}
              <NInput
                :value="redaction.outputName"
                :disabled="redaction.phase === 'running'"
                :placeholder="t('redaction.outputPlaceholder')"
                @update:value="redaction.renameOutput"
              />
            </label>
            <label>
              {{ t("redaction.destinationPath") }}
              <NInput
                :value="redaction.destination"
                :disabled="redaction.phase === 'running'"
                :placeholder="t('redaction.destinationPlaceholder')"
                @update:value="redaction.chooseDestination"
              />
            </label>
            <NButton
              :disabled="redaction.phase === 'running'"
              @click="redaction.chooseDestinationFolder"
              >{{ t("redaction.browse") }}</NButton
            >
          </NCard>
          <NButton
            block
            type="primary"
            :disabled="!redaction.canRequestSummary"
            @click="openSummary"
          >
            {{ t("redaction.review") }}
          </NButton>
        </template>
      </aside>
    </NLayoutSider>
  </NLayout>

  <NModal
    v-model:show="showSummary"
    preset="card"
    :title="t('redaction.summaryTitle')"
    style="width: min(92vw, 42rem)"
  >
    <NThing :title="redaction.source?.name" :description="redaction.source?.path" />
    <NList bordered>
      <template v-for="selection in redaction.selectionSummary" :key="selection.page">
        <NListItem v-for="word in selection.words" :key="word.index">{{
          t("redaction.selection", { page: selection.page, word: word.text })
        }}</NListItem>
        <NListItem v-for="(zone, index) in selection.zones" :key="zone.id">{{
          t("redaction.zoneSelection", { page: selection.page, zone: index + 1 })
        }}</NListItem>
      </template>
    </NList>
    <p>
      <strong>{{ t("redaction.output") }}</strong> {{ redaction.outputPreview?.outputPath }}
    </p>
    <NAlert type="warning">{{ t("redaction.irreversibleWarning") }}</NAlert>
    <template #action>
      <NSpace justify="end">
        <NButton @click="showSummary = false">{{ t("redaction.back") }}</NButton>
        <NButton
          type="primary"
          @click="
            showSummary = false;
            redaction.confirmRedaction();
          "
        >
          {{ t("redaction.confirm") }}
        </NButton>
      </NSpace>
    </template>
  </NModal>
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

.selection-mode {
  margin: 0;
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

.zone-drawing-layer {
  inset: 0;
  pointer-events: none;
  position: absolute;
  touch-action: none;
}

.zone-drawing-layer.interactive {
  cursor: crosshair;
  pointer-events: auto;
}

.zone-preview {
  background: #000;
  border: 2px solid #f59e0b;
  box-sizing: border-box;
  cursor: move;
  position: absolute;
}

.zone-draft {
  pointer-events: none;
}

.zone-handle {
  background: #f59e0b;
  border: 1px solid #78350f;
  border-radius: 50%;
  box-sizing: border-box;
  cursor: nwse-resize;
  height: 0.75rem;
  padding: 0;
  position: absolute;
  width: 0.75rem;
}

.zone-handle.top-left {
  left: -0.45rem;
  top: -0.45rem;
}

.zone-handle.top-right {
  cursor: nesw-resize;
  right: -0.45rem;
  top: -0.45rem;
}

.zone-handle.bottom-left {
  bottom: -0.45rem;
  cursor: nesw-resize;
  left: -0.45rem;
}

.zone-handle.bottom-right {
  bottom: -0.45rem;
  right: -0.45rem;
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
