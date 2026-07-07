import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import type { ServerInfoFromBackend } from '$lib/types'

class ServerInfoStore {
    serverId = $state<string | null>(null)
    gameId = $state<number | null>(null)
    regionInfo = $state<string | null>(null)
    isPrivateServer = $state(false)
    accessCode = $state<string | null>(null)
    loaded = $state(false)

    private apply(info: ServerInfoFromBackend) {
        this.serverId = info.server_id
        this.gameId = info.game_id
        this.regionInfo = info.region_info
        this.isPrivateServer = info.is_private_server
        this.accessCode = info.access_code
        this.loaded = true
    }

    async init() {
        const stored = await invoke<ServerInfoFromBackend | null>(
            'get_server_info'
        ).catch(() => null)

        if (stored) this.apply(stored)

        await listen<ServerInfoFromBackend>('serverInfomation', (event) => {
            this.apply(event.payload)
        })
    }
}

export const serverInfo = new ServerInfoStore()
