// import { appDataDir, join } from '@tauri-apps/api/path'
// import {
//     mkdir,
//     exists,
//     writeTextFile,
//     readTextFile,
// } from '@tauri-apps/plugin-fs'
// import type { AppType } from '$lib/types'

import type { AppType } from "$lib/downloadRoblox";
import { operating_system } from "$lib/stores/operating_system.svelte";
import { appDataDir, homeDir, join } from "@tauri-apps/api/path";
import { BaseDirectory, exists, mkdir, readTextFile, remove, writeTextFile } from "@tauri-apps/plugin-fs";
import { info } from "@tauri-apps/plugin-log";
import { load, Store } from "@tauri-apps/plugin-store";
import { store } from "@tauri-store/svelte";
import { get } from "svelte/store";

// async function ensureDir(path: string) {
//     if (!(await exists(path))) {
//         await mkdir(path, { recursive: true })
//     }
// }


function coerceValue(value: string): boolean | number | string {
    if (value === 'true') return true
    if (value === 'false') return false
    if (/^-?\d+(\.\d+)?$/.test(value)) return Number(value)
    return value
}

function decoerceValue(value: unknown): string {
    if (value === null || value === undefined) return ''
    return String(value)
}

function isMacOs(): boolean {
    return get(operating_system) === 'macos'
}

function getMacAppBundleName(app_type: AppType): string {
    return app_type === 'studio' ? 'RobloxStudio.app' : 'Roblox.app'
}

// export async function saveFastFlags(
//     version_hash: string,
//     flags: Record<string, string>,
//     appType: AppType = 'player'
// ): Promise<void> {
//     console.log(`[FastFlag] Attempting save for ${appType}, version: ${version_hash}`)
//     if (!version_hash) {
//         console.error('[FastFlag] Aborting save: version_hash is empty')
//         throw new Error('No version hash provided')
//     }

//     try {
//         const baseDir = await appDataDir()
//         const appFolder = appType === 'studio' ? 'Studio' : 'Player'
//         const clientSettingFolder = await join(
//             baseDir,
//             appFolder,
//             'Versions',
//             version_hash,
//             'ClientSettings'
//         )
//         const clientSettingFile = await join(
//             clientSettingFolder,
//             'ClientAppSettings.json'
//         )

//         await ensureDir(clientSettingFolder)

//         const coerced = Object.fromEntries(
//             Object.entries(flags)
//                 .filter(([_, v]) => v !== null && v !== undefined)
//                 .map(([k, v]) => [k, coerceValue(v)])
//         )

//         const content = JSON.stringify(coerced, null, 2)
//         console.log(`[FastFlag] Writing ${content.length} bytes to ${clientSettingFile}`)

//         await writeTextFile(clientSettingFile, content)

//         // Final check
//         const verifyExists = await exists(clientSettingFile)
//         if (!verifyExists) throw new Error('File was not created after write operation')

//         console.log('[FastFlag] Save successful and verified')
//     } catch (err) {
//         console.error('[FastFlag] CRITICAL ERROR during save:', err)
//         throw err
//     }
// }


// export async function getFastFlags(
//     version_hash: string,
//     appType: AppType = 'player'
// ): Promise<Record<string, string>> {
//     console.log(`[FastFlag] Loading flags for ${appType}, version: ${version_hash}`)
//     if (!version_hash) return {}
//     const baseDir = await appDataDir()
//     const appFolder = appType === 'studio' ? 'Studio' : 'Player'
//     const clientSettingFolder = await join(
//         baseDir,
//         appFolder,
//         'Versions',
//         version_hash,
//         'ClientSettings'
//     )
//     const clientSettingFile = await join(
//         clientSettingFolder,
//         'ClientAppSettings.json'
//     )

//     try {
//         if (!(await exists(clientSettingFile))) {
//             console.log(`[FastFlag] No file found at ${clientSettingFile}`)
//             return {}
//         }

//         const raw = await readTextFile(clientSettingFile)
//         if (!raw.trim()) return {}
//         const parsed: Record<string, unknown> = JSON.parse(raw)

//         console.log(`[FastFlag] Loaded ${Object.keys(parsed).length} flags from disk`)

//         return Object.fromEntries(
//             Object.entries(parsed).map(([k, v]) => [k, decoerceValue(v)])
//         )
//     } catch (e) {
//         console.error(`[FastFlag] Failed to load flags:`, e)
//         return {}
//     }
// }


async function resolveClientSettingsPaths(
    app_type: AppType,
    version: string,
    vng?: boolean
): Promise<{ dir: string; file: string }> {
    if (isMacOs()) {
        const bundleName = getMacAppBundleName(app_type)
        const home = await homeDir()
        const dir = await join(home, '/Applications', bundleName, 'Contents', 'MacOS', 'ClientSettings')
        const file = await join(dir, 'ClientAppSettings.json')
        return { dir, file }
    }

    const folder = app_type === 'player' ? (vng ? 'PlayerVNG' : 'Player') : 'Studio'
    const versionDir = await join(await appDataDir(), folder, 'Versions', version)
    const dir = await join(versionDir, 'ClientSettings')
    const file = await join(dir, 'ClientAppSettings.json')
    return { dir, file }
}

export async function getFastFlags(app_type: AppType = 'player'): Promise<Record<string, string>> {
    const clientSettings = `${app_type}GlobalClientSettings.json`

    if (!(await exists(clientSettings, { baseDir: BaseDirectory.AppData }))) {
        return {}
    }

    const raw = await readTextFile(clientSettings, { baseDir: BaseDirectory.AppData })

    if (!raw.trim()) return {}
    const parsed: Record<string, unknown> = JSON.parse(raw)

    return Object.fromEntries(
        Object.entries(parsed).map(([k, v]) => [k, decoerceValue(v)])
    )
}

export async function saveFastFlags(
    flags: Record<string, string>,
    app_type: AppType = 'player',
) {
    const clientSettings = `${app_type}GlobalClientSettings.json`

    const coerced = Object.fromEntries(
        Object.entries(flags)
            .filter(([_, v]) => v !== null && v !== undefined)
            .map(([k, v]) => [k, coerceValue(v)])
    )

    const content = JSON.stringify(coerced, null, 2)
    await writeTextFile(clientSettings, content, { baseDir: BaseDirectory.AppData })
}

export async function loadFlag(app_type: AppType, version: string, disable: boolean, vng?: boolean) {
    const { dir: clientSettingDir, file: clientSettingPath } = await resolveClientSettingsPaths(
        app_type,
        version,
        vng
    )

    const clientSettings = `${app_type}GlobalClientSettings.json`
    if (!(await exists(clientSettings, { baseDir: BaseDirectory.AppData }))) return

    const raw = await readTextFile(clientSettings, { baseDir: BaseDirectory.AppData })

    if (!(await exists(clientSettingDir))) {
        await mkdir(clientSettingDir, { recursive: true })
    }

    if (!disable) {
        await writeTextFile(clientSettingPath, raw)
    } else if (await exists(clientSettingPath)) {
        await remove(clientSettingPath)
    }
}