import { resolveResource, join, BaseDirectory, appDataDir } from '@tauri-apps/api/path';
import { mkdir, copyFile, readDir, readTextFile, writeTextFile } from '@tauri-apps/plugin-fs';
import { error } from '@tauri-apps/plugin-log';
import { createNewMod, getModPath, modExists, deleteMod, loadMods } from '$lib/mods/modManagement'
import { invoke } from '@tauri-apps/api/core';

async function processFontAssets(inputDir: string, outputDir: string) { // work like the bloxstrap one
  const entries = await readDir(inputDir);

  for (const entry of entries) {
    if (!entry.name?.endsWith(".json")) continue;

    const inputPath = await join(inputDir, entry.name);
    const raw = await readTextFile(inputPath);
    const data = JSON.parse(raw);

    if (!Array.isArray(data.Faces)) continue;

    let changed = false;
    for (const face of data.Faces) {
      if (face.AssetId !== undefined) {
        face.AssetId = "rbxasset://fonts/CustomFont.ttf";
        changed = true;
      }
    }

    if (changed) {
      const outputPath = await join(outputDir, entry.name);
      await writeTextFile(outputPath, JSON.stringify(data, null, 2));
    }
  }
}

async function getModIdByName(name: string): Promise<string | undefined> {
  const mods = await loadMods();
  const mod = mods.find(m => m.name === name);
  return mod?.id;
}

export async function customFont(Fontpath: string, rblxFamiliesDir: string) {
    const modName = `[BUILT-IN] Custom Font`

    if (await modExists(modName)) {
        try {
            const id = await getModIdByName(modName);
            deleteMod(id!)
        } catch (err) {
            await error(`BUILT-IN MOD FAILED; something went wrong during deleting duplicate mod ${error}`)
        }
    }

    createNewMod(modName);
    const modPath = getModPath(modName)

    try {
        mkdir(`Mods/${modName}`, {baseDir: BaseDirectory.AppData})

        const copyDest = await join(await appDataDir(), "Mods", modName, "content", "fonts")
        const famDest = await join(await appDataDir(), "Mods", modName, "content", "fonts", "families")

        invoke("copy_file", {from : Fontpath, to: copyDest})
        processFontAssets(rblxFamiliesDir, famDest)

    } catch (err) {
        error(`BUILT-IN MOD FAILED; something went wrong when trying to create font mod`)
    }
}