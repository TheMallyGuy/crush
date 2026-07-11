import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { getCurrentWindow, getAllWindows } from '@tauri-apps/api/window'
import type { ServerInfoFromBackend } from '$lib/types'
import { serverDataHandler } from '$lib/serverSelector'

const MAIN_WINDOW_LABEL = 'CrushMainWindow'

async function onPlayerJoined(info: ServerInfoFromBackend) {
    const label = getCurrentWindow().label

    if (label !== MAIN_WINDOW_LABEL) {
        const windows = await getAllWindows()
        const mainWindowOpen = windows.some((w) => w.label === MAIN_WINDOW_LABEL)
        if (mainWindowOpen) {
            return
        }
    }
    serverDataHandler(info)
}

class ServerInfoStore {
    serverId = $state<string | null>(null)
    gameId = $state<number | null>(null)
    regionInfo = $state<string | null>(null)
    isPrivateServer = $state(false)
    accessCode = $state<string | null>(null)
    ip = $state<string | null>(null)
    loaded = $state(false)

    private initialized = false

    private apply(info: ServerInfoFromBackend) {
        this.serverId = info.server_id
        this.gameId = info.game_id
        this.regionInfo = info.region_info
        this.isPrivateServer = info.is_private_server
        this.accessCode = info.access_code
        this.ip = info.ip
        this.loaded = true
    }

    async init() {
        if (this.initialized) return
        this.initialized = true

        const stored = await invoke<ServerInfoFromBackend | null>(
            'get_server_info'
        ).catch(() => null)

        if (stored) this.apply(stored)

        await listen<ServerInfoFromBackend>('serverInfomation', (event) => {
            this.apply(event.payload)
        })

        await listen<ServerInfoFromBackend>('playerJoined', (event) => {
            this.apply(event.payload)
            onPlayerJoined(event.payload)
        })
    }
}

export const serverInfo = new ServerInfoStore()
