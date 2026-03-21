import { appDataDir, join } from '@tauri-apps/api/path'
import { invoke } from '@tauri-apps/api/core'

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
