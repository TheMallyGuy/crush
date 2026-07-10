import { load } from '@tauri-apps/plugin-store'
import { serverService } from './discordAuth.svelte'
import { validateServers } from '../serverSelector'

interface ServerManagementData {
    autoSubmit: boolean
    preferredRegion: string
    autoValidate: boolean
}

const DEFAULTS: ServerManagementData = {
    autoSubmit: true,
    preferredRegion: '',
    autoValidate: false,
}

const VALIDATE_INTERVAL = 15 * 60_000

class ServerManagementSettings {
    autoSubmit = $state(true)
    preferredRegion = $state('')
    autoValidate = $state(false)
    loaded = $state(false)

    #validateTimer: ReturnType<typeof setInterval> | null = null

    async init() {
        const store = await load('config.json')
        const saved = await store.get<ServerManagementData>('serverManagement')
        const data = { ...DEFAULTS, ...saved }

        this.autoSubmit = data.autoSubmit
        this.preferredRegion = data.preferredRegion
        this.autoValidate = data.autoValidate
        this.loaded = true
    }

    async save() {
        const store = await load('config.json')
        await store.set('serverManagement', {
            autoSubmit: this.autoSubmit,
            preferredRegion: this.preferredRegion,
            autoValidate: this.autoValidate,
        } satisfies ServerManagementData)
        await store.save()
    }


    syncAutoValidateTimer() {
        if (this.autoValidate && serverService.isLoggedIn) {
            this.#startAutoValidate()
        } else {
            this.#stopAutoValidate()
        }
    }

    #startAutoValidate() {
        if (this.#validateTimer) return

        void this.#runValidatePass()
        this.#validateTimer = setInterval(() => {
            void this.#runValidatePass()
        }, VALIDATE_INTERVAL)
    }

    #stopAutoValidate() {
        if (this.#validateTimer) {
            clearInterval(this.#validateTimer)
            this.#validateTimer = null
        }
    }

    async #runValidatePass() {
        if (!this.autoValidate || !serverService.isLoggedIn) {
            this.#stopAutoValidate()
            return
        }

        try {
            await validateServers()
        } catch (e) {
            console.error('[serverManagement] auto validate pass failed', e)
        }
    }
}

export const serverManagementSettings = new ServerManagementSettings()
