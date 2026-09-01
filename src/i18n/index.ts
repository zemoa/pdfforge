import { createI18n } from "vue-i18n";

import en from "./locales/en";
import fr from "./locales/fr";

export type SupportedLocale = "en" | "fr";

const supportedLocales: readonly SupportedLocale[] = ["en", "fr"];

export function detectSystemLocale(): SupportedLocale {
  const browserLocales =
    navigator.languages.length > 0 ? navigator.languages : [navigator.language];
  const matchedLocale = browserLocales
    .map((locale) => locale.toLowerCase().split("-")[0])
    .find((locale): locale is SupportedLocale =>
      supportedLocales.includes(locale as SupportedLocale),
    );

  return matchedLocale ?? "en";
}

export const i18n = createI18n({
  legacy: false,
  locale: detectSystemLocale(),
  fallbackLocale: "en",
  messages: { en, fr },
});
