import { register, init, getLocaleFromNavigator, waitLocale } from 'svelte-i18n';
import { resolveResource } from '@tauri-apps/api/path';
import { readTextFile } from '@tauri-apps/plugin-fs';
import { load } from '@tauri-apps/plugin-store';

const locales = [
  'af-ZA',
  'ar-SA',
  'ca-ES',
  'cs-CZ',
  'da-DK',
  'de-DE',
  'el-GR',
  'en-US',
  'es-ES',
  'fi-FI',
  'fr-FR',
  'he-IL',
  'hu-HU',
  'it-IT',
  'ja-JP',
  'ko-KR',
  'nl-NL',
  'no-NO',
  'pl-PL',
  'pt-BR',
  'pt-PT',
  'ro-RO',
  'ru-RU',
  'sr-SP',
  'sv-SE',
  'tr-TR',
  'uk-UA',
  'vi-VN',
  'vls-BE',
  'zh-CN',
  'zh-TW',
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
    fallbackLocale: 'en-US',
    initialLocale: (await config.get<string>('language')) || 'en-US',
  });

  await waitLocale();
}