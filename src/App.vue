<script setup lang="ts">
import { darkTheme, NConfigProvider, NMessageProvider, type GlobalTheme } from "naive-ui";
import { computed } from "vue";
import { RouterView } from "vue-router";

import WindowControls from "./components/WindowControls.vue";
import { useAppearance } from "./composables/useAppearance";

const { resolvedTheme } = useAppearance();
const naiveTheme = computed<GlobalTheme | null>(() =>
  resolvedTheme.value === "dark" ? darkTheme : null,
);
</script>

<template>
  <NConfigProvider :theme="naiveTheme">
    <WindowControls />
    <NMessageProvider><RouterView /></NMessageProvider>
  </NConfigProvider>
</template>
