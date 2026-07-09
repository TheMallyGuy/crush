import { fetch } from "@tauri-apps/plugin-http";
import { openUrl } from "@tauri-apps/plugin-opener";
import { load, type Store } from "@tauri-apps/plugin-store";

interface DiscordProfile {
    id: string
    username: string
    avatar: string | null
}

interface CloudLoginData {
    token: string
    expiresAt?: string
    user?: DiscordProfile
}

const DOMAIN = "https://crush-service.mally.qzz.io"
const POLL_INTERVAL = 2000
const LOGIN_TIMEOUT = 5 * 60_000

export class CloudService {
    inAuth = $state(false)
    isLoggedIn = $state(false)
    authError = $state<string | null>(null)
    user = $state<DiscordProfile | null>(null)
    validatedCount = $state(0)

    #config: Store | null = null
    #token = ''

    async #getStore(): Promise<Store> {
        if (!this.#config) {
            this.#config = await load('config.json')
        }
        return this.#config
    }

    async #persist(data: CloudLoginData) {
        const store = await this.#getStore()
        await store.set('serverService', data)
        await store.save()
    }

    async #clear() {
        this.#token = ''
        this.user = null
        this.isLoggedIn = false
        this.validatedCount = 0
        await this.#persist({ token: '' })
    }

    async init() {
        const store = await this.#getStore()
        const loginData: CloudLoginData = (await store.get('serverService')) ?? { token: '' }

        this.#token = loginData.token
        this.user = loginData.user ?? null

        if (!this.#token) {
            this.isLoggedIn = false
            return
        }

        const res = await fetch(`${DOMAIN}/v1/validation/me`, {
            headers: { Authorization: `Bearer ${this.#token}` },
        })

        if (!res.ok) {
            await this.#clear()
            return
        }

        const me = await res.json()
        this.user = { id: me.id, username: me.username, avatar: me.avatar }
        this.validatedCount = me.validatedCount ?? 0
        this.isLoggedIn = true
        await this.#persist({ token: this.#token, user: this.user })
    }

    async auth() {
        this.inAuth = true
        this.authError = null

        const pair = crypto.randomUUID()

        try {
            await openUrl(`${DOMAIN}/auth/discord?pair=${pair}`)

            const deadline = Date.now() + LOGIN_TIMEOUT
            while (Date.now() < deadline) {
                const res = await fetch(`${DOMAIN}/auth/pair?pair=${pair}`)

                if (res.status === 200) {
                    const body = await res.json()
                    this.#token = body.token
                    this.user = body.user
                    this.isLoggedIn = true
                    await this.#persist({
                        token: body.token,
                        expiresAt: body.expiresAt,
                        user: body.user,
                    })
                    return
                }

                if (res.status !== 204) {
                    this.authError = `Login failed (${res.status})`
                    return
                }

                await new Promise((r) => setTimeout(r, POLL_INTERVAL))
            }

            this.authError = 'Login timed out'
        } catch (e) {
            console.error('[serverManagement] auth error', e)
            this.authError = e instanceof Error ? e.message : String(e)
        } finally {
            this.inAuth = false
        }
    }

    async logout() {
        if (this.#token) {
            await fetch(`${DOMAIN}/auth/logout`, {
                method: 'POST',
                headers: { Authorization: `Bearer ${this.#token}` },
            }).catch(() => { })
        }

        await this.#clear()
    }
}

export const serverService = new CloudService()
