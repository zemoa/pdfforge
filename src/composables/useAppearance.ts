import { computed, ref } from "vue";

export type AppearanceMode = "dark" | "light" | "system";
type ResolvedTheme = Exclude<AppearanceMode, "system">;

const storageKey = "pdfforge.appearance-mode";
const systemPrefersDark = window.matchMedia("(prefers-color-scheme: dark)");
const savedMode = window.localStorage.getItem(storageKey);
const appearanceMode = ref<AppearanceMode>(
  savedMode === "dark" || savedMode === "light" || savedMode === "system" ? savedMode : "system",
);
const systemIsDark = ref(systemPrefersDark.matches);

systemPrefersDark.addEventListener("change", (event) => {
  systemIsDark.value = event.matches;
});

const resolvedTheme = computed<ResolvedTheme>(() => {
  if (appearanceMode.value === "system") {
    return systemIsDark.value ? "dark" : "light";
  }

  return appearanceMode.value;
});

export function useAppearance() {
  function selectAppearance(mode: AppearanceMode) {
    appearanceMode.value = mode;
    window.localStorage.setItem(storageKey, mode);
  }

  return { appearanceMode, resolvedTheme, selectAppearance };
}
