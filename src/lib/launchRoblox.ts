import { appDataDir, join } from '@tauri-apps/api/path'
import { invoke } from '@tauri-apps/api/core'
import { load } from '@tauri-apps/plugin-store'
import { type Mod } from './mods/modManagement'
import { restoreFileFromPackage, getPackageForFile } from '$lib/downloadRoblox'

export async function applyMods(roblox_hash: string) {
    const store    = await load("mods.json")
    const modStore = await load('mod_manifests.json')
    const mods     = await store.get<Mod[]>('mods') ?? []
    const appData  = await appDataDir()
    const versionDir = await join(appData, 'Player', 'Versions', roblox_hash)

    const packagesToRestore = new Set<string>()
    const modsToClear: string[] = []

    for (const mod of mods) {
        if (!mod.enabled) {
            const files = await modStore.get<string[]>(mod.name) ?? []
            if (files.length > 0) {
                for (const file of files) {
                    const pkg = getPackageForFile(file)
                    if (pkg) packagesToRestore.add(pkg)
                }
                modsToClear.push(mod.name)
            }
        }
    }

    if (packagesToRestore.size > 0) {
        await Promise.all(
            Array.from(packagesToRestore).map(pkg => 
                restoreFileFromPackage(pkg, roblox_hash, versionDir, true)
            )
        )
        for (const modName of modsToClear) {
            await modStore.delete(modName)
        }
    }

    const enabledMods = mods.filter(m => m.enabled)
    if (enabledMods.length > 0) {
        await Promise.all(enabledMods.map(async mod => {
            const modDir = await join(appData, 'Mods', mod.name)
            const copiedFiles: string[] = await invoke('apply_mod', { modDir, versionDir })
            await modStore.set(mod.name, copiedFiles)
        }))
    }

    await modStore.save()
}

export async function removeMod(mod: Mod, roblox_hash: string) {
    const modStore   = await load('mod_manifests.json')
    const appData    = await appDataDir()
    const versionDir = await join(appData, 'Player', 'Versions', roblox_hash)

    const files = await modStore.get<string[]>(mod.name) ?? []

    for (const relativePath of files) {
        await restoreFileFromPackage(relativePath, roblox_hash, versionDir)
    }

    await modStore.delete(mod.name)
    await modStore.save()
}

export async function launchPlayer(hash: string) {
    const playerLocation = await join(
        await appDataDir(),
        'Player',
        'Versions',
        hash,
        'RobloxPlayerBeta.exe'
    )
    await invoke('launch', { path: playerLocation })
}
