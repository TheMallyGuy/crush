import { resolveResource, join, BaseDirectory, appDataDir } from '@tauri-apps/api/path';
import { mkdir, copyFile, readDir, readTextFile, writeTextFile } from '@tauri-apps/plugin-fs';
import { error, info } from '@tauri-apps/plugin-log';
import { createNewMod, getModPath, modExists, deleteMod, loadMods } from '$lib/mods/modManagement'
import { invoke } from '@tauri-apps/api/core';

async function processFontAssets(inputDir: string, outputDir: string, fontName: string) { // work like the bloxstrap one
    const entries = await readDir(inputDir);
    await mkdir(outputDir, { recursive: true });

    for (const entry of entries) {
        if (!entry.name?.endsWith(".json")) continue;

        const inputPath = await join(inputDir, entry.name);
        const outputPath = await join(outputDir, entry.name);

        await copyFile(inputPath, outputPath); // always copy first

        const raw = await readTextFile(outputPath);
        const data = JSON.parse(raw);

        if (!Array.isArray(data.faces)) continue;

        for (const face of data.faces) {
            if (face.assetId !== undefined) {
                face.assetId = `rbxasset://fonts/${fontName}`;
            }
        }

        await writeTextFile(outputPath, JSON.stringify(data, null, 2));
    }
}

export async function getModIdByName(name: string): Promise<string | undefined> {
  const mods = await loadMods();
  const mod = mods.find(m => m.name === name);
  return mod?.id;
}

export async function customFont(fontPath: string, rblxFamiliesDir: string) {
    const modName = `[BUILT-IN] Custom Font`

    if (await modExists(modName)) {
        try {
            const id = await getModIdByName(modName)
            await deleteMod(id!)
        } catch (err) {
            await error(`BUILT-IN MOD FAILED; something went wrong during deleting duplicate mod ${err}`)
        }
    }

    await createNewMod(modName)

    try {
        await mkdir(`Mods/${modName}/content/fonts/families`, { baseDir: BaseDirectory.AppData, recursive: true })

        const fileName = fontPath.split(/[\\/]/).at(-1)!
        const copyDest = await join(await appDataDir(), "Mods", modName, "content", "fonts", fileName)
        const famDest = await join(await appDataDir(), "Mods", modName, "content", "fonts", "families")

        await invoke("copy_file", { from: fontPath, to: copyDest })
        await processFontAssets(rblxFamiliesDir, famDest, fileName)
    } catch (err) {
        await error(`BUILT-IN MOD FAILED; something went wrong when trying to create font mod: ${err}`)
    }
}

export async function customCursor(arrowCursor: string, arrowFarCursor: string) {
    const modName = `[BUILT-IN] Custom Cursor`

    if (await modExists(modName)) {
        try {
            const id = await getModIdByName(modName)
            await deleteMod(id!)
        } catch (err) {
            await error(`BUILT-IN MOD FAILED; something went wrong during deleting duplicate mod ${err}`)
        }
    }

    await createNewMod(modName)

    try {
        await mkdir(`Mods/${modName}/content/textures/`, { baseDir: BaseDirectory.AppData, recursive: true })

        const fileNameArrow = arrowCursor.split(/[\\/]/).at(-1)!
        const fileNameArrowFar = arrowFarCursor.split(/[\\/]/).at(-1)!

        const copyDestArrow = await join(await appDataDir(), "Mods", modName, "content", "textures", "Cursors", "KeyboardMouse", fileNameArrow)
        const copyDestArrowFar = await join(await appDataDir(), "Mods", modName, "content", "textures", "Cursors", "KeyboardMouse", fileNameArrowFar)


        await invoke("copy_file", { from: arrowCursor, to: copyDestArrow })
        await invoke("copy_file", { from: arrowFarCursor, to: copyDestArrowFar })
        // that was easy
    } catch (err) { 
        await error(`BUILT-IN MOD FAILED; something went wrong when trying to copy cursors: ${err}`)
    }
}

export async function ensureCursor() {
    const modName = `[BUILT-IN] Custom Cursor`

    if (await modExists(modName)) {
        try {
            const id = await getModIdByName(modName)
            await deleteMod(id!)
        } catch (err) {
            await error(`BUILT-IN MOD FAILED; something went wrong during deleting duplicate mod ${err}`)
        }
    }
}