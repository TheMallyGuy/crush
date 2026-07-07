import { load } from "@tauri-apps/plugin-store";
import type { ServerInfoFromBackend } from "./types";
import { fetch } from "@tauri-apps/plugin-http";


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

const BATCH_SIZE = 10

export async function serverDataHandler(info: ServerInfoFromBackend) {
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