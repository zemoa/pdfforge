<script setup lang="ts">
import { NButton, NLayout, NLayoutContent, NSelect, NSpace, NText } from "naive-ui";
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { useRouter } from "vue-router";

import BrandMark from "../components/BrandMark.vue";
import LineIcon from "../components/LineIcon.vue";
import { type AppearanceMode, useAppearance } from "../composables/useAppearance";
import type { SupportedLocale } from "../i18n";

const { t, locale } = useI18n();
const router = useRouter();
const { appearanceMode, selectAppearance } = useAppearance();

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
          <BrandMark :size="28" /><span class="welcome-rail__line" />
        </aside>
        <div class="welcome-content">
          <header class="welcome-header">
            <div>
              <span class="eyebrow">PDFForge</span>
              <h1>{{ t("welcome.heading") }}</h1>
            </div>
            <NText depth="3">{{ t("welcome.body") }}</NText>
          </header>

          <section class="tools" :aria-label="t('welcome.tools')">
            <NButton block class="tool-action" size="large" @click="router.push('/merge')">
              <span class="tool-action__content">
                <LineIcon name="merge" :size="18" /><span class="tool-action__title">{{
                  t("welcome.merge")
                }}</span>
                <span class="tool-action__description">{{ t("welcome.mergeDescription") }}</span>
              </span>
            </NButton>
            <NButton block class="tool-action" size="large" @click="router.push('/split')">
              <span class="tool-action__content">
                <LineIcon name="split" :size="18" /><span class="tool-action__title">{{
                  t("welcome.split")
                }}</span>
                <span class="tool-action__description">{{ t("welcome.splitDescription") }}</span>
              </span>
            </NButton>
            <NButton block class="tool-action" size="large" @click="router.push('/redact')">
              <span class="tool-action__content">
                <LineIcon name="redact" :size="18" /><span class="tool-action__title">{{
                  t("welcome.redact")
                }}</span>
                <span class="tool-action__description">{{ t("welcome.redactDescription") }}</span>
              </span>
            </NButton>
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
          </NSpace>
        </div>
      </main>
    </NLayoutContent>
  </NLayout>
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
  gap: 1.5rem;
  padding-top: 3.2rem;
}

.welcome-rail__line {
  background: var(--border);
  height: 2rem;
  width: 1px;
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
  margin: 0.3rem 0 0;
}

.welcome-header :deep(.n-text) {
  font-size: 0.72rem;
  max-width: 14rem;
}

.eyebrow {
  color: var(--text);
  font-size: 0.78rem;
  font-weight: 700;
}

.tools {
  display: grid;
  gap: 0.35rem;
  margin-top: 1.25rem;
}

.tool-action {
  --n-color: transparent !important;
  --n-color-focus: var(--surface) !important;
  --n-color-hover: var(--surface) !important;
  --n-color-pressed: var(--surface-secondary) !important;
  --n-text-color: var(--text) !important;
  --n-text-color-focus: var(--text) !important;
  --n-text-color-hover: var(--text) !important;
  --n-text-color-pressed: var(--text) !important;
  background: transparent;
  border: 1px solid transparent;
  color: var(--text);
  height: auto;
  min-height: 4.4rem;
  padding: 0.2rem;
  text-align: left;
}

.tool-action:hover {
  background: var(--surface);
  border-color: var(--border);
}

.tool-action__content {
  align-items: center;
  display: grid;
  gap: 0.2rem 0.7rem;
  grid-template-columns: auto minmax(0, 1fr);
  width: 100%;
  padding: 0.65rem 0.75rem;
}

.tool-action__title {
  font-size: 0.78rem;
  font-weight: 650;
}

.tool-action__description {
  color: var(--text-secondary);
  font-size: 0.6875rem;
  font-weight: 400;
  line-height: 1.35;
  grid-column: 2;
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
  .preferences {
    justify-content: start;
  }
}
</style>
