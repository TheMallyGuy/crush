import { appDataDir, join } from '@tauri-apps/api/path'
import { invoke } from '@tauri-apps/api/core'
import { load, Store } from '@tauri-apps/plugin-store'
import { type Mod } from '$lib/types'
import { restoreFileFromPackage, getPackageForFile } from '$lib/downloadRoblox'

async function revertUnusedMods(
    mods: Mod[],
    manifestStore: Store,
    robloxHash: string,
    versionDir: string
) {
    const activeModNames = new Set(
        mods.filter((m) => m.enabled).map((m) => m.name)
    )
    const storedModNames = await manifestStore.keys()
    const modsToRevert = storedModNames.filter((name) => !activeModNames.has(name))

    if (modsToRevert.length === 0) return

    const filesByPackage = new Map<string, string[]>()

    for (const modName of modsToRevert) {
        const files = (await manifestStore.get<string[]>(modName)) ?? []
        if (files.length === 0) continue

        for (const file of files) {
            const pkg = getPackageForFile(file)
            if (!pkg) continue

            if (!filesByPackage.has(pkg)) {
                filesByPackage.set(pkg, [])
            }

            const normalized = file.replace(/\\/g, '/')
            filesByPackage.get(pkg)!.push(normalized)
        }
    }

    if (filesByPackage.size === 0) {
        for (const modName of modsToRevert) {
            await manifestStore.delete(modName)
        }
        return
    }

    await Promise.all(
        Array.from(filesByPackage.entries()).map(([pkg, files]) =>
            restoreFileFromPackage(pkg, robloxHash, versionDir, true, files)
        )
    )

    for (const modName of modsToRevert) {
        await manifestStore.delete(modName)
    }
}

async function applyEnabledMods(
    mods: Mod[],
    manifestStore: Store,
    versionDir: string,
    appData: string
) {
    const enabledMods = mods.filter((m) => m.enabled)
    if (enabledMods.length === 0) return

    for (const mod of enabledMods) {
        const modDir = await join(appData, 'Mods', mod.name)
        const copiedFiles: string[] = await invoke('apply_mod', {
            modDir,
            versionDir,
        })
        await manifestStore.set(mod.name, copiedFiles)
    }
}

export async function applyMods(robloxHash: string) {
    const modsConfig = await load('mods.json')
    const manifestStore = await load('mod_manifests.json')
    const mods = (await modsConfig.get<Mod[]>('mods')) ?? []
    const appData = await appDataDir()
    const versionDir = await join(appData, 'Player', 'Versions', robloxHash)

    await revertUnusedMods(mods, manifestStore, robloxHash, versionDir)
    await applyEnabledMods(mods, manifestStore, versionDir, appData)

    await manifestStore.save()
}

export async function removeMod(mod: Mod, robloxHash: string) {
    const manifestStore = await load('mod_manifests.json')
    const appData = await appDataDir()
    const versionDir = await join(appData, 'Player', 'Versions', robloxHash)

    const files = (await manifestStore.get<string[]>(mod.name)) ?? []
    if (files.length === 0) {
        await manifestStore.delete(mod.name)
        await manifestStore.save()
        return
    }

    const filesByPackage = new Map<string, string[]>()
    for (const file of files) {
        const pkg = getPackageForFile(file)
        if (!pkg) continue

        if (!filesByPackage.has(pkg)) {
            filesByPackage.set(pkg, [])
        }
        filesByPackage.get(pkg)!.push(file.replace(/\\/g, '/'))
    }

    await Promise.all(
        Array.from(filesByPackage.entries()).map(([pkg, pkgFiles]) =>
            restoreFileFromPackage(pkg, robloxHash, versionDir, true, pkgFiles)
        )
    )

    await manifestStore.delete(mod.name)
    await manifestStore.save()
}

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
