import { createSignal } from "solid-js";
import type { Locale } from "../lib/types";
import { sv, type TranslationKey } from "./sv";
import { en } from "./en";
import { pt } from "./pt";

const dictionaries: Record<Locale, Record<TranslationKey, string>> = {
  "sv-SE": sv,
  "en-GB": en,
  "pt-BR": pt,
};

const [locale, setLocale] = createSignal<Locale>("sv-SE");

export { locale, setLocale };

export function t(key: TranslationKey): string {
  return dictionaries[locale()][key] ?? key;
}

export const localeNames: Record<Locale, string> = {
  "sv-SE": "Svenska",
  "en-GB": "English",
  "pt-BR": "Português",
};
