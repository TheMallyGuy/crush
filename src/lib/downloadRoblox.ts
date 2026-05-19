import { fetch } from '@tauri-apps/plugin-http'
import { invoke } from '@tauri-apps/api/core'
import { load, Store } from '@tauri-apps/plugin-store'
import { info } from '@tauri-apps/plugin-log'
import { exists, BaseDirectory, writeFile, mkdir, readDir, remove } from '@tauri-apps/plugin-fs'
import { appCacheDir, appDataDir, join } from '@tauri-apps/api/path'
import { get } from 'svelte/store'
import { _ } from 'svelte-i18n'

const playerExtractRoots: Record<string, string> = {
    'RobloxApp.zip': '',
    'shaders.zip': 'shaders/',
    'ssl.zip': 'ssl/',

    'WebView2.zip': '',
    'WebView2RuntimeInstaller.zip': 'WebView2RuntimeInstaller/',

    'content-avatar.zip': 'content/avatar/',
    'content-configs.zip': 'content/configs/',
    'content-fonts.zip': 'content/fonts/',
    'content-sky.zip': 'content/sky/',
    'content-sounds.zip': 'content/sounds/',
    'content-textures2.zip': 'content/textures/',
    'content-models.zip': 'content/models/',

    'content-platform-fonts.zip': 'PlatformContent/pc/fonts/',
    'content-platform-dictionaries.zip':
        'PlatformContent/pc/shared_compression_dictionaries/',
    'content-terrain.zip': 'PlatformContent/pc/terrain/',
    'content-textures3.zip': 'PlatformContent/pc/textures/',

    'extracontent-luapackages.zip': 'ExtraContent/LuaPackages/',
    'extracontent-translations.zip': 'ExtraContent/translations/',
    'extracontent-models.zip': 'ExtraContent/models/',
    'extracontent-textures.zip': 'ExtraContent/textures/',
    'extracontent-places.zip': 'ExtraContent/places/',
}

const studioExtractRoots: Record<string, string> = {
    'RobloxStudio.zip': '',
    'redist.zip': '',
    'Libraries.zip': '',
    'LibrariesQt5.zip': '',
    'WebView2.zip': '',
    'WebView2RuntimeInstaller.zip': 'WebView2RuntimeInstaller/',
    'shaders.zip': 'shaders/',
    'ssl.zip': 'ssl/',
    'Plugins.zip': 'Plugins/',
    'StudioFonts.zip': 'StudioFonts/',
    'BuiltInPlugins.zip': 'BuiltInPlugins/',
    'ApplicationConfig.zip': 'ApplicationConfig/',
    'BuiltInStandalonePlugins.zip': 'BuiltInStandalonePlugins/',
    'content-qt_translations.zip': 'content/qt_translations/',
    'content-sky.zip': 'content/sky/',
    'content-fonts.zip': 'content/fonts/',
    'content-avatar.zip': 'content/avatar/',
    'content-models.zip': 'content/models/',
    'content-sounds.zip': 'content/sounds/',
    'content-configs.zip': 'content/configs/',
    'content-api-docs.zip': 'content/api_docs/',
    'content-textures2.zip': 'content/textures/',
    'content-studio_svg_textures.zip': 'content/studio_svg_textures/',
    'content-platform-fonts.zip': 'PlatformContent/pc/fonts/',
    'content-platform-dictionaries.zip':
        'PlatformContent/pc/shared_compression_dictionaries/',
    'content-terrain.zip': 'PlatformContent/pc/terrain/',
    'content-textures3.zip': 'PlatformContent/pc/textures/',
    'extracontent-translations.zip': 'ExtraContent/translations/',
    'extracontent-luapackages.zip': 'ExtraContent/LuaPackages/',
    'extracontent-textures.zip': 'ExtraContent/textures/',
    'extracontent-scripts.zip': 'ExtraContent/scripts/',
    'extracontent-models.zip': 'ExtraContent/models/',
    'studiocontent-models.zip': 'StudioContent/models/',
    'studiocontent-textures.zip': 'StudioContent/textures/',
}

export type AppType = 'player' | 'studio'

function getExtractRoots(appType: AppType): Record<string, string> {
    return appType === 'studio' ? studioExtractRoots : playerExtractRoots
}

function getSortedExtractRoots(appType: AppType) {
    return Object.entries(getExtractRoots(appType)).sort(
        (a, b) => b[1].length - a[1].length
    )
}

function getLowercaseExtractRoots(appType: AppType) {
    return Object.entries(getExtractRoots(appType)).map(([k, v]) => [
        k.toLowerCase(),
        v,
    ])
}

// Keep legacy non-parameterised references for Player (backwards compat)
const sortedExtractRoots = Object.entries(playerExtractRoots).sort(
    (a, b) => b[1].length - a[1].length
)

import type { ProgressEvent, ProgressCallback, Installation, Versions, InstallationEntry } from './types'

async function reindexVersions(appType: AppType = 'player'): Promise<InstallationEntry[]> {
    const dataDir = await appDataDir()
    const appFolder = appType === 'studio' ? 'Studio' : 'Player'
    const versionsDir = await join(dataDir, appFolder, 'Versions')
    const exeName = appType === 'studio' ? 'RobloxStudioBeta.exe' : 'RobloxPlayerBeta.exe'

    const storeKey = 'versions.json'
    const versionStore = await load(storeKey)
    const storedVersions = (await versionStore.get<InstallationEntry[]>('versions')) ?? []

    const dirExists = await exists(versionsDir)
    if (!dirExists) {
        // wipe stored entries for this appType since the folder is gone
        const filtered = storedVersions.filter(e => e.appType !== appType)
        await versionStore.set('versions', filtered)
        await versionStore.save()
        return []
    }

    const entries = await readDir(versionsDir)
    const validEntries: InstallationEntry[] = []

    for (const entry of entries) {
        if (!entry.isDirectory) continue
        if (!entry.name || typeof entry.name !== 'string') continue
        if (!entry.name.startsWith('version-')) continue

        const exePath = await join(versionsDir, entry.name, exeName)

        const appSettingsPath = await join(versionsDir, entry.name, 'AppSettings.xml')

        const isComplete = await exists(exePath) && await exists(appSettingsPath)

        if (!isComplete) {
            info(`removing incomplete installation: ${entry.name}`)
            await remove(await join(versionsDir, entry.name), { recursive: true })

            continue
        }

        const existing = storedVersions.find(
            e => e.versionHash === entry.name && e.appType === appType
        )

        validEntries.push(
            existing ?? {
                id: `Installation ${validEntries.length + 1}`,
                versionHash: entry.name,
                appType,
                installedAt: new Date().toISOString(),
            }
        )
    }

    // merge

    const otherAppTypeEntries = storedVersions.filter(e => e.appType !== appType)
    const merged = [...otherAppTypeEntries, ...validEntries]
    await versionStore.set('versions', merged)

    await versionStore.save()

    return validEntries
}

async function ensureDir(path: string) {
    const existsDir = await exists(path)

    if (!existsDir) {
        await mkdir(path, { recursive: true })
    }
}

async function downloadAssetFile(assetUrl: string): Promise<string> {
    const match = assetUrl.match(/version-[^-]+-(.+)$/)
    let fileName = match?.[1] ?? assetUrl.split('/').pop()?.split('?')[0] ?? `file.zip`
    if (!fileName.endsWith('.zip')) fileName += '.zip'
    fileName = fileName.toLowerCase()

    const res = await fetch(assetUrl)
    if (!res.ok) {
        info(`error url : ${assetUrl}`)
        throw new Error(`HTTP ${res.status}: ${res.statusText} ${assetUrl}`)
    }
    const buffer = await res.arrayBuffer()
    await writeFile(fileName, new Uint8Array(buffer), {
        baseDir: BaseDirectory.AppCache,
    })

    return fileName
}

async function downloadAssets(
    assetsUrls: string[],
    onProgress: ProgressCallback,
    limit = 4
) {
    const uniqueUrls = Array.from(new Set(assetsUrls))
    const total = uniqueUrls.length
    let completed = 0
    let currentIndex = 0
    let aborted = false
    let firstError: Error | null = null

    const workers = Array.from({ length: limit }, async () => {
        while (currentIndex < total) {
            if (aborted) return
            const index = currentIndex++
            const assetUrl = uniqueUrls[index]
            if (!assetUrl) break

            try {
                const fileName = await downloadAssetFile(assetUrl)
                if (!aborted)
                    onProgress({ type: 'download', file: fileName, done: ++completed, total })
            } catch (err) {
                aborted = true
                firstError = err instanceof Error ? err : new Error(String(err))
                return
            }
        }
    })

    await Promise.all(workers)
    if (firstError) throw firstError
}

async function extractIndividualZip(
    zipName: string,
    dest: string,
    installRoot: string,
    cacheDir: string
) {
    const zipPath = await join(cacheDir, zipName)
    const destPath = dest ? await join(installRoot, dest) : installRoot

    const zipExists = await exists(zipPath)
    if (!zipExists) return false

    await ensureDir(destPath)
    await invoke('extract_zip', { zipPath, dest: destPath })
    return true
}

async function extractAll(
    versionHash: string,
    onProgress: ProgressCallback,
    appType: AppType = 'player'
) {
    const cacheDir = await appCacheDir()
    const dataDir = await appDataDir()
    const appFolder = appType === 'studio' ? 'Studio' : 'Player'
    const installRoot = await join(dataDir, appFolder, 'Versions', versionHash)
    await ensureDir(installRoot)

    const lowercaseRoots = getLowercaseExtractRoots(appType)
    const total = lowercaseRoots.length

    for (const [index, [zipName, dest]] of lowercaseRoots.entries()) {
        await extractIndividualZip(zipName, dest, installRoot, cacheDir)
        onProgress({ type: 'extract', file: zipName, done: index + 1, total })
    }
}

async function checkForUpdates(
    CurrentVersions: Versions,
    appType: AppType = 'player'
): Promise<boolean> {
    const latest: string = await invoke(
        appType === 'studio' ? 'get_latest_version_studio' : 'get_latest_version_player'
    )
    return !CurrentVersions.versions.includes(latest)
}

export async function getLatestVersion(appType: AppType = 'player'): Promise<string> {
    const versionStore = await load('versions.json')
    const installations = (await versionStore.get<InstallationEntry[]>('versions')) ?? []
    return installations.filter(i => i.appType === appType).at(-1)?.versionHash ?? ''
}

async function resolveBestRegion(onProgress: ProgressCallback): Promise<string> {
    const conf = await load('config.json')
    let bestRegion = await conf.get<string>('bestRegion')

    if (!bestRegion) {
        onProgress({
            type: 'status',
            message: get(_)('typescript.downloader.findingBestRegion'),
        })
        bestRegion = await invoke<string>('get_best_region')
        await conf.set('bestRegion', bestRegion)
        await conf.save()
    }
    return bestRegion
}

async function writeAppSettings(versionHash: string, appType: AppType = 'player') {
    const dataDir = await appDataDir()
    const appFolder = appType === 'studio' ? 'Studio' : 'Player'
    const exeName =
        appType === 'studio' ? 'RobloxStudioBeta.exe' : 'RobloxPlayerBeta.exe'
    const xmlPath = await join(
        dataDir,
        appFolder,
        'Versions',
        versionHash,
        'AppSettings.xml'
    )
    const xml = `<?xml version="1.0" encoding="UTF-8"?>
<Settings>
\t<ContentFolder>content</ContentFolder>
\t<BaseUrl>http://www.roblox.com</BaseUrl>
</Settings>`
    await writeFile(xmlPath, new TextEncoder().encode(xml))
}

async function getInstallationUrls(
    onProgress: ProgressCallback,
    appType: AppType = 'player',
    version?: string
): Promise<string[]> {
    onProgress({
        type: 'status',
        message: get(_)('typescript.downloader.preparingForDownload'),
    })
    const bestRegion = await resolveBestRegion(onProgress)

    onProgress({
        type: 'status',
        message: get(_)('typescript.downloader.fetchingUrls'),
    })

    const assetsUrls: string[] = await invoke('get_download_deployment_urls', {
        player: appType === 'player',
        region: bestRegion,
        ...(version && { version }),
    })

    if (!assetsUrls || assetsUrls.length === 0) {
        throw new Error('No download URLs found for the specified version/region.')
    }

    return assetsUrls
}

async function completeInstallation(
    onProgress: ProgressCallback,
    versionHash: string,
    appType: AppType = 'player'
): Promise<void> {
    onProgress({
        type: 'status',
        message: get(_)('typescript.downloader.extractingFiles'),
    })
    await extractAll(versionHash, onProgress, appType)

    onProgress({
        type: 'status',
        message: get(_)('typescript.downloader.xmlWriting'),
    })
    await writeAppSettings(versionHash, appType)
}

async function performFullInstallation(
    onProgress: ProgressCallback,
    appType: AppType = 'player',
    version?: string,
): Promise<string> {
    const store = await load('config.json')
    const assetsUrls = await getInstallationUrls(onProgress, appType, version)
    const installation = await store.get<Installation>('installation')
    const limit = installation?.parallel ?? 4

    onProgress({
        type: 'status',
        message: get(_)('typescript.downloader.downloadingAssets'),
    })
    await downloadAssets(assetsUrls, onProgress, limit)

    const versionHash =
        assetsUrls[0].match(/(version-[^-]+)/)?.[1] ?? 'unknownversion'

    await completeInstallation(onProgress, versionHash, appType)

    return versionHash
}

async function checkInstallationExists(
    appType: AppType = 'player',
    version?: string
): Promise<boolean> {
    if (!version || typeof version !== 'string') return false
    const dataDir = await appDataDir()
    const appFolder = appType === 'studio' ? 'Studio' : 'Player'
    const exeName =
        appType === 'studio' ? 'RobloxStudioBeta.exe' : 'RobloxPlayerBeta.exe'
    const exePath = await join(dataDir, appFolder, 'Versions', version, exeName)
    return await exists(exePath)
}

export async function downloadRoblox(
    onProgress: ProgressCallback,
    appType: AppType = 'player',
): Promise<string> {
    const config = await load('config.json')
    const storeKey = appType === 'studio' ? 'studio-versions.json' : 'versions.json'
    const versionStore = await load(storeKey)

    // reindex
    const actualEntries = await reindexVersions(appType)
    const actualVersions = actualEntries.map(e => e.versionHash)

    const storedEntries = (await versionStore.get<InstallationEntry[]>('versions')) ?? []
    const storedVersions = storedEntries.map(e => e.versionHash)

    if (actualVersions.length !== storedVersions.length ||
        actualVersions.some(v => !storedVersions.includes(v))) {
        info(`Version list mismatch, reindexing.`)
        await versionStore.set('versions', actualEntries)
        await versionStore.save()
    }

    const savedInstallation = await config.get<Installation>('installation')
    if (savedInstallation?.dontUpdate) {
        const current = await versionStore.get<InstallationEntry>('currentlyUsing')
        return current?.versionHash ?? actualVersions.at(-1) ?? ''
    }

    return handleLatestVersion(onProgress, appType, actualEntries, versionStore, config)
}

async function handleExplicitVersion(
    onProgress: ProgressCallback,
    appType: AppType,
    version: string,
    versionList: string[],
    versionStore: Store
): Promise<string> {
    const isMissing = !(await checkInstallationExists(appType, version))
    if (isMissing) {
        await performFullInstallation(onProgress, appType, version)
    }

    await saveVersion(onProgress, version, versionStore, appType)
    return version
}

async function handleLatestVersion(
    onProgress: ProgressCallback,
    appType: AppType,
    entries: InstallationEntry[],
    versionStore: Store,
    store: Store
): Promise<string> {
    onProgress({
        type: 'status',
        message: get(_)('typescript.downloader.updateChecking'),
    })

    const versionHashes = entries.map(e => e.versionHash)
    const needsUpdate = await checkForUpdates({ versions: versionHashes }, appType)
    const currentlyUsing = await versionStore.get<InstallationEntry>('currentlyUsing')
    const isMissing = !(await checkInstallationExists(appType, currentlyUsing?.versionHash))
    const installationCfg = await store.get<Installation>('installation')
    const shouldForceInstall = installationCfg?.forceReinstall

    if (needsUpdate || isMissing || shouldForceInstall) {
        const versionHash = await performFullInstallation(onProgress, appType)
        await saveVersion(onProgress, versionHash, versionStore, appType)
        return versionHash
    }

    // no update needed
    return currentlyUsing?.versionHash ?? versionHashes.at(-1) ?? ''
}

async function saveVersion(
    onProgress: ProgressCallback,
    version: string,
    versionStore: Store,
    appType: AppType = 'player',
    label?: string
) {
    onProgress({
        type: 'status',
        message: get(_)('typescript.downloader.versionSaving'),
    })

    const existing = (await versionStore.get<InstallationEntry[]>('versions')) ?? []
    const alreadyExists = existing.some(e => e.versionHash === version && e.appType === appType)

    const newEntry: InstallationEntry = alreadyExists
        ? existing.find(e => e.versionHash === version && e.appType === appType)!
        : {
            id: label ?? `Installation ${existing.length + 1}`,
            versionHash: version,
            appType,
            installedAt: new Date().toISOString(),
        }

    if (!alreadyExists) {
        existing.push(newEntry)
        await versionStore.set('versions', existing)
    }

    // always update currentlyUsing to the latest launched version
    await versionStore.set('currentlyUsing', newEntry)
    await versionStore.save()

    onProgress({
        type: 'status',
        message: get(_)('typescript.downloader.installationComplete'),
    })
}

export function getPackageForFile(
    relativePath: string,
    appType: AppType = 'player'
): string | null {
    const normalized = relativePath.replace(/\\/g, '/').toLowerCase()
    const sorted = getSortedExtractRoots(appType)
    const [packageName] =
        sorted.find(
            ([, prefix]) => prefix === '' || normalized.startsWith(prefix.toLowerCase())
        ) ?? []
    return packageName ?? null
}

export async function restoreFileFromPackage(
    input: string,
    versionGuid: string,
    versionDir: string,
    isPackageInput = false,
    files?: string[],
    appType: AppType = 'player'
) {
    const { packageName, prefix } = resolvePackageInfo(input, isPackageInput, appType)

    if (!packageName) {
        info(`No package found for ${input}, skipping restore`)
        return
    }

    const cacheDir = await appCacheDir()
    const zipFileName = packageName.toLowerCase()
    const zipPath = await join(cacheDir, zipFileName)

    info(`Checking zip exists at: ${zipPath}`)
    const zipExists = await exists(zipPath)
    info(`Zip exists: ${zipExists}`)

    if (!zipExists) {
        await downloadMissingPackage(packageName, versionGuid, zipPath)
    }

    const destDir = prefix ? await join(versionDir, prefix) : versionDir
    await ensureDir(destDir)

    if (!files || files.length === 0) {
        await invoke('extract_zip', { zipPath, dest: destDir })
        return
    }

    const prefixLower = prefix.toLowerCase()
    const strippedFiles = files.map((f) => {
        const normalized = f.replace(/\\/g, '/')
        const normalizedLower = normalized.toLowerCase()
        if (prefixLower && normalizedLower.startsWith(prefixLower)) {
            return normalized.substring(prefix.length)
        }
        return normalized
    })

    info(`stripping prefix "${prefix}" from ${files.length} files`)
    info(`sample stripped: ${strippedFiles[0]}`)
    info(`extracting files: ${JSON.stringify(strippedFiles)} from ${zipPath} to ${destDir}`)

    await invoke('extract_files_from_zip', {
        zipPath,
        dest: destDir,
        files: strippedFiles,
    })
}

export function resolvePackageInfo(
    input: string,
    isPackageInput: boolean,
    appType: AppType = 'player'
) {
    const extractRoots = getExtractRoots(appType)

    if (isPackageInput) {
        return {
            packageName: input,
            prefix: extractRoots[input] ?? '',
        }
    }

    const normalized = input.replace(/\\/g, '/').toLowerCase()
    const sorted = getSortedExtractRoots(appType)
    const [packageName, prefix] =
        sorted.find(
            ([, p]) => p === '' || normalized.startsWith(p.toLowerCase())
        ) ?? []

    return {
        packageName: packageName ?? null,
        prefix: prefix ?? '',
    }
}

async function downloadMissingPackage(
    packageName: string,
    versionGuid: string,
    zipPath: string
) {
    info(`Downloading ${packageName} for restore...`)
    const url = `https://setup.rbxcdn.com/${versionGuid}-${packageName}`
    const res = await fetch(url)
    if (!res.ok) {
        throw new Error(
            get(_)('typescript.downloader.packageDownloadFailed', {
                values: { packageName, error: res.statusText },
            })
        )
    }

    const buffer = await res.arrayBuffer()
    await writeFile(zipPath, new Uint8Array(buffer))
}

export async function getCurrentInstallation(appType: AppType = 'player'): Promise<{
    version: string
    installPath: string
    exists: boolean
} | null> {
    const storeKey = appType === 'studio' ? 'studio-versions.json' : 'versions.json'
    const versionStore = await load(storeKey)
    const versionList = (await versionStore.get<InstallationEntry[]>('versions')) ?? []
    const latest = versionList.filter(e => e.appType === appType).at(-1)

    if (!latest) return null


    const dataDir = await appDataDir()
    const appFolder = appType === 'studio' ? 'Studio' : 'Player'
    const exeName = appType === 'studio' ? 'RobloxStudioBeta.exe' : 'RobloxPlayerBeta.exe'
    const installPath = await join(dataDir, appFolder, 'Versions', latest.versionHash)
    const exePath = await join(installPath, exeName)
    const installExists = await exists(exePath)


    return {
        version: latest.versionHash,
        installPath,
        exists: installExists,
    }
}