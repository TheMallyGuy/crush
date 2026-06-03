import { load } from '@tauri-apps/plugin-store'

class AppSettings {
    lockedInMode = $state(false)
    discordRpcEnabled = $state(true)
    currentLocale = $state('en')
    loaded = $state(false)

    async init() {
        const store = await load('config.json')
        this.lockedInMode = (await store.get<boolean>('lockedIn')) ?? false
        this.discordRpcEnabled = (await store.get<boolean>('discordRpcEnabled')) ?? true
        this.currentLocale = (await store.get<string>('language')) ?? 'en'
        this.loaded = true
    }

    async setLockedIn(val: boolean) {
        this.lockedInMode = val
        const store = await load('config.json')
        await store.set('lockedIn', val)
        await store.save()
    }
}

export const settings = new AppSettings()