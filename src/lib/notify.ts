import { writable } from 'svelte/store'

export type NotifyVariant = 'info' | 'success' | 'warning' | 'danger' | 'default'

export interface Notification {
    id: string
    variant: NotifyVariant
    title: string
    description?: string
    duration?: number
}

function createNotifyStore() {
    const { subscribe, update } = writable<Notification[]>([])

    function send(notif: Omit<Notification, 'id'>) {
        const id = crypto.randomUUID()
        const duration = notif.duration ?? 4000

        update(n => [...n, { ...notif, id }])

        if (duration > 0) {
            setTimeout(() => dismiss(id), duration)
        }
    }

    function dismiss(id: string) {
        update(n => n.filter(notif => notif.id !== id))
    }

    return { subscribe, send, dismiss }
}

export const notify = createNotifyStore()