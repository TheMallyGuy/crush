import { appDataDir, join } from '@tauri-apps/api/path'
import { invoke } from '@tauri-apps/api/core'
import { load } from '@tauri-apps/plugin-store'
import { type Mod } from './mods/modManagement'

export async function applyMods(roblox_hash: string) {
    const store = await load("mods.json")
    const mods = await store.get<Mod[]>('mods') ?? []
    const appData = await appDataDir()

    for (const mod of mods) {
        if (mod.enabled) {
            const modLocation = await join(appData, "Mods", mod.name)
            const robloxLocation = await join(appData, "Player", "Versions", roblox_hash)
            await invoke("apply_mod", { modDir: modLocation, versionDir: robloxLocation })
        }
    }
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
