import { appDataDir, join } from '@tauri-apps/api/path'
import { invoke } from '@tauri-apps/api/core'
import { load, Store } from '@tauri-apps/plugin-store'
import type { AppType ,Mod } from '$lib/types'
import { restoreFileFromPackage, getPackageForFile } from '$lib/downloadRoblox'

function getAppFolder(appType: AppType): string {
    return appType === 'studio' ? 'Studio' : 'Player'
}

function getManifestStore(appType: AppType): string {
    return appType === 'studio' ? 'studio_mod_manifests.json' : 'mod_manifests.json'
}

function getModsConfig(appType: AppType): string {
    return appType === 'studio' ? 'studio_mods.json' : 'mods.json'
}

async function revertAppliedMods(
    manifestStore: Store,
    robloxHash: string,
    versionDir: string,
    appType: AppType
) {
    const storedModNames = await manifestStore.keys()
    if (storedModNames.length === 0) return

    const filesByPackage = new Map<string, string[]>()

    for (const modName of storedModNames) {
        const files = (await manifestStore.get<string[]>(modName)) ?? []
        if (files.length === 0) continue

        for (const file of files) {
            const pkg = getPackageForFile(file, appType)
            if (!pkg) continue

            if (!filesByPackage.has(pkg)) {
                filesByPackage.set(pkg, [])
            }

            const normalized = file.replace(/\\/g, '/')
            filesByPackage.get(pkg)!.push(normalized)
        }
    }

    if (filesByPackage.size > 0) {
        await Promise.all(
            Array.from(filesByPackage.entries()).map(([pkg, files]) =>
                restoreFileFromPackage(pkg, robloxHash, versionDir, true, files, appType)
            )
        )
    }

    for (const modName of storedModNames) {
        await manifestStore.delete(modName)
    }
}

async function applyEnabledMods(
    mods: Mod[],
    manifestStore: Store,
    versionDir: string,
    appData: string,
    appType: AppType
) {
    const enabledMods = mods.filter((m) => m.enabled)
    if (enabledMods.length === 0) return

    const modSubFolder = appType === 'studio' ? 'StudioMods' : 'Mods'

    for (const mod of enabledMods) {
        const modDir = await join(appData, modSubFolder, mod.name)
        const copiedFiles: string[] = await invoke('apply_mod', {
            modDir,
            versionDir,
        })
        await manifestStore.set(mod.name, copiedFiles)
    }
}

export async function applyMods(robloxHash: string, appType: AppType = 'player') {
    const modsConfig = await load(getModsConfig(appType))
    const manifestStore = await load(getManifestStore(appType))
    const mods = (await modsConfig.get<Mod[]>('mods')) ?? []
    const appData = await appDataDir()
    const versionDir = await join(appData, getAppFolder(appType), 'Versions', robloxHash)

    await revertAppliedMods(manifestStore, robloxHash, versionDir, appType)
    await applyEnabledMods(mods, manifestStore, versionDir, appData, appType)

    await manifestStore.save()
}

export async function removeMod(mod: Mod, robloxHash: string, appType: AppType = 'player') {
    const manifestStore = await load(getManifestStore(appType))
    const appData = await appDataDir()
    const versionDir = await join(appData, getAppFolder(appType), 'Versions', robloxHash)

    const files = (await manifestStore.get<string[]>(mod.name)) ?? []
    if (files.length === 0) {
        await manifestStore.delete(mod.name)
        await manifestStore.save()
        return
    }

    const filesByPackage = new Map<string, string[]>()
    for (const file of files) {
        const pkg = getPackageForFile(file, appType)
        if (!pkg) continue

        if (!filesByPackage.has(pkg)) {
            filesByPackage.set(pkg, [])
        }
        filesByPackage.get(pkg)!.push(file.replace(/\\/g, '/'))
    }

    await Promise.all(
        Array.from(filesByPackage.entries()).map(([pkg, pkgFiles]) =>
            restoreFileFromPackage(pkg, robloxHash, versionDir, true, pkgFiles, appType)
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

export async function launchStudio(hash: string, placeFile?: string | null) {
    const appData = await appDataDir()
    const studioLocation = await join(
        appData,
        'Studio',
        'Versions',
        hash,
        'RobloxStudioBeta.exe'
    )
    const args = placeFile ? [placeFile] : []

    await invoke('launch', { path: studioLocation, arguments: args })
}