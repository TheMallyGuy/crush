import { fetch } from "@tauri-apps/plugin-http";
import { set, get, has, remove, clear, stats } from 'tauri-plugin-cache-api';


type adapter = "kliko" | "froststrap" // define all adapaters

type kiloAdapter = {
    id: string
    name: string
    author?: string
    description?: string
    download: string
    thumbnail?: string
}

type froststrapRaw = {
    communityMods?: frosttrapAdapter[]
}

type frosttrapAdapter = { // only include required fields
    id: string,
    name: string,
    download: string
    author: string
    description: string
    thumbnail: string
    modtype: number
}

export type universalAdapter = {
    id: string
    name: string
    author?: string
    description?: string
    download: string
    thumbnail?: string,
    adapter: string,
    modType?: "boostrap" | "mod"
}

export async function fetchMods(source: adapter): Promise<universalAdapter[]> {
    switch (source) {
        case 'kliko': {
            const exists = await has('kliko');

            if (exists) {
                const cached = await get<universalAdapter[]>("kliko")
                if (Array.isArray(cached)) return cached
            }

            const data = await fetch("https://raw.githubusercontent.com/klikos-modloader/marketplace/refs/heads/main/index.json")
            const items: kiloAdapter[] = await data.json()
            const mapped = items.map((item) => ({
                id: item.id ?? undefined,
                name: item.name ?? undefined,
                author: item.author ?? undefined,
                description: item.description ?? undefined,
                download: item.download ?? undefined,
                thumbnail: item.thumbnail ?? undefined,
                adapter: "Kilos Library",
                modType: "mod" as "mod" // always be mod because kilo lib always have mods
            }))

            await set("kliko", mapped, { ttl: 3600, compress: true })

            return mapped
        }
        case 'froststrap': {
            const exists = await has('froststrap');

            if (exists) {
                const cached = await get<universalAdapter[]>("froststrap")
                if (Array.isArray(cached)) return cached
            }

            const data = await fetch("https://raw.githubusercontent.com/RealMeddsam/config/refs/heads/main/Data.json")
            const raw: froststrapRaw = await data.json()
            const items: frosttrapAdapter[] = raw.communityMods ?? []

            const mapped = items.map((item) => ({
                id: item.id ?? undefined,
                name: item.name ?? undefined,
                author: item.author ?? undefined,
                description: item.description ?? undefined,
                download: item.download ?? undefined,
                thumbnail: item.thumbnail ?? undefined,
                adapter: "Froststrap Library",
                modType: (item.modtype === 5 ? "boostrap" : "mod") as "boostrap" | "mod"
            }))

            await set("froststrap", mapped, { ttl: 3600, compress: true })

            return mapped
        }

        default:
            return []
    }
}