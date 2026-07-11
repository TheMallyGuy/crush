import { load } from '@tauri-apps/plugin-store'

interface SettingsData {
    lockedIn: boolean
    redRings: boolean
    discordRpcEnabled: boolean
    language: string
    robloxWarpped: boolean
    uwuEnabled: boolean
    hoverSfxEnabled: boolean
}

const DEFAULTS: SettingsData = {
    lockedIn: false,
    redRings: false,
    discordRpcEnabled: true,
    language: 'en',
    robloxWarpped: true,
    uwuEnabled: false,
    hoverSfxEnabled: false,
}

class AppSettings {
    lockedInMode = $state(false)
    redRings = $state(false)
    discordRpcEnabled = $state(true)
    currentLocale = $state('en')
    robloxWarpped = $state(true)
    uwuEnabled = $state(false)
    hoverSfxEnabled = $state(false)
    loaded = $state(false)

    async init() {
        const store = await load('config.json')
        let saved = await store.get<SettingsData>('settings')

        if (!saved) {
            // migrate flat keys to nested settings key
            saved = {
                lockedIn: (await store.get<boolean>('lockedIn')) ?? DEFAULTS.lockedIn,
                redRings: (await store.get<boolean>('redRings')) ?? DEFAULTS.redRings,
                discordRpcEnabled: (await store.get<boolean>('discordRpcEnabled')) ?? DEFAULTS.discordRpcEnabled,
                language: (await store.get<string>('language')) ?? DEFAULTS.language,
                robloxWarpped: (await store.get<boolean>('robloxWarpped')) ?? DEFAULTS.robloxWarpped,
                uwuEnabled: (await store.get<boolean>('uwuEnabled')) ?? DEFAULTS.uwuEnabled,
                hoverSfxEnabled: (await store.get<boolean>('hoverSfxEnabled')) ?? DEFAULTS.hoverSfxEnabled,
            }
            await store.delete('lockedIn')
            await store.delete('redRings')
            await store.delete('discordRpcEnabled')
            await store.delete('language')
            await store.delete('robloxWarpped')
            await store.delete('uwuEnabled')
            await store.delete('hoverSfxEnabled')
            await store.set('settings', saved)
            await store.save()
        }

        const data = { ...DEFAULTS, ...saved }
        this.lockedInMode = data.lockedIn
        this.discordRpcEnabled = data.discordRpcEnabled
        this.currentLocale = data.language
        this.robloxWarpped = data.robloxWarpped
        this.redRings = data.redRings
        this.uwuEnabled = data.uwuEnabled
        this.hoverSfxEnabled = data.hoverSfxEnabled
        this.loaded = true
    }

    async save() {
        const store = await load('config.json')
        await store.set('settings', {
            lockedIn: this.lockedInMode,
            redRings: this.redRings,
            discordRpcEnabled: this.discordRpcEnabled,
            language: this.currentLocale,
            robloxWarpped: this.robloxWarpped,
            uwuEnabled: this.uwuEnabled,
            hoverSfxEnabled: this.hoverSfxEnabled,
        } satisfies SettingsData)
        await store.save()
    }

    async setLockedIn(val: boolean) {
        this.lockedInMode = val
        await this.save()
    }

    async setRedRings(val: boolean) {
        this.redRings = val
        await this.save()
    }

    async setUwuEnabled(val: boolean) {
        this.uwuEnabled = val
        await this.save()
    }
}

export const settings = new AppSettings()