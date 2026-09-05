<script setup lang="ts">
import { NButton, NTooltip } from "naive-ui";
import { useI18n } from "vue-i18n";
import { useRouter } from "vue-router";

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
const leftPanelCollapsed = defineModel<boolean>("leftPanelCollapsed", { default: false });
const rightPanelCollapsed = defineModel<boolean>("rightPanelCollapsed", { default: false });

const tools: ReadonlyArray<{ id: ToolId; label: "merge" | "split" | "redact"; symbol: string }> = [
  { id: "merge", label: "merge", symbol: "↗" },
  { id: "split", label: "split", symbol: "÷" },
  { id: "redact", label: "redact", symbol: "▰" },
];

function navigate(path: string) {
  if (!props.navigationDisabled) void router.push(path);
}
</script>

<template>
  <section
    class="tool-workspace-shell"
    :class="{
      'tool-workspace-shell--left-collapsed': leftPanelCollapsed,
      'tool-workspace-shell--right-collapsed': rightPanelCollapsed,
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
            <span aria-hidden="true">F</span>
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
              <span aria-hidden="true">{{ tool.symbol }}</span>
            </NButton>
          </template>
          {{ t(`common.tools.${tool.label}`) }}
        </NTooltip>
      </div>
    </nav>

    <header class="tool-topbar">
      <div class="tool-topbar__title">
        <span class="tool-topbar__eyebrow">PDFForge</span>
        <h1>{{ title }}</h1>
      </div>
      <slot name="topbar" />
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
  --panel-width: 17.5rem;
  box-sizing: border-box;
  display: grid;
  grid-template-columns: var(--rail-width) var(--panel-width) minmax(0, 1fr) var(--panel-width);
  grid-template-rows: 3.25rem minmax(0, 1fr) auto;
  height: 100vh;
  min-height: 36rem;
  padding-top: 2.25rem;
}

.tool-workspace-shell--left-collapsed {
  grid-template-columns: var(--rail-width) 0 minmax(0, 1fr) var(--panel-width);
}

.tool-workspace-shell--right-collapsed {
  grid-template-columns: var(--rail-width) var(--panel-width) minmax(0, 1fr) 0;
}

.tool-workspace-shell--left-collapsed.tool-workspace-shell--right-collapsed {
  grid-template-columns: var(--rail-width) 0 minmax(0, 1fr) 0;
}

.tool-rail {
  background: color-mix(in srgb, var(--n-color) 96%, var(--n-base-color));
  border-right: 1px solid var(--n-border-color);
  display: flex;
  flex-direction: column;
  grid-column: 1;
  grid-row: 1 / -1;
  padding: 0.5rem;
}

.brand-button,
.tool-rail__button {
  height: 2.75rem;
  padding: 0;
  width: 2.75rem;
}

.brand-button {
  color: var(--n-primary-color);
  font-size: 1.35rem;
  font-style: italic;
  font-weight: 850;
}

.tool-rail__tools {
  display: grid;
  gap: 0.45rem;
  margin-top: 1.75rem;
}

.tool-rail__button {
  font-size: 1.25rem;
}

.tool-rail__button--active {
  background: color-mix(in srgb, var(--n-primary-color) 16%, transparent);
  color: var(--n-primary-color);
}

.tool-topbar {
  align-items: center;
  border-bottom: 1px solid var(--n-border-color);
  display: flex;
  grid-column: 2 / -1;
  grid-row: 1;
  justify-content: space-between;
  min-width: 0;
  padding: 0 1.25rem;
}

.tool-topbar__title {
  align-items: baseline;
  display: flex;
  gap: 0.65rem;
  min-width: 0;
}

.tool-topbar__eyebrow {
  color: var(--n-text-color-3);
  font-size: 0.75rem;
  font-weight: 700;
  letter-spacing: 0.06em;
  text-transform: uppercase;
}

.tool-topbar h1 {
  font-size: 1rem;
  font-weight: 650;
  letter-spacing: -0.015em;
  margin: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.tool-panel {
  background: color-mix(in srgb, var(--n-color) 98%, var(--n-base-color));
  min-width: 0;
  overflow: visible;
  position: relative;
  transition: width 160ms ease;
}

.tool-panel--left {
  border-right: 1px solid var(--n-border-color);
  grid-column: 2;
  grid-row: 2;
}

.tool-panel--right {
  border-left: 1px solid var(--n-border-color);
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
  gap: 0.9rem;
  height: 100%;
  overflow: auto;
  padding: 1rem;
  transition: opacity 100ms ease;
}

.panel-toggle {
  align-items: center;
  background: var(--n-color);
  border: 1px solid var(--n-border-color);
  border-radius: 999px;
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
  grid-column: 3;
  grid-row: 2;
  min-width: 0;
  overflow: hidden;
}

.tool-statusbar {
  align-items: center;
  border-top: 1px solid var(--n-border-color);
  display: flex;
  grid-column: 2 / -1;
  grid-row: 3;
  min-height: 3.25rem;
  padding: 0 1rem;
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
