<script setup lang="ts">
import {
  darkTheme,
  NConfigProvider,
  NMessageProvider,
  type GlobalTheme,
  type GlobalThemeOverrides,
} from "naive-ui";
import { computed } from "vue";
import { RouterView } from "vue-router";

import WindowControls from "./components/WindowControls.vue";
import UpdateNotification from "./components/UpdateNotification.vue";
import { useAppearance } from "./composables/useAppearance";

const { resolvedTheme } = useAppearance();
const naiveTheme = computed<GlobalTheme | null>(() =>
  resolvedTheme.value === "dark" ? darkTheme : null,
);
const themeOverrides = computed<GlobalThemeOverrides>(() => {
  const dark = resolvedTheme.value === "dark";
  return {
    common: {
      baseColor: dark ? "#111111" : "#f7f7f7",
      bodyColor: dark ? "#111111" : "#f7f7f7",
      cardColor: dark ? "#181818" : "#ffffff",
      dividerColor: dark ? "#303030" : "#dedede",
      inputColor: dark ? "#181818" : "#ffffff",
      modalColor: dark ? "#181818" : "#ffffff",
      primaryColor: dark ? "#8177ff" : "#6055e8",
      primaryColorHover: dark ? "#958dff" : "#7469ed",
      primaryColorPressed: dark ? "#6f65e8" : "#5045d2",
      successColor: dark ? "#5ba77d" : "#3e8d67",
      successColorHover: dark ? "#5ba77d" : "#3e8d67",
      successColorPressed: dark ? "#5ba77d" : "#3e8d67",
      successColorSuppl: dark ? "#5ba77d" : "#3e8d67",
      errorColor: dark ? "#dc6270" : "#c83d4b",
      errorColorHover: dark ? "#dc6270" : "#c83d4b",
      errorColorPressed: dark ? "#dc6270" : "#c83d4b",
      errorColorSuppl: dark ? "#dc6270" : "#c83d4b",
      textColorBase: dark ? "#f4f4f4" : "#151515",
      textColor2: dark ? "#a0a0a0" : "#777777",
      textColor3: dark ? "#737373" : "#999999",
    },
    Button: {
      borderRadiusMedium: "7px",
      borderRadiusSmall: "5px",
      colorQuaternaryHover: dark ? "#282828" : "#ececec",
      colorQuaternaryPressed: dark ? "#333333" : "#e1e1e1",
      textColor: dark ? "#f4f4f4" : "#151515",
      textColorFocus: dark ? "#f4f4f4" : "#151515",
      textColorHover: dark ? "#f4f4f4" : "#151515",
      textColorPressed: dark ? "#f4f4f4" : "#151515",
      textColorText: dark ? "#f4f4f4" : "#151515",
      textColorTextFocus: dark ? "#f4f4f4" : "#151515",
      textColorTextHover: dark ? "#f4f4f4" : "#151515",
      textColorTextPressed: dark ? "#f4f4f4" : "#151515",
    },
  };
});
</script>

<template>
  <NConfigProvider :theme="naiveTheme" :theme-overrides="themeOverrides">
    <WindowControls />
    <NMessageProvider><UpdateNotification /><RouterView /></NMessageProvider>
  </NConfigProvider>
</template>
