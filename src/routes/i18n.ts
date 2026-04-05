import { register, init, getLocaleFromNavigator } from 'svelte-i18n';
import { resolveResource } from '@tauri-apps/api/path';
import { readTextFile } from '@tauri-apps/plugin-fs';

export async function setupI18n() {
  const enPath = await resolveResource('resources/locales/en.json');
  const en = JSON.parse(await readTextFile(enPath));

  const { addMessages } = await import('svelte-i18n');
  addMessages('en', en);

  init({
    fallbackLocale: 'en',
    initialLocale: 'en',
  });
}