<script setup lang="ts">
import { NButton } from "naive-ui";
import { useI18n } from "vue-i18n";

import { windowClient, type WindowResizeDirection } from "../application/windowClient";

const { t } = useI18n();

const resizeHandles = [
  { direction: "North", position: "north" },
  { direction: "East", position: "east" },
  { direction: "South", position: "south" },
  { direction: "West", position: "west" },
  { direction: "NorthEast", position: "north-east" },
  { direction: "SouthEast", position: "south-east" },
  { direction: "SouthWest", position: "south-west" },
  { direction: "NorthWest", position: "north-west" },
] as const satisfies ReadonlyArray<{
  direction: WindowResizeDirection;
  position: string;
}>;

function startDragging(event: MouseEvent) {
  if (event.button === 0 && event.detail === 1) void windowClient.startDragging();
}

function startResizeDragging(event: MouseEvent, direction: WindowResizeDirection) {
  if (event.button === 0) void windowClient.startResizeDragging(direction);
}

function toggleMaximize() {
  void windowClient.toggleMaximize();
}
</script>

<template>
  <div
    class="window-drag-region"
    aria-hidden="true"
    @mousedown="startDragging"
    @dblclick="toggleMaximize"
  >
    <div class="window-controls" aria-label="Window controls" @mousedown.stop @dblclick.stop>
      <NButton
        quaternary
        class="window-control"
        :title="t('window.minimize')"
        @click="windowClient.minimize"
      >
        <span aria-hidden="true">−</span>
        <span class="visually-hidden">{{ t("window.minimize") }}</span>
      </NButton>
      <NButton
        quaternary
        class="window-control"
        :title="t('window.maximize')"
        @click="toggleMaximize"
      >
        <span aria-hidden="true">□</span>
        <span class="visually-hidden">{{ t("window.maximize") }}</span>
      </NButton>
      <NButton
        quaternary
        class="window-control window-control--close"
        :title="t('window.close')"
        @click="windowClient.close"
      >
        <span aria-hidden="true">×</span>
        <span class="visually-hidden">{{ t("window.close") }}</span>
      </NButton>
    </div>
  </div>
  <div
    v-for="handle in resizeHandles"
    :key="handle.direction"
    class="window-resize-handle"
    :class="`window-resize-handle--${handle.position}`"
    aria-hidden="true"
    @mousedown.stop.prevent="startResizeDragging($event, handle.direction)"
  ></div>
</template>

<style scoped>
.window-drag-region {
  position: fixed;
  z-index: 10;
  top: 0;
  right: 0;
  left: 0;
  height: 2.25rem;
  background-color: var(--surface);
  border-bottom: 1px solid var(--border);
  cursor: grab;
  user-select: none;
}

.window-drag-region:active {
  cursor: grabbing;
}

.window-controls {
  display: flex;
  position: absolute;
  top: 0.2rem;
  right: 0.3rem;
  cursor: default;
}

.window-control {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 2.1rem;
  height: 1.7rem;
  padding: 0;
  font-size: 1.125rem;
  line-height: 1;
}

.window-control :deep(.n-button__content) {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 100%;
  height: 100%;
  line-height: 1;
}

.window-control--close {
  font-size: 1.375rem;
}

.window-control:hover {
  background: var(--surface-secondary);
  color: var(--text);
}

.window-control--close:hover {
  background: var(--danger);
  color: #fff;
}

.window-resize-handle {
  position: fixed;
  z-index: 20;
  touch-action: none;
  user-select: none;
}

.window-resize-handle--north,
.window-resize-handle--south {
  right: 0.625rem;
  left: 0.625rem;
  height: 0.375rem;
  cursor: ns-resize;
}

.window-resize-handle--north {
  top: 0;
}

.window-resize-handle--south {
  bottom: 0;
}

.window-resize-handle--east,
.window-resize-handle--west {
  top: 0.625rem;
  bottom: 0.625rem;
  width: 0.375rem;
  cursor: ew-resize;
}

.window-resize-handle--east {
  right: 0;
}

.window-resize-handle--west {
  left: 0;
}

.window-resize-handle--north-east,
.window-resize-handle--south-east,
.window-resize-handle--south-west,
.window-resize-handle--north-west {
  z-index: 21;
  width: 0.625rem;
  height: 0.625rem;
}

.window-resize-handle--north-east,
.window-resize-handle--south-west {
  cursor: nesw-resize;
}

.window-resize-handle--south-east,
.window-resize-handle--north-west {
  cursor: nwse-resize;
}

.window-resize-handle--north-east,
.window-resize-handle--north-west {
  top: 0;
}

.window-resize-handle--south-east,
.window-resize-handle--south-west {
  bottom: 0;
}

.window-resize-handle--north-east,
.window-resize-handle--south-east {
  right: 0;
}

.window-resize-handle--south-west,
.window-resize-handle--north-west {
  left: 0;
}

.visually-hidden {
  position: absolute;
  width: 1px;
  height: 1px;
  overflow: hidden;
  clip: rect(0 0 0 0);
  white-space: nowrap;
}
</style>
