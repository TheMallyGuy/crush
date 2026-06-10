import { writable } from 'svelte/store'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'

export interface IslandNotification {
    id: string
    title: string
    description?: string
    image?: string
    duration?: number
}

export type IslandPayload = Omit<IslandNotification, 'id'>

export function showIsland(payload: IslandPayload): Promise<void> {
    return invoke('island_show', { payload })
}


function createIslandStore() {
    const { subscribe, update } = writable<IslandNotification[]>([])

    function push(payload: IslandPayload) {
        const id = crypto.randomUUID()
        const duration = payload.duration ?? 5000

        update(n => [...n, { ...payload, id }])

        if (duration > 0) {
            setTimeout(() => dismiss(id), duration)
        }
    }

    function dismiss(id: string) {
        update(n => n.filter(notif => notif.id !== id))
    }

    return { subscribe, push, dismiss }
}

export const island = createIslandStore()

export async function initIslandListener(): Promise<UnlistenFn> {
    return listen<IslandPayload>('dynamic-island', event => {
        island.push(event.payload)
    })
}
