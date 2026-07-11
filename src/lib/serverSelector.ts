import { load } from "@tauri-apps/plugin-store";
import type { ServerInfoFromBackend, Result } from "./types";
import { fetch } from "@tauri-apps/plugin-http";
import type { CloudLoginData } from "./stores/discordAuth.svelte";
import { invoke } from "@tauri-apps/api/core";


const DOMAIN = "https://crush-service.mally.qzz.io"

interface serverItem {
    gameId: number
    jobId: string
    region: string
    ip: string
}

type SubmitResult = {
    inserted: unknown[]
    skipped: string[]
}

const REGION_FORMAT = /^[A-Z][a-z]*(?:\s[A-Z][a-z]*)*,\s[A-Z][a-z]*(?:\s[A-Z][a-z]*)*$/i

async function submitNewData(data: serverItem[]): Promise<SubmitResult> {
    const payload = data.map((item) => ({
        gameId: item.gameId,
        jobId: item.jobId,
        fetchedRegion: item.region,
        serverIp: item.ip,
    }))


    const res = await fetch(`${DOMAIN}/v1/submissions/new`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(payload)
    })


    if (!res.ok) {
        const error = await res.json().catch(() => ({ error: res.statusText }))
        throw new Error(`submission failed (${res.status}): ${JSON.stringify(error)}`)
    }

    const result: SubmitResult = await res.json()
    console.log(`[serverSelector] submitted ${data.length} server(s)`, result)
    return result
}

export async function fetchRegions(): Promise<string[]> {
    const res = await fetch(`${DOMAIN}/v1/servers/regions`)

    if (!res.ok) {
        throw new Error(`Failed to fetch regions: ${res.status} ${res.statusText}`)
    }

    return res.json()
}

const BATCH_SIZE = 10

export async function serverDataHandler(info: ServerInfoFromBackend) {
    const configStore = await load("config.json")
    const settings = await configStore.get<{ autoSubmit?: boolean }>("serverManagement")
    if (settings?.autoSubmit === false) {
        return
    }

    if (info.is_private_server) {
        return
    }
    if (!info.ip) {
        return
    }
    if (!REGION_FORMAT.test(info.region_info)) {
        return
    }

    const store = await load("servers.list")
    const data = (await store.get<serverItem[]>('servers')) ?? [];

    data.push({
        gameId: info.game_id,
        jobId: info.server_id,
        region: info.region_info,
        ip: info.ip
    })

    if (data.length < BATCH_SIZE) {
        await store.set('servers', data)
        return
    }

    const submittable = data.filter((item) => REGION_FORMAT.test(item.region))

    if (submittable.length === 0) {
        await store.set('servers', [])
        return
    }

    try {
        await submitNewData(submittable)
        await store.set('servers', [])
    } catch (e) {
        console.error('[serverSelector] batch submission failed, keeping buffer', e)
        await store.set('servers', submittable)
    }
}

interface ValidateData {
    id: number,
    serverId: number,
    gameId: number,
    jobId: string
}

async function getToken(): Promise<string> {
    const store = await load("config.json")

    const data = await store.get<CloudLoginData>("serverService");

    if (!data) {
        throw new Error("error no key in store");
    }

    return data?.token
}

interface ServerMetaData {
    serverIp: string,
    gameId: number,
    jobId: string;
    fetchedRegion: string,
}

type JoinGameResponse = {
    status: number;
    jobId: string;
    joinScript: {
        UdmuxEndpoints: {
            Address: string
        }[]
    } | null
}

interface RobloxServerListEntry {
    id: string
    maxPlayers: number
    playing: number
    fps: number
    ping: number
}

interface RobloxServerListResponse {
    data: RobloxServerListEntry[]
    nextPageCursor: string | null
}

async function listPublicServers(gameId: number, amount: number): Promise<RobloxServerListEntry[]> {
    const servers: RobloxServerListEntry[] = []
    let cursor: string | null = null

    while (servers.length < amount) {
        const url = new URL(`https://games.roblox.com/v1/games/${gameId}/servers/Public`)
        url.searchParams.set('sortOrder', 'Asc')
        url.searchParams.set('limit', '100')
        if (cursor) url.searchParams.set('cursor', cursor)

        const res = await fetch(url.toString())

        if (!res.ok) {
            throw new Error(`Failed to list servers: ${res.status} ${res.statusText}`)
        }

        const page = await res.json() as RobloxServerListResponse
        servers.push(...page.data)

        if (!page.nextPageCursor) break
        cursor = page.nextPageCursor
    }

    return servers.slice(0, amount)
}

async function getRandomServers(amount: number, gameId: number): Promise<ServerMetaData[]> {
    const value = await getCookie()
    const cookie = `.ROBLOSECURITY=${value}`
    const csrf = await invoke<string>('get_csrf_token', { cookie })

    const targets = await listPublicServers(gameId, amount)
    const fetchedServers: ServerMetaData[] = []

    for (const server of targets) {
        const attemptId = crypto.randomUUID()

        let res: JoinGameResponse
        try {
            res = await invoke<JoinGameResponse>('join_game_instance', {
                cookie,
                csrf,
                gameId: server.id,
                placeId: gameId,
                attemptId,
            })
        } catch (err) {
            console.warn(`[serverSelector] join failed for instance ${server.id}, skipping`, err)
            continue
        }

        if (res.status !== 2 || !res.joinScript) {
            console.warn(`[serverSelector] instance ${server.id} not joinable (status ${res.status}), skipping`)
            continue
        }

        const geo = await (await fetch(`https://get.geojs.io/v1/ip/geo/${res.joinScript.UdmuxEndpoints[0].Address}.json`)).json() as { city: string, country: string };

        fetchedServers.push({
            serverIp: res.joinScript.UdmuxEndpoints[0].Address,
            gameId: gameId,
            jobId: res.jobId,
            fetchedRegion: `${geo.city}, ${geo.country}`
        })
    }

    return fetchedServers
}


export async function submissionRandomServer(amount: number, gameId: number) {
    const rand = await getRandomServers(amount, gameId)
    const authorization = await getToken()

    const res = await fetch(`${DOMAIN}/v1/submissions/manual`, {
        method: "POST",
        headers: {
            Authorization: `Bearer ${authorization}`,
            "Content-Type": "application/json"
        },
        body: JSON.stringify(rand)
    });

    if (!res.ok) throw new Error("went wrong when sub rand servers")
}


async function getValidateData(authorization: string): Promise<ValidateData[]> {
    const res = await fetch(`${DOMAIN}/v1/validation/batch?limit=50`, {
        headers: {
            Authorization: `Bearer ${authorization}`
        }
    });

    if (!res.ok) {
        throw new Error(`Failed to fetch validation data: ${res.status} ${res.statusText}`);
    }

    const data = await res.json();
    return data as ValidateData[];
}

type UpdatedReport = {
    updated: {
        id: number,
    }[]
    outdated: number[]
}

async function reportDeadOrAlive(authorization: string, updated: UpdatedReport) {
    const res = await fetch(`${DOMAIN}/v1/validation/report`, {
        method: "POST",
        headers: {
            Authorization: `Bearer ${authorization}`,
            "Content-Type": "application/json"
        },
        body: JSON.stringify(updated)
    });

    if (!res.ok) {
        const body = await res.text().catch(() => '<no body>')
        throw new Error(`Failed to report validation results: ${res.status} ${res.statusText} — ${body}`);
    }
}

interface ParsedCookie {
    domain: string
    httpOnly: boolean
    path: string
    secure: boolean
    expires: number
    name: string
    value: string
}

function parseNetscapeCookies(raw: string): ParsedCookie[] {
    const chunks = raw.split(/(?=#HttpOnly_)/g)
    const cookies: ParsedCookie[] = []

    for (const chunk of chunks) {
        const cleaned = chunk.replace(/^#HttpOnly_/, '').replace(/;\s*$/, '').trim()
        if (!cleaned) continue

        const fields = cleaned.split('\t')
        if (fields.length < 7) continue

        const [domain, , path, secure, expires, name, value] = fields
        cookies.push({
            domain,
            httpOnly: chunk.startsWith('#HttpOnly_'),
            path,
            secure: secure === 'TRUE',
            expires: Number(expires),
            name,
            value,
        })
    }

    return cookies
}

export async function getPerferedRegion(gameId: number, region: string): Promise<Result> {
    const res = await fetch(`${DOMAIN}/v1/games/give`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
            gameId: gameId,
            region: region
        })
    })

    if (!res.ok) throw new Error(`Failed to fetch preferred-region server: ${res.status} ${res.statusText}`)

    return res.json()
}

async function getCookie(): Promise<string> {
    const raw: string = await invoke("read_current_cookie")
    const { CookiesData } = JSON.parse(raw) as { CookiesData: string }
    const decrypted: string = await invoke("decrypt_cookie_data", { encrypted: CookiesData })

    const robloSecurity = parseNetscapeCookies(decrypted).find((c) => c.name === '.ROBLOSECURITY')

    if (!robloSecurity) {
        throw new Error('.ROBLOSECURITY cookie not found in decrypted cookie jar')
    }

    return robloSecurity.value
}


async function isServerInstanceAlive(gameId: number, instance: string) {
    const attemptId = crypto.randomUUID()
    const value = await getCookie()
    const cookie = `.ROBLOSECURITY=${value}`
    const csrf = await invoke<string>('get_csrf_token', { cookie })


    let data: { status: number }
    try {
        data = await invoke<{ status: number }>('join_game_instance', {
            cookie,
            csrf,
            gameId: instance,
            placeId: gameId,
            attemptId,
        })
    } catch (err) {
        throw new Error(`Failed to fetch validation data: ${err}`)
    }

    if (data.status == 2) {
        return true
    } else {
        return false
    }
}

export async function validateServers() {
    const authKey = await getToken()
    const serversUn = await getValidateData(authKey)

    const updated: { id: number }[] = []
    const outdated: number[] = []

    for (const item of serversUn) {
        const check = await isServerInstanceAlive(item.gameId, item.jobId)

        if (check) {
            updated.push({ id: item.id })
        } else {
            outdated.push(item.id)
        }
    }

    if (updated.length === 0 && outdated.length === 0) {
        return // nothing to report — the server rejects an empty report as 400
    }

    const report: UpdatedReport = {
        updated: updated,
        outdated: outdated
    }

    await reportDeadOrAlive(authKey, report)
}