<script setup lang="ts">
import { NButton, NCard, NLayout, NLayoutContent, NSelect, NSpace, NText, NThing } from "naive-ui";
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
        <NCard class="welcome-card" embedded>
          <NThing>
            <template #header>{{ t("welcome.heading") }}</template>
            <NText depth="3">{{ t("welcome.body") }}</NText>
          </NThing>

          <NSpace class="tools" vertical size="large">
            <NButton block type="primary" size="large" @click="router.push('/merge')">
              {{ t("welcome.merge") }}
            </NButton>
            <NButton block size="large" @click="router.push('/split')">
              {{ t("welcome.split") }}
            </NButton>
            <NButton block size="large" @click="router.push('/redact')">
              {{ t("welcome.redact") }}
            </NButton>
          </NSpace>

          <NSpace class="preferences" vertical size="large">
            <label>
              <span>{{ t("common.theme") }}</span>
              <NSelect
                :value="appearanceMode"
                :options="themeOptions"
                @update:value="selectAppearance($event as AppearanceMode)"
              />
            </label>
            <label>
              <span>{{ t("welcome.language") }}</span>
              <NSelect
                :value="locale"
                :options="languageOptions"
                @update:value="selectLanguage($event as SupportedLocale)"
              />
            </label>
          </NSpace>
        </NCard>
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

.welcome-card {
  width: min(100%, 38rem);
}

.preferences {
  margin-top: 2rem;
}

.tools {
  margin-top: 2rem;
}

label {
  display: grid;
  gap: 0.5rem;
}
</style>
