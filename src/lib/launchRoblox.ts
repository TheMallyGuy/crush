import { appDataDir, join } from '@tauri-apps/api/path'
import { invoke } from '@tauri-apps/api/core'
import { load } from '@tauri-apps/plugin-store'
import { type Mod } from './mods/modManagement'
import { restoreFileFromPackage, getPackageForFile } from '$lib/downloadRoblox'

/**
 * Applies or removes mods based on their enabled state.
 * Refactored to use declarative logic for clearer state management.
 */
export async function applyMods(roblox_hash: string) {
    const store = await load('mods.json')
    const modStore = await load('mod_manifests.json')
    const mods = (await store.get<Mod[]>('mods')) ?? []
    const appData = await appDataDir()
    const versionDir = await join(appData, 'Player', 'Versions', roblox_hash)

    // Identify disabled mods that need to be cleared and their files restored.
    const disabledModsToProcess = await Promise.all(
        mods
            .filter((mod) => !mod.enabled)
            .map(async (mod) => ({
                name: mod.name,
                files: (await modStore.get<string[]>(mod.name)) ?? [],
            }))
    )

    const modsToClear = disabledModsToProcess
        .filter((m) => m.files.length > 0)
        .map((m) => m.name)

    const packagesToRestore = new Set(
        disabledModsToProcess
            .flatMap((m) => m.files)
            .map(getPackageForFile)
            .filter((pkg): pkg is string => !!pkg)
    )

    // Restore files from original packages if any mods were disabled.
    if (packagesToRestore.size > 0) {
        await Promise.all(
            Array.from(packagesToRestore).map((pkg) =>
                restoreFileFromPackage(pkg, roblox_hash, versionDir, true)
            )
        )
        await Promise.all(modsToClear.map((name) => modStore.delete(name)))
    }

    // Apply enabled mods.
    const enabledMods = mods.filter((m) => m.enabled)
    if (enabledMods.length > 0) {
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

    await modStore.save()
}

/**
 * Removes a specific mod and restores any files it may have overridden.
 */
export async function removeMod(mod: Mod, roblox_hash: string) {
    const modStore = await load('mod_manifests.json')
    const appData = await appDataDir()
    const versionDir = await join(appData, 'Player', 'Versions', roblox_hash)

    const files = (await modStore.get<string[]>(mod.name)) ?? []

    await Promise.all(
        files.map((relativePath) =>
            restoreFileFromPackage(relativePath, roblox_hash, versionDir)
        )
    )

    await modStore.delete(mod.name)
    await modStore.save()
}

/**
 * Launches the Roblox player with optional deeplink parameters.
 */
export async function launchPlayer(hash: string, deeplink: string | null) {
    const appData = await appDataDir()
    const playerLocation = await join(
        appData,
        'Player',
        'Versions',
        hash,
        'RobloxPlayerBeta.exe'
    )
    const args = deeplink ? ['--play', '--deeplink', deeplink] : ['--play']

    await invoke('launch', { path: playerLocation, arguments: args })
}
