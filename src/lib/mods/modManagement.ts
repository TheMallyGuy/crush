import { invoke } from '@tauri-apps/api/core'
import { mkdir, remove, exists } from '@tauri-apps/plugin-fs'
import { join, appDataDir } from '@tauri-apps/api/path'
import { load } from '@tauri-apps/plugin-store'
import { openPath } from '@tauri-apps/plugin-opener'
import { applyMods } from '$lib/launchRoblox'
import type { Mod } from '$lib/types'

let storePromise = load('mods.json')

async function getStore() {
    return await storePromise
}

export async function loadMods(): Promise<Mod[]> {
    const store = await getStore()
    return (await store.get<Mod[]>('mods')) ?? []
}

export async function createNewMod(name: string) {
    let modsList = await loadMods()
    const store = await getStore()

    const id = crypto.randomUUID()

    const modLocation = await join(await appDataDir(), 'Mods', name)
    await mkdir(modLocation, { recursive: true })

    const newMod: Mod = { id, name, enabled: true }

    await store.set('mods', [...modsList, newMod])
    await store.save()
}

export async function renameMod(id: string, new_name: string) {
    const store = await getStore()
    const base = await appDataDir()
    let modsList = await loadMods()

    const mod = modsList.find((m) => m.id === id)
    if (!mod) throw new Error('Mod not found')

    const oldPath = await join(base, 'Mods', mod.name)
    const newPath = await join(base, 'Mods', new_name)

    await invoke('rename', { name: oldPath, new_name: newPath })

    const updated = modsList.map((m) =>
        m.id === id ? { ...m, name: new_name } : m
    )

    await store.set('mods', updated)
    await store.save()
}

export async function deleteMod(id: string) {
    const store = await getStore()
    const base = await appDataDir()
    let modsList = await loadMods()

    const mod = modsList.find((m) => m.id === id)
    if (!mod) return

    const modPath = await join(base, 'Mods', mod.name)
    try {
        await remove(modPath, { recursive: true })
    } catch (e) {
        console.error('Failed to remove mod directory', e)
    }

    const updated = modsList.filter((m) => m.id !== id)
    await store.set('mods', updated)
    await store.save()
}

export async function toggleMod(id: string) {
    const store = await getStore()
    let modsList = await loadMods()
    const updated = modsList.map((m) =>
        m.id === id ? { ...m, enabled: !m.enabled } : m
    )
    await store.set('mods', updated)
    await store.save()

    load('versions.json').then(async (versionStore) => {
        const versionList = (await versionStore.get<string[]>('versions')) ?? []
        const latestVersion = versionList.at(-1)
        if (latestVersion) {
            applyMods(latestVersion).catch(console.error)
        }
    })
}
export async function saveModsOrder(mods: Mod[]) {
    const store = await getStore()
    await store.set('mods', mods)
    await store.save()
}

export async function openModFolder(name: string) {
    const modLocation = await join(await appDataDir(), 'Mods', name)
    await openPath(modLocation)
}

export async function getModPath(name:string): Promise<string> {
    return await join(await appDataDir(), 'Mods', name)
}

export async function modExists(name: string): Promise<boolean> {
    const modLocation = await join(await appDataDir(), 'Mods', name)    
    return exists(modLocation)
}