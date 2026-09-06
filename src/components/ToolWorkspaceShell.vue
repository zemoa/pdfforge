<script setup lang="ts">
import { NButton, NTooltip } from "naive-ui";
import { useI18n } from "vue-i18n";
import { useRouter } from "vue-router";

import BrandMark from "./BrandMark.vue";
import LineIcon from "./LineIcon.vue";
import { useAppearance } from "../composables/useAppearance";

type ToolId = "merge" | "split" | "redact";

const props = withDefaults(
  defineProps<{
    activeTool: ToolId;
    navigationDisabled?: boolean;
    title: string;
  }>(),
  { navigationDisabled: false },
);

const { t } = useI18n();
const router = useRouter();
const { appearanceMode, selectAppearance } = useAppearance();
const leftPanelCollapsed = defineModel<boolean>("leftPanelCollapsed", { default: false });
const rightPanelCollapsed = defineModel<boolean>("rightPanelCollapsed", { default: false });

const tools: ReadonlyArray<{
  id: ToolId;
  label: "merge" | "split" | "redact";
  icon: "merge" | "split" | "redact";
}> = [
  { id: "merge", label: "merge", icon: "merge" },
  { id: "split", label: "split", icon: "split" },
  { id: "redact", label: "redact", icon: "redact" },
];

function navigate(path: string) {
  if (!props.navigationDisabled) void router.push(path);
}

function cycleAppearance() {
  const nextMode = appearanceMode.value === "light" ? "dark" : "light";
  selectAppearance(nextMode);
}
</script>

<template>
  <section
    class="tool-workspace-shell"
    :class="{
      'tool-workspace-shell--left-collapsed': leftPanelCollapsed,
      'tool-workspace-shell--right-collapsed': rightPanelCollapsed,
      'tool-workspace-shell--without-right': !$slots['right-panel'],
    }"
  >
    <nav class="tool-rail" :aria-label="t('common.navigation')">
      <NTooltip placement="right">
        <template #trigger>
          <NButton
            quaternary
            class="brand-button"
            :aria-label="t('common.home')"
            :disabled="navigationDisabled"
            @click="navigate('/')"
          >
            <BrandMark :size="28" />
          </NButton>
        </template>
        {{ t("common.home") }}
      </NTooltip>

      <div class="tool-rail__tools">
        <NTooltip v-for="tool in tools" :key="tool.id" placement="right">
          <template #trigger>
            <NButton
              quaternary
              class="tool-rail__button"
              :class="{ 'tool-rail__button--active': activeTool === tool.id }"
              :aria-current="activeTool === tool.id ? 'page' : undefined"
              :aria-label="t(`common.tools.${tool.label}`)"
              :disabled="navigationDisabled"
              @click="navigate(`/${tool.id}`)"
            >
              <LineIcon :name="tool.icon" :size="18" />
            </NButton>
          </template>
          {{ t(`common.tools.${tool.label}`) }}
        </NTooltip>
      </div>
      <div class="tool-rail__bottom">
        <NTooltip placement="right">
          <template #trigger>
            <NButton
              quaternary
              class="tool-rail__button"
              :aria-label="t('common.theme')"
              @click="cycleAppearance"
            >
              <LineIcon name="theme" :size="17" />
            </NButton>
          </template>
          {{ t("common.theme") }}
        </NTooltip>
        <NButton
          quaternary
          class="tool-rail__button"
          :aria-label="t('common.preferences')"
          @click="navigate('/')"
        >
          <LineIcon name="settings" :size="17" />
        </NButton>
      </div>
    </nav>

    <header class="tool-topbar">
      <div class="tool-topbar__title">
        <span class="tool-topbar__brand">PDFForge</span>
        <h1>{{ title }}</h1>
      </div>
      <div class="tool-topbar__actions">
        <slot name="topbar" />
        <span class="local-status"><i />{{ t("common.local") }}</span>
      </div>
    </header>

    <aside
      class="tool-panel tool-panel--left"
      :class="{ 'tool-panel--collapsed': leftPanelCollapsed }"
    >
      <div class="tool-panel__body"><slot name="left-panel" /></div>
      <NButton
        quaternary
        class="panel-toggle panel-toggle--left"
        :aria-label="t(leftPanelCollapsed ? 'common.expandPanel' : 'common.collapsePanel')"
        @click="leftPanelCollapsed = !leftPanelCollapsed"
      >
        {{ leftPanelCollapsed ? "›" : "‹" }}
      </NButton>
    </aside>

    <main class="tool-main"><slot /></main>

    <aside
      v-if="$slots['right-panel']"
      class="tool-panel tool-panel--right"
      :class="{ 'tool-panel--collapsed': rightPanelCollapsed }"
    >
      <NButton
        quaternary
        class="panel-toggle panel-toggle--right"
        :aria-label="t(rightPanelCollapsed ? 'common.expandPanel' : 'common.collapsePanel')"
        @click="rightPanelCollapsed = !rightPanelCollapsed"
      >
        {{ rightPanelCollapsed ? "‹" : "›" }}
      </NButton>
      <div class="tool-panel__body"><slot name="right-panel" /></div>
    </aside>

    <footer v-if="$slots.footer" class="tool-statusbar"><slot name="footer" /></footer>
  </section>
</template>

<style scoped>
.tool-workspace-shell {
  --rail-width: 3.75rem;
  --panel-width: 14.25rem;
  box-sizing: border-box;
  display: grid;
  grid-template-columns: var(--rail-width) var(--panel-width) minmax(0, 1fr) var(--panel-width);
  grid-template-rows: 3rem minmax(0, 1fr) auto;
  height: 100vh;
  min-height: 36rem;
  padding-top: 2.25rem;
}

.tool-workspace-shell--without-right {
  grid-template-columns: var(--rail-width) var(--panel-width) minmax(0, 1fr);
}

.tool-workspace-shell--left-collapsed {
  grid-template-columns: var(--rail-width) 0 minmax(0, 1fr) var(--panel-width);
}

.tool-workspace-shell--without-right.tool-workspace-shell--left-collapsed {
  grid-template-columns: var(--rail-width) 0 minmax(0, 1fr);
}

.tool-workspace-shell--right-collapsed {
  grid-template-columns: var(--rail-width) var(--panel-width) minmax(0, 1fr) 0;
}

.tool-workspace-shell--left-collapsed.tool-workspace-shell--right-collapsed {
  grid-template-columns: var(--rail-width) 0 minmax(0, 1fr) 0;
}

.tool-rail {
  background: var(--surface);
  border-right: 1px solid var(--border);
  display: flex;
  flex-direction: column;
  grid-column: 1;
  grid-row: 1 / -1;
  padding: 0.5rem;
}

.brand-button,
.tool-rail__button {
  height: 2.5rem;
  padding: 0;
  width: 2.75rem;
}

.brand-button {
  color: var(--accent);
}

.tool-rail__tools {
  display: grid;
  gap: 0.45rem;
  margin-top: 1.75rem;
}

.tool-rail__bottom {
  display: grid;
  gap: 0.35rem;
  margin-top: auto;
}

.tool-rail__button {
  font-size: 1.25rem;
}

.tool-rail__button--active {
  --n-color-hover: var(--accent-soft) !important;
  --n-color-pressed: var(--accent-soft) !important;
  --n-text-color-hover: var(--accent) !important;
  --n-text-color-pressed: var(--accent) !important;
  background: var(--accent-soft);
  color: var(--accent);
}

.tool-rail__button:not(.tool-rail__button--active):hover,
.brand-button:hover {
  background: var(--surface-secondary);
  color: var(--text);
}

.tool-topbar {
  align-items: center;
  background: var(--surface);
  border-bottom: 1px solid var(--border);
  display: flex;
  grid-column: 2 / -1;
  grid-row: 1;
  justify-content: space-between;
  min-width: 0;
  padding: 0 8rem 0 1.25rem;
}

.tool-topbar__title {
  align-items: baseline;
  display: flex;
  gap: 0.65rem;
  min-width: 0;
}

.tool-topbar__brand {
  color: var(--text);
  font-size: 0.8125rem;
  font-weight: 700;
  letter-spacing: 0.06em;
}

.tool-topbar h1 {
  color: var(--text-secondary);
  font-size: 0.8125rem;
  font-weight: 550;
  letter-spacing: -0.01em;
  margin: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.tool-topbar__actions {
  align-items: center;
  display: flex;
  gap: 0.75rem;
}

.local-status {
  align-items: center;
  color: var(--text-secondary);
  display: inline-flex;
  font-size: 0.6875rem;
  gap: 0.38rem;
  white-space: nowrap;
}

.local-status i {
  background: var(--success);
  border-radius: 50%;
  height: 0.38rem;
  width: 0.38rem;
}

.tool-panel {
  background: var(--bg);
  min-width: 0;
  overflow: visible;
  position: relative;
  transition: width 160ms ease;
}

.tool-panel--left {
  border-right: 1px solid var(--border);
  grid-column: 2;
  grid-row: 2;
}

.tool-panel--right {
  border-left: 1px solid var(--border);
  grid-column: 4;
  grid-row: 2;
}

.tool-panel--collapsed {
  width: 0;
}

.tool-panel--collapsed .tool-panel__body {
  opacity: 0;
  pointer-events: none;
  visibility: hidden;
}

.tool-panel__body {
  box-sizing: border-box;
  display: grid;
  gap: 0.75rem;
  height: 100%;
  overflow: auto;
  padding: 1rem;
  transition: opacity 100ms ease;
}

.panel-toggle {
  align-items: center;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  display: flex;
  height: 1.75rem;
  justify-content: center;
  padding: 0;
  position: absolute;
  top: 50%;
  width: 1.75rem;
  z-index: 2;
}

.panel-toggle--left {
  right: -0.875rem;
}

.panel-toggle--right {
  left: -0.875rem;
}

.tool-main {
  background: var(--surface-secondary);
  grid-column: 3;
  grid-row: 2;
  min-width: 0;
  overflow: hidden;
}

.tool-statusbar {
  align-items: center;
  background: var(--surface);
  border-top: 1px solid var(--border);
  display: flex;
  grid-column: 2 / -1;
  grid-row: 3;
  min-height: 3.5rem;
  padding: 0 1.25rem;
}

@media (max-width: 58rem) {
  .tool-workspace-shell {
    grid-template-columns: var(--rail-width) minmax(0, 1fr);
  }

  .tool-workspace-shell--left-collapsed,
  .tool-workspace-shell--right-collapsed,
  .tool-workspace-shell--left-collapsed.tool-workspace-shell--right-collapsed {
    grid-template-columns: var(--rail-width) minmax(0, 1fr);
  }

  .tool-workspace-shell--without-right {
    grid-template-columns: var(--rail-width) minmax(0, 1fr);
  }

  .tool-topbar,
  .tool-statusbar {
    grid-column: 2;
  }

  .tool-main {
    grid-column: 2;
  }

  .tool-panel {
    position: fixed;
    top: 5.5rem;
    bottom: 0;
    width: min(18rem, calc(100vw - 3.75rem));
    z-index: 4;
  }

  .tool-panel--left {
    grid-column: auto;
    grid-row: auto;
    left: 3.75rem;
  }

  .tool-panel--right {
    grid-column: auto;
    grid-row: auto;
    right: 0;
  }

  .tool-panel--collapsed {
    width: 0;
  }
}
</style>
