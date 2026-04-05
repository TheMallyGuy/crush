import { register, init, getLocaleFromNavigator, waitLocale } from 'svelte-i18n';
import { resolveResource } from '@tauri-apps/api/path';
import { readTextFile } from '@tauri-apps/plugin-fs';
import { load } from '@tauri-apps/plugin-store';

const locales = [
  'af', 'ar', 'ca', 'cs', 'da', 'de', 'el', 'en',
  'es-ES', 'fi', 'fr', 'he', 'hu', 'it', 'ja', 'ko',
  'nl', 'no', 'pl', 'pt-BR', 'vi'
];

async function loadLocale(locale: string) {
  const path = await resolveResource(`resources/locales/${locale}/default.json`);
  const content = await readTextFile(path);
  return JSON.parse(content);
}

export async function setupI18n() {
  const config = await load("config.json")

  for (const locale of locales) {
    register(locale, () => loadLocale(locale));
  }

  await init({
    fallbackLocale: 'en',
    initialLocale: await config.get('language') || 'en',
  });

  await waitLocale();
}