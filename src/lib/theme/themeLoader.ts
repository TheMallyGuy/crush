import { open } from '@tauri-apps/plugin-dialog'
import {
    readFile,
    writeFile,
    mkdir,
    exists,
    readDir,
    remove,
} from '@tauri-apps/plugin-fs'
import { appDataDir, join, dirname, basename } from '@tauri-apps/api/path'
import { convertFileSrc } from '@tauri-apps/api/core'
import { parseXml } from '$lib/theme/xmlParser'
import { themeStore, normalizeAssetKey } from '$lib/theme/themeStore'
import type { ThemeState, LoadResult, BootstrapConfig } from '$lib/types'

function toErr(e: unknown): string {
    if (typeof e === 'string') return e
    if (e instanceof Error) return e.message
    try {
        return JSON.stringify(e)
    } catch {
        return 'Unknown error'
    }
}

function collectAssets(config: BootstrapConfig): string[] {
    const seen = new Set<string>()
    for (const el of config.elements) {
        const src =
            el.props.source ||
            el.props.Source ||
            el.props.ImageSource ||
            el.props.Background ||
            el.props.File
        if (typeof src !== 'string') continue

        if (src.startsWith('theme://')) {
            seen.add(src.slice(8))
        } else if (
            !src.includes('://') &&
            !src.includes(':') &&
            (src.endsWith('.png') ||
                src.endsWith('.jpg') ||
                src.endsWith('.gif') ||
                src.endsWith('.html'))
        ) {
            seen.add(src)
        }
    }
    return [...seen]
}

function safeName(name: string): string {
    return name.replace(/[^a-zA-Z0-9._-]/g, '_').slice(0, 64)
}

export async function loadThemeFromDialog(): Promise<LoadResult | null> {
    const selected = await open({
        title: 'Select Bootstrapper Theme',
        filters: [{ name: 'Theme Files', extensions: ['xml', 'html'] }],
        multiple: false,
    }).catch((e) => {
        throw new Error(`Dialog failed: ${toErr(e)}`)
    })

    if (!selected || typeof selected !== 'string') return null

    const filePath = selected
    const fileDir = await dirname(filePath)
    const rawBase = await basename(filePath)
    const isHtml = rawBase.toLowerCase().endsWith('.html')
    const nameOnly = rawBase.replace(/\.[^.]+$/, '')
    const themeName = safeName(nameOnly)

    const fileBytes = await readFile(filePath)
    const fileContent = new TextDecoder().decode(fileBytes)

    const appData = await appDataDir()
    const destDir = await join(appData, 'themes', themeName)
    await mkdir(destDir, { recursive: true })

    if (isHtml) {
        const destHtmlPath = await join(destDir, `custom.html`)
        await writeFile(destHtmlPath, fileBytes)
        const state: ThemeState = {
            themeName,
            customHtml: fileContent,
            isHtmlTheme: true,
            assetMap: {},
        }
        themeStore.set(state)
        return { state, themeName, destDir, missing: [] }
    } else {
        const config = parseXml(fileContent)
        const assetNames = collectAssets(config)
        const assetMap: Record<string, string> = {}
        const missing: string[] = []

        await Promise.all(
            assetNames.map(async (filename) => {
                const srcPath = await join(fileDir, filename)
                const destPath = await join(destDir, filename)

                if (!(await exists(srcPath).catch(() => false))) {
                    missing.push(filename)
                    return
                }

                const bytes = await readFile(srcPath)
                await writeFile(destPath, bytes)

                assetMap[normalizeAssetKey(filename)] = convertFileSrc(destPath)
            })
        )

        const destXmlPath = await join(destDir, `${themeName}.xml`)
        await writeFile(destXmlPath, fileBytes).catch(() => {})

        const state: ThemeState = {
            themeName,
            config,
            assetMap,
            isHtmlTheme: false,
        }
        themeStore.set(state)

        return { state, themeName, destDir, missing }
    }
}

export async function saveActiveTheme(themeName: string | null): Promise<void> {
    const appData = await appDataDir()
    const themesDir = await join(appData, 'themes')
    if (!(await exists(themesDir))) {
        await mkdir(themesDir, { recursive: true })
    }
    const activeJson = await join(themesDir, 'active.json')
    const content = JSON.stringify({ active: themeName })
    await writeFile(activeJson, new TextEncoder().encode(content))
}

export async function loadSavedTheme(): Promise<void> {
    try {
        const appData = await appDataDir()
        const activeJson = await join(appData, 'themes', 'active.json')
        if (await exists(activeJson)) {
            const bytes = await readFile(activeJson)
            const data = JSON.parse(new TextDecoder().decode(bytes))
            if (data.active) {
                await loadThemeFromAppData(data.active)
            }
        }
    } catch (e) {
        console.error('Failed to load saved theme:', e)
    }
}

export async function listThemes(): Promise<string[]> {
    try {
        const appData = await appDataDir()
        const themesDir = await join(appData, 'themes')
        if (!(await exists(themesDir))) return []

        const entries = await readDir(themesDir)
        return entries
            .filter((e) => e.isDirectory)
            .map((e) => e.name)
            .filter((n) => n !== 'active.json')
    } catch {
        return []
    }
}

export async function removeTheme(themeName: string): Promise<void> {
    const appData = await appDataDir()
    const themeDir = await join(appData, 'themes', themeName)
    if (await exists(themeDir)) {
        await remove(themeDir, { recursive: true })
    }
}

export async function loadThemeFromAppData(
    themeName: string
): Promise<LoadResult | null> {
    const appData = await appDataDir()
    const destDir = await join(appData, 'themes', themeName)

    // check for HTML theme first
    const htmlPath = await join(destDir, `custom.html`)
    if (await exists(htmlPath).catch(() => false)) {
        const fileBytes = await readFile(htmlPath)
        const fileContent = new TextDecoder().decode(fileBytes)
        const state: ThemeState = {
            themeName,
            customHtml: fileContent,
            isHtmlTheme: true,
            assetMap: {},
        }
        themeStore.set(state)
        return { state, themeName, destDir, missing: [] }
    }

    const xmlPath = await join(destDir, `${themeName}.xml`)
    if (!(await exists(xmlPath).catch(() => false))) return null

    const xmlBytes = await readFile(xmlPath)
    const xmlContent = new TextDecoder().decode(xmlBytes)
    const config = parseXml(xmlContent)

    const assetNames = collectAssets(config)
    const assetMap: Record<string, string> = {}
    const missing: string[] = []

    await Promise.all(
        assetNames.map(async (filename) => {
            const destPath = await join(destDir, filename)
            if (await exists(destPath).catch(() => false)) {
                assetMap[normalizeAssetKey(filename)] = convertFileSrc(destPath)
            } else {
                missing.push(filename)
            }
        })
    )

    const state: ThemeState = {
        themeName,
        config,
        assetMap,
        isHtmlTheme: false,
    }
    themeStore.set(state)
    return { state, themeName, destDir, missing }
}
