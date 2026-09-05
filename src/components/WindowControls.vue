<script setup lang="ts">
import { NButton } from "naive-ui";
import { useI18n } from "vue-i18n";

import { windowClient } from "../application/windowClient";

const { t } = useI18n();

function startDragging(event: MouseEvent) {
  if (event.button === 0 && event.detail === 1) void windowClient.startDragging();
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
</template>

<style scoped>
.window-drag-region {
  position: fixed;
  z-index: 10;
  top: 0;
  right: 0;
  left: 0;
  height: 2.25rem;
  background-color: color-mix(in srgb, Canvas 94%, CanvasText);
  border-bottom: 1px solid color-mix(in srgb, Canvas 78%, CanvasText);
  cursor: grab;
  user-select: none;
}

.window-drag-region:active {
  cursor: grabbing;
}

.window-controls {
  display: flex;
  position: absolute;
  top: 0.25rem;
  right: 0.25rem;
  cursor: default;
}

.window-control {
  width: 2.25rem;
  height: 1.75rem;
  font-size: 1.125rem;
}

.window-control--close {
  font-size: 1.375rem;
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
