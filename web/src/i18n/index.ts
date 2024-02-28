import { createI18n } from "vue-i18n";
import en from "./lang/en";
const messages = { en };

export const i18n = createI18n({
  legacy: false,
  locale: "en", // set locale
  fallbackLocale: "en", // set fallback locale
  messages,
});
