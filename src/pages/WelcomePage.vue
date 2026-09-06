<script setup lang="ts">
import { NButton, NLayout, NLayoutContent, NSelect, NSpace, NText, NTooltip } from "naive-ui";
import { computed, ref } from "vue";
import { useI18n } from "vue-i18n";
import { useRouter } from "vue-router";

import packageInfo from "../../package.json";
import BrandMark from "../components/BrandMark.vue";
import LineIcon from "../components/LineIcon.vue";
import UpdateDialog from "../components/UpdateDialog.vue";
import { type AppearanceMode, useAppearance } from "../composables/useAppearance";
import type { SupportedLocale } from "../i18n";

const { t, locale } = useI18n();
const router = useRouter();
const { appearanceMode, selectAppearance } = useAppearance();
const showUpdate = ref(false);

const themeOptions = computed(() => [
  { label: t("appearance.system"), value: "system" },
  { label: t("appearance.light"), value: "light" },
  { label: t("appearance.dark"), value: "dark" },
]);
const languageOptions = computed(() => [
  { label: t("common.english"), value: "en" },
  { label: t("common.french"), value: "fr" },
]);

function selectLanguage(value: SupportedLocale) {
  locale.value = value;
}
</script>

<template>
  <NLayout class="application-shell">
    <NLayoutContent content-style="padding: 0;">
      <main class="welcome-page">
        <aside class="welcome-rail">
          <BrandMark :size="28" />
        </aside>
        <div class="welcome-content">
          <header class="welcome-header">
            <div>
              <h1>{{ t("welcome.heading") }}</h1>
            </div>
            <NText depth="3">{{ t("welcome.body") }}</NText>
          </header>

          <section class="tools" :aria-label="t('welcome.tools')">
            <NTooltip>
              <template #trigger>
                <NButton
                  block
                  class="tool-action tool-action--merge"
                  size="large"
                  @click="router.push('/merge')"
                >
                  <span class="tool-action__content">
                    <span class="tool-action__icon"><LineIcon name="merge" :size="32" /></span>
                    <span class="tool-action__title">{{ t("welcome.merge") }}</span>
                  </span>
                </NButton>
              </template>
              {{ t("welcome.mergeDescription") }}
            </NTooltip>
            <NTooltip>
              <template #trigger>
                <NButton
                  block
                  class="tool-action tool-action--split"
                  size="large"
                  @click="router.push('/split')"
                >
                  <span class="tool-action__content">
                    <span class="tool-action__icon"><LineIcon name="split" :size="32" /></span>
                    <span class="tool-action__title">{{ t("welcome.split") }}</span>
                  </span>
                </NButton>
              </template>
              {{ t("welcome.splitDescription") }}
            </NTooltip>
            <NTooltip>
              <template #trigger>
                <NButton
                  block
                  class="tool-action tool-action--redact"
                  size="large"
                  @click="router.push('/redact')"
                >
                  <span class="tool-action__content">
                    <span class="tool-action__icon"><LineIcon name="redact" :size="32" /></span>
                    <span class="tool-action__title">{{ t("welcome.redact") }}</span>
                  </span>
                </NButton>
              </template>
              {{ t("welcome.redactDescription") }}
            </NTooltip>
          </section>

          <NSpace class="preferences" align="center" justify="end" size="small" wrap>
            <label class="preference-control">
              <NText depth="3">{{ t("common.theme") }}</NText>
              <NSelect
                size="small"
                :value="appearanceMode"
                :options="themeOptions"
                @update:value="selectAppearance($event as AppearanceMode)"
              />
            </label>
            <label class="preference-control">
              <NText depth="3">{{ t("welcome.language") }}</NText>
              <NSelect
                size="small"
                :value="locale"
                :options="languageOptions"
                @update:value="selectLanguage($event as SupportedLocale)"
              />
            </label>
            <NButton
              quaternary
              size="small"
              class="version-link"
              :title="t('welcome.version')"
              @click="showUpdate = true"
            >
              PDFForge v{{ packageInfo.version }}
            </NButton>
          </NSpace>
        </div>
      </main>
    </NLayoutContent>
  </NLayout>
  <UpdateDialog v-model:show="showUpdate" />
</template>

<style scoped>
.application-shell {
  background: var(--bg);
  min-height: 100vh;
}

.welcome-page {
  display: grid;
  grid-template-columns: 3.75rem minmax(0, 1fr);
  min-height: 100vh;
}

.welcome-content {
  align-self: center;
  margin: 2.25rem auto 0;
  padding: 2rem;
  width: min(100% - 4rem, 39rem);
}

.welcome-rail {
  align-items: center;
  background: var(--surface);
  border-right: 1px solid var(--border);
  display: flex;
  flex-direction: column;
  padding-top: 3.2rem;
}

.welcome-header {
  align-items: flex-end;
  border-bottom: 1px solid var(--border);
  display: flex;
  justify-content: space-between;
  padding-bottom: 1rem;
}

.welcome-header h1 {
  font-size: 0.875rem;
  font-weight: 600;
  letter-spacing: -0.01em;
  margin: 0;
}

.welcome-header :deep(.n-text) {
  font-size: 0.72rem;
  max-width: 14rem;
}

.tools {
  display: grid;
  gap: 0.75rem;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  margin-top: 1.25rem;
}

.tool-action {
  --n-color: transparent !important;
  --n-color-focus: var(--tool-surface) !important;
  --n-color-hover: var(--tool-surface) !important;
  --n-color-pressed: var(--tool-surface) !important;
  --n-text-color: var(--text) !important;
  --n-text-color-focus: var(--text) !important;
  --n-text-color-hover: var(--text) !important;
  --n-text-color-pressed: var(--text) !important;
  background: transparent;
  border: 1px solid var(--tool-border);
  color: var(--text);
  aspect-ratio: 1;
  height: auto;
  min-height: 0;
  padding: 0.2rem;
  text-align: center;
}

.tool-action:hover {
  background: var(--tool-surface);
  border-color: var(--tool-color);
}

.tool-action:is(:hover, :focus-visible) :deep(.n-button__content) {
  color: var(--text) !important;
}

.tool-action--merge {
  --tool-border: var(--universe-merge-border);
  --tool-color: var(--universe-merge);
  --tool-surface: var(--universe-merge-soft);
}

.tool-action--split {
  --tool-border: var(--universe-split-border);
  --tool-color: var(--universe-split);
  --tool-surface: var(--universe-split-soft);
}

.tool-action--redact {
  --tool-border: var(--universe-redact-border);
  --tool-color: var(--universe-redact);
  --tool-surface: var(--universe-redact-soft);
}

.tool-action :deep(.n-button__content) {
  min-width: 0;
  white-space: normal;
}

.tool-action__content {
  align-content: center;
  align-items: center;
  display: grid;
  gap: 0.5rem;
  grid-template-columns: minmax(0, 1fr);
  justify-items: center;
  width: 100%;
  padding: 0.85rem;
}

.tool-action__icon {
  align-items: center;
  background: var(--tool-surface);
  border: 1px solid var(--tool-border);
  border-radius: 50%;
  color: var(--tool-color);
  display: flex;
  height: 4.25rem;
  justify-content: center;
  width: 4.25rem;
}

.tool-action__title {
  font-size: 0.78rem;
  font-weight: 650;
}

.preferences {
  border-top: 1px solid var(--border);
  margin-top: 1.5rem;
  padding-top: 1rem;
}

.preference-control {
  display: grid;
  grid-template-columns: auto 7.75rem;
  align-items: center;
  gap: 0.5rem;
}

.version-link {
  color: var(--text-tertiary);
  font-size: 0.6875rem;
}

@media (max-width: 34rem) {
  .welcome-content {
    padding: 1.25rem;
    width: auto;
  }
  .welcome-header {
    align-items: start;
    flex-direction: column;
    gap: 0.5rem;
  }
  .tools {
    gap: 0.5rem;
  }
  .preferences {
    justify-content: start;
  }
}
</style>
