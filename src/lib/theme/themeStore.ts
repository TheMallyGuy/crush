import { writable } from 'svelte/store'
import { convertFileSrc } from '@tauri-apps/api/core'
import type { BootstrapConfig } from '$lib/theme/xmlParser'

export interface ThemeState {
    themeName: string
    config?: BootstrapConfig
    customHtml?: string
    isHtmlTheme: boolean
    assetMap: Record<string, string>
}

export const themeStore = writable<ThemeState | null>(null)

export function normalizeAssetKey(source: string): string {
    if (source.startsWith('theme://')) return source.slice(8)
    return source
}

export function resolveAsset(
    state: ThemeState,
    source: string | undefined
): string {
    if (!source) return ''

    if (source.includes('://') && !source.startsWith('theme://')) return source

    const key = normalizeAssetKey(source)
    const mapped = state.assetMap[key]

    if (mapped) return mapped

    if (source.includes(':') || source.startsWith('/')) {
        return convertFileSrc(source)
    }

    return source
}
