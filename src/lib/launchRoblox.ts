import { appDataDir, join } from '@tauri-apps/api/path'
import { invoke } from '@tauri-apps/api/core'
import { load, Store } from '@tauri-apps/plugin-store'
import { type Mod } from '$lib/types'
import { restoreFileFromPackage, getPackageForFile } from '$lib/downloadRoblox'

async function revertDisabledMods(
    mods: Mod[],
    modStore: Store,
    robloxHash: string,
    versionDir: string
) {
    const modsToClear: string[] = []
    const packageFilesMap = new Map<string, string[]>()

    const disabledMods = mods.filter((m) => !m.enabled)

    for (const mod of disabledMods) {
        const files = (await modStore.get<string[]>(mod.name)) ?? []
        if (files.length === 0) continue

        for (const file of files) {
            const pkg = getPackageForFile(file)
            if (!pkg) continue

            if (!packageFilesMap.has(pkg)) {
                packageFilesMap.set(pkg, [])
            }

            const normalized = file.replace(/\\/g, '/')
            packageFilesMap.get(pkg)!.push(normalized)
        }
        modsToClear.push(mod.name)
    }

    if (packageFilesMap.size === 0) return

    await Promise.all(
        Array.from(packageFilesMap.entries()).map(([pkg, files]) =>
            restoreFileFromPackage(pkg, robloxHash, versionDir, true, files)
        )
    )

    for (const modName of modsToClear) {
        await modStore.delete(modName)
    }
}

async function applyEnabledMods(
    mods: Mod[],
    modStore: Store,
    versionDir: string,
    appData: string
) {
    const enabledMods = mods.filter((m) => m.enabled)
    if (enabledMods.length === 0) return

    await Promise.all(
        enabledMods.map(async (mod) => {
            const modDir = await join(appData, 'Mods', mod.name)
            const copiedFiles: string[] = await invoke('apply_mod', {
                modDir,
                versionDir,
            })
            await modStore.set(mod.name, copiedFiles)
        })
    )
}

export async function applyMods(robloxHash: string) {
    const store = await load('mods.json')
    const modStore = await load('mod_manifests.json')
    const mods = (await store.get<Mod[]>('mods')) ?? []
    const appData = await appDataDir()
    const versionDir = await join(appData, 'Player', 'Versions', robloxHash)

    await revertDisabledMods(mods, modStore, robloxHash, versionDir)
    await applyEnabledMods(mods, modStore, versionDir, appData)

    await modStore.save()
}

export async function removeMod(mod: Mod, robloxHash: string) {
    const modStore = await load('mod_manifests.json')
    const appData = await appDataDir()
    const versionDir = await join(appData, 'Player', 'Versions', robloxHash)

    const files = (await modStore.get<string[]>(mod.name)) ?? []

    for (const relativePath of files) {
        await restoreFileFromPackage(relativePath, robloxHash, versionDir)
    }

    await modStore.delete(mod.name)
    await modStore.save()
}

export async function launchPlayer(hash: string, deeplink: string | null) {
    const playerLocation = await join(
        await appDataDir(),
        'Player',
        'Versions',
        hash,
        'RobloxPlayerBeta.exe'
    )
    const args = deeplink ? ['--play', '--deeplink', deeplink] : ['--play']

    await invoke('launch', { path: playerLocation, arguments: args })
}
