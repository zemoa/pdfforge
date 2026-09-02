<script setup lang="ts">
import { NButton, NLayout, NLayoutContent, NSelect, NSpace, NText } from "naive-ui";
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { useRouter } from "vue-router";

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
    <NLayoutContent content-style="padding: 2rem;">
      <main class="welcome-page">
        <div class="welcome-content">
          <header class="welcome-header">
            <h1>{{ t("welcome.heading") }}</h1>
            <NText depth="3">{{ t("welcome.body") }}</NText>
          </header>

          <section class="tools" :aria-label="t('welcome.tools')">
            <NButton
              block
              class="tool-action"
              type="primary"
              size="large"
              @click="router.push('/merge')"
            >
              <span class="tool-action__content">
                <span class="tool-action__title">{{ t("welcome.merge") }}</span>
                <span class="tool-action__description">{{ t("welcome.mergeDescription") }}</span>
              </span>
            </NButton>
            <NButton block class="tool-action" size="large" @click="router.push('/split')">
              <span class="tool-action__content">
                <span class="tool-action__title">{{ t("welcome.split") }}</span>
                <span class="tool-action__description">{{ t("welcome.splitDescription") }}</span>
              </span>
            </NButton>
            <NButton block class="tool-action" size="large" @click="router.push('/redact')">
              <span class="tool-action__content">
                <span class="tool-action__title">{{ t("welcome.redact") }}</span>
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
  min-height: 100vh;
}

.welcome-page {
  display: grid;
  min-height: calc(100vh - 4rem);
  place-items: center;
}

.welcome-content {
  width: min(100%, 48rem);
}

.welcome-header h1 {
  margin: 0;
  font-size: clamp(2rem, 7vw, 3rem);
  letter-spacing: -0.04em;
}

.tools {
  display: grid;
  gap: 0.75rem;
  margin-top: 2.5rem;
}

.tool-action {
  height: auto;
  min-height: 5.75rem;
  padding: 0.25rem 0.5rem;
  text-align: left;
}

.tool-action__content {
  display: grid;
  gap: 0.3rem;
  width: 100%;
  padding: 0.65rem 0.75rem;
}

.tool-action__title {
  font-size: 1.0625rem;
  font-weight: 650;
}

.tool-action__description {
  font-size: 0.875rem;
  font-weight: 400;
  line-height: 1.35;
  opacity: 0.78;
}

.preferences {
  margin-top: 1.5rem;
}

.preference-control {
  display: grid;
  grid-template-columns: auto 7.75rem;
  align-items: center;
  gap: 0.5rem;
}

@media (max-width: 34rem) {
  .preferences {
    justify-content: start;
  }
}
</style>
