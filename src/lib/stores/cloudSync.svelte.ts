import { openUrl } from '@tauri-apps/plugin-opener'
import { fetch } from '@tauri-apps/plugin-http'
import { load, Store } from '@tauri-apps/plugin-store'
import type { CloudConfig } from '$lib/types'
import { notify } from '$lib/notify'
import {
    encryptConfig,
    decryptConfig,
    WrongPasswordError,
} from '$lib/crypto/configCrypto'

const BASE = 'https://cloud-config.mally.qzz.io'
const POLL_INTERVAL = 2000
const LOGIN_TIMEOUT = 5 * 60_000
const AUTO_SYNC_INTERVAL = 60_000

// this was hard to make

class CloudSync {
    isLoggedIn = $state(false)
    isInAuth = $state(false)
    isSyncing = $state(false)
    autoSync = $state(true)
    token = $state('')
    hasPassword = $state(false)
    authError = $state<string | null>(null)

    #config: Store | null = null
    #autoSyncTimer: ReturnType<typeof setInterval> | null = null
    #password = ''

    async #getStore(): Promise<Store> {
        if (!this.#config) {
            this.#config = await load('config.json')
        }
        return this.#config
    }

    async #patchCloudConfig(patch: Partial<CloudConfig>) {
        const store = await this.#getStore()
        const current: CloudConfig = (await store.get('cloudConfig')) ?? {
            authToken: '',
            automaticallyCloud: true,
        }
        const next: CloudConfig = { ...current, ...patch }
        await store.set('cloudConfig', next)
        await store.save()
    }

    async init() {
        const store = await this.#getStore()
        const cloudConfig: CloudConfig = (await store.get('cloudConfig')) ?? {
            authToken: '',
            automaticallyCloud: true,
        }

        this.token = cloudConfig.authToken ?? ''
        this.isLoggedIn = Boolean(this.token)
        this.autoSync = cloudConfig.automaticallyCloud ?? true
        this.#password = cloudConfig.syncPassword ?? ''
        this.hasPassword = Boolean(this.#password)

        if (this.token) {
            await this.#verifyToken()
        }

        this.startAutoSync()
    }

    async setPassword(password: string) {
        this.#password = password
        this.hasPassword = Boolean(password)
        await this.#patchCloudConfig({ syncPassword: password })
    }

    #requirePassword(): string {
        if (!this.#password) {
            throw new Error('set an encryption password first')
        }
        return this.#password
    }

    startAutoSync(intervalMs = AUTO_SYNC_INTERVAL) {
        this.stopAutoSync()
        this.#autoSyncTimer = setInterval(() => {
            void this.autoSyncIfChanged()
        }, intervalMs)
    }

    stopAutoSync() {
        if (this.#autoSyncTimer) {
            clearInterval(this.#autoSyncTimer)
            this.#autoSyncTimer = null
        }
    }

    async #verifyToken() {
        const res = await fetch(`${BASE}/auth/me`, {
            method: 'POST',
            headers: { Authorization: `Bearer ${this.token}` },
        })

        if (!res.ok) {
            this.isLoggedIn = false
        }
    }

    async login() {
        this.isInAuth = true
        this.authError = null

        const pair = crypto.randomUUID()

        try {
            await openUrl(`${BASE}/auth/login?pair=${pair}`)

            const deadline = Date.now() + LOGIN_TIMEOUT
            while (Date.now() < deadline) {
                const res = await fetch(`${BASE}/auth/poll?pair=${pair}`)

                if (res.status === 200) {
                    const body = await res.json()
                    this.token = body.token
                    await this.#patchCloudConfig({ authToken: this.token })
                    this.isLoggedIn = true
                    return
                }

                if (res.status !== 204) {
                    this.authError = `poll failed: ${res.status}`
                    return
                }

                await new Promise((r) => setTimeout(r, POLL_INTERVAL))
            }

            this.authError = 'login timed out'
        } catch (e) {
            console.error('auth error', e)
            this.authError = e instanceof Error ? e.message : String(e)
        } finally {
            this.isInAuth = false
        }
    }

    async setAutoSync(checked: boolean) {
        this.autoSync = checked
        await this.#patchCloudConfig({ automaticallyCloud: checked })
    }


    async #buildSyncPayload(): Promise<string> {
        const store = await this.#getStore()
        const data = Object.fromEntries(await store.entries())

        const cloudConfig = data.cloudConfig as CloudConfig | undefined
        if (cloudConfig) {
            const { authToken, lastSyncHash, syncPassword, ...rest } =
                cloudConfig
            data.cloudConfig = rest
        }

        return JSON.stringify(data)
    }

    async #hash(text: string): Promise<string> {
        const digest = await crypto.subtle.digest(
            'SHA-256',
            new TextEncoder().encode(text)
        )
        return Array.from(new Uint8Array(digest))
            .map((b) => b.toString(16).padStart(2, '0'))
            .join('')
    }

    async #postSync(payload: string) {
        const res = await fetch(`${BASE}/config/sync`, {
            method: 'POST',
            headers: {
                Authorization: `Bearer ${this.token}`,
                'Content-Type': 'text/plain',
            },
            body: payload,
        })
        if (!res.ok) throw new Error(`sync failed: ${res.status}`)
    }

    async autoSyncIfChanged() {
        if (!this.isLoggedIn || !this.autoSync || this.isSyncing) return
        if (!this.#password) return
        const payload = await this.#buildSyncPayload()
        const hash = await this.#hash(payload)

        const store = await this.#getStore()
        const cloudConfig = (await store.get('cloudConfig')) as
            | CloudConfig
            | undefined
        if (cloudConfig?.lastSyncHash === hash) return

        this.isSyncing = true
        try {
            await this.#postSync(await encryptConfig(payload, this.#password))
            await this.#patchCloudConfig({ lastSyncHash: hash })
        } catch (e) {
            console.error('auto sync failed', e)
        } finally {
            this.isSyncing = false
        }
    }

    async syncToCloud() {
        if (this.isSyncing) return
        this.isSyncing = true
        try {
            const password = this.#requirePassword()
            const payload = await this.#buildSyncPayload()
            await this.#postSync(await encryptConfig(payload, password))
            await this.#patchCloudConfig({ lastSyncHash: await this.#hash(payload) })

            notify.send({ title: 'Synced to cloud', variant: 'success' })
        } catch (e) {
            notify.send({
                title: 'Sync to cloud failed',
                description: e instanceof Error ? e.message : String(e),
                variant: 'danger',
            })
        } finally {
            this.isSyncing = false
        }
    }

    async syncFromCloud() {
        if (this.isSyncing) return
        this.isSyncing = true
        try {
            const password = this.#requirePassword()
            const res = await fetch(`${BASE}/config/sync`, {
                headers: { Authorization: `Bearer ${this.token}` },
            })
            if (!res.ok) throw new Error(`sync failed: ${res.status}`)

            const plaintext = await decryptConfig(await res.text(), password)
            const pulled = JSON.parse(plaintext) as Record<string, unknown>
            const store = await this.#getStore()


            const localCloud = (await store.get('cloudConfig')) as
                | CloudConfig
                | undefined
            pulled.cloudConfig = {
                ...(pulled.cloudConfig as CloudConfig | undefined),
                authToken: localCloud?.authToken ?? this.token,
                syncPassword: localCloud?.syncPassword ?? this.#password,
            }

            for (const [key, value] of Object.entries(pulled)) {
                await store.set(key, value)
            }
            await store.save()

            this.autoSync =
                (pulled.cloudConfig as CloudConfig).automaticallyCloud ??
                this.autoSync


            await this.#patchCloudConfig({
                lastSyncHash: await this.#hash(await this.#buildSyncPayload()),
            })

            notify.send({ title: 'Synced from cloud', variant: 'success' })
        } catch (e) {
            if (e instanceof WrongPasswordError) {
                notify.send({
                    title: 'Wrong password',
                    description:
                        'Your config was encrypted with a different password. Nothing was changed.',
                    variant: 'warning',
                })
            } else {
                notify.send({
                    title: 'Sync from cloud failed',
                    description: e instanceof Error ? e.message : String(e),
                    variant: 'danger',
                })
            }
        } finally {
            this.isSyncing = false
        }
    }
}

export const cloud = new CloudSync()
