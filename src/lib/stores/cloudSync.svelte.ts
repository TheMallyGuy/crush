import { openUrl } from '@tauri-apps/plugin-opener'
import { fetch } from '@tauri-apps/plugin-http'
import { load, Store } from '@tauri-apps/plugin-store'
import type {
    CloudConfig,
    CloudUniversalConfig,
    Installation,
    Integrations,
} from '$lib/types'
import { notify } from '$lib/notify'
import {
    encryptConfig,
    decryptConfig,
    WrongPasswordError,
} from '$lib/crypto/configCrypto'
import {
    crushToUniversal,
    universalToCrush,
    mergeUniversal,
    type CrushSnapshot,
} from '$lib/cloud/universal'
import { getFastFlags, saveFastFlags } from '$lib/fastflag/fastflagManagement'
import { _ } from 'svelte-i18n'
import { get } from 'svelte/store'

const BASE = 'https://cloud-config.mally.qzz.io/v1'
const POLL_INTERVAL = 2000
const LOGIN_TIMEOUT = 5 * 60_000
const AUTO_SYNC_INTERVAL = 60_000

const NS = 'pages.settings.cloudSync'
type Values = Record<string, string | number | boolean | Date | null | undefined>
const t = (key: string, values?: Values) =>
    values ? get(_)(key, { values }) : get(_)(key)

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
            throw new Error(t(`${NS}.errors.noPassword`))
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
                    this.authError = t(`${NS}.errors.pollFailed`, {
                        status: res.status,
                    })
                    return
                }

                await new Promise((r) => setTimeout(r, POLL_INTERVAL))
            }

            this.authError = t(`${NS}.errors.loginTimedOut`)
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


    async #readSnapshot(): Promise<CrushSnapshot> {
        const store = await this.#getStore()
        return {
            useFlag: (await store.get('useFlag')) as boolean | undefined,
            bestRegion: (await store.get('bestRegion')) as string | undefined,
            installation: (await store.get('installation')) as
                | Installation
                | undefined,
            integrations: (await store.get('integrations')) as
                | Integrations
                | undefined,
            lockedIn: (await store.get('lockedIn')) as boolean | undefined,
            redRings: (await store.get('redRings')) as boolean | undefined,
            fastFlags: await getFastFlags('player'),
        }
    }

    async #applySnapshot(s: CrushSnapshot) {
        const store = await this.#getStore()
        if (s.useFlag !== undefined) await store.set('useFlag', s.useFlag)
        if (s.bestRegion !== undefined)
            await store.set('bestRegion', s.bestRegion)
        if (s.installation) await store.set('installation', s.installation)
        if (s.integrations) await store.set('integrations', s.integrations)
        if (s.lockedIn !== undefined) await store.set('lockedIn', s.lockedIn)
        if (s.redRings !== undefined) await store.set('redRings', s.redRings)
        await store.save()
        if (s.fastFlags) await saveFastFlags(s.fastFlags, 'player')
    }


    async #contribution(): Promise<CloudUniversalConfig> {
        return crushToUniversal(await this.#readSnapshot())
    }


    async #fetchUniversal(
        password: string
    ): Promise<CloudUniversalConfig | null> {
        const res = await fetch(`${BASE}/config/sync`, {
            headers: { Authorization: `Bearer ${this.token}` },
        })
        if (!res.ok) return null // 500 = nothing stored yet
        const text = await res.text()
        if (!text.trim()) return null
        return JSON.parse(
            await decryptConfig(text, password)
        ) as CloudUniversalConfig
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

    async #pushContribution(password: string, contribution: CloudUniversalConfig) {
        const existing = await this.#fetchUniversal(password)
        const merged = mergeUniversal(existing, contribution)
        await this.#postSync(await encryptConfig(JSON.stringify(merged), password))
    }

    async autoSyncIfChanged() {
        if (!this.isLoggedIn || !this.autoSync || this.isSyncing) return
        if (!this.#password) return

        const contribution = await this.#contribution()
        const hash = await this.#hash(JSON.stringify(contribution))

        const store = await this.#getStore()
        const cloudConfig = (await store.get('cloudConfig')) as
            | CloudConfig
            | undefined
        if (cloudConfig?.lastSyncHash === hash) return

        this.isSyncing = true
        try {
            await this.#pushContribution(this.#password, contribution)
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
            const contribution = await this.#contribution()
            await this.#pushContribution(password, contribution)
            await this.#patchCloudConfig({
                lastSyncHash: await this.#hash(JSON.stringify(contribution)),
            })

            notify.send({
                title: t(`${NS}.notifies.success`),
                variant: 'success',
            })
        } catch (e) {
            notify.send({
                title: t(`${NS}.notifies.failed`),
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


            const universal = JSON.parse(
                await decryptConfig(await res.text(), password)
            ) as CloudUniversalConfig

            if (universal.schemaVersion !== 1) {
                throw new Error(
                    `unsupported cloud schema version: ${universal.schemaVersion}`
                )
            }


            const current = await this.#readSnapshot()
            await this.#applySnapshot(universalToCrush(universal, current))

            await this.#patchCloudConfig({
                lastSyncHash: await this.#hash(
                    JSON.stringify(await this.#contribution())
                ),
            })

            notify.send({
                title: t(`${NS}.notifies.syncFromSuccess`),
                variant: 'success',
            })
        } catch (e) {
            if (e instanceof WrongPasswordError) {
                notify.send({
                    title: t(`${NS}.notifies.wrongPassword`),
                    description: t(`${NS}.notifies.wrongPasswordDescription`),
                    variant: 'warning',
                })
            } else {
                notify.send({
                    title: t(`${NS}.notifies.syncFromFailed`),
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
