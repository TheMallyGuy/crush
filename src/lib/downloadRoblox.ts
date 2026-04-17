import { fetch } from '@tauri-apps/plugin-http'
import { invoke } from '@tauri-apps/api/core'
import { load, Store } from '@tauri-apps/plugin-store'
import { info } from '@tauri-apps/plugin-log'
import { exists, BaseDirectory, writeFile, mkdir } from '@tauri-apps/plugin-fs'
import { appCacheDir, appDataDir, join } from '@tauri-apps/api/path'
import { get } from 'svelte/store'
import { _ } from 'svelte-i18n'

const extractRoots: Record<string, string> = {
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

const sortedExtractRoots = Object.entries(extractRoots).sort(
    (a, b) => b[1].length - a[1].length
)

const lowercaseExtractRoots = Object.entries(extractRoots).map(([k, v]) => [
    k.toLowerCase(),
    v,
])

import type { ProgressEvent, ProgressCallback, Installation, Versions } from './types'

async function ensureDir(path: string) {
    const existsDir = await exists(path)

    if (!existsDir) {
        await mkdir(path, { recursive: true })
    }
}

async function downloadAssetFile(
    assetUrl: string,
    onProgress: ProgressCallback,
    done: number,
    total: number
) {
    const match = assetUrl.match(/version-[^-]+-(.+)$/)
    let fileName = match?.[1] ?? assetUrl.split('/').pop()?.split('?')[0] ?? `file-${done}.zip`
    if (!fileName.endsWith('.zip')) fileName += '.zip'
    fileName = fileName.toLowerCase()

    const res = await fetch(assetUrl)
    if (!res.ok){
        info(`error url : ${assetUrl}`)
        throw new Error(`HTTP ${res.status}`)
    }
    const buffer = await res.arrayBuffer()
    await writeFile(fileName, new Uint8Array(buffer), {
        baseDir: BaseDirectory.AppCache,
    })
    onProgress({ type: 'download', file: fileName, done, total })
}

async function downloadAssets(
    assetsUrls: string[],
    onProgress: ProgressCallback,
    limit = 4
) {
    const uniqueUrls = Array.from(new Set(assetsUrls))
    const total = uniqueUrls.length
    let done = 0
    let currentIndex = 0

    const workers = Array.from({ length: limit }, async () => {
        while (currentIndex < total) {
            const index = currentIndex++
            const assetUrl = uniqueUrls[index]
            if (!assetUrl) break

            await downloadAssetFile(assetUrl, onProgress, ++done, total)
        }
    })

    await Promise.all(workers)
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

async function extractAll(versionHash: string, onProgress: ProgressCallback) {
    const cacheDir = await appCacheDir()
    const dataDir = await appDataDir()
    const installRoot = await join(dataDir, 'Player', 'Versions', versionHash)
    await ensureDir(installRoot)

    const total = lowercaseExtractRoots.length

    for (const [index, [zipName, dest]] of lowercaseExtractRoots.entries()) {
        await extractIndividualZip(zipName, dest, installRoot, cacheDir)
        onProgress({ type: 'extract', file: zipName, done: index + 1, total })
    }
}

async function checkForUpdates(CurrentVersions: Versions): Promise<boolean> {
    const latest: string = await invoke('get_latest_version_player')

    return !CurrentVersions.versions.includes(latest)
}

export async function getLatestVersion(): Promise<string> {
    const versionStore = await load('versions.json')
    const versionList = (await versionStore.get<string[]>('versions')) ?? []
    const latestVersion = versionList.at(-1) ?? ''

    return latestVersion
}

async function resolveBestRegion(
    onProgress: ProgressCallback
): Promise<string> {
    const conf = await load('config.json')
    let bestRegion = await conf.get<string>('bestRegion')

    if (!bestRegion) {
        onProgress({ type: 'status', message: get(_)('typescript.downloader.findingBestRegion') })
        bestRegion = await invoke<string>('get_best_region')
        await conf.set('bestRegion', bestRegion)
        await conf.save()
    }
    return bestRegion
}

async function writeAppSettings(versionHash: string) {
    const dataDir = await appDataDir()
    const xmlPath = await join(
        dataDir,
        'Player',
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
    bestRegion: string,
    version?: string
): Promise<string[]> {
    const assetsUrls: string[] = await invoke('get_download_deployment_urls', {
        region: bestRegion,
        ...(version && { version }),
    })

    if (!assetsUrls || assetsUrls.length === 0) {
        throw new Error(
            'No download URLs found for the specified version/region.'
        )
    }

    return assetsUrls
}

async function processAssets(
    assetsUrls: string[],
    onProgress: ProgressCallback
) {
    onProgress({
        type: 'status',
        message: get(_)('typescript.downloader.downloadingAssets'),
    })
    await downloadAssets(assetsUrls, onProgress)

    onProgress({
        type: 'status',
        message: get(_)('typescript.downloader.extractingFiles'),
    })
    const versionHash =
        assetsUrls[0].match(/(version-[^-]+)/)?.[1] ?? 'unknownversion'
    await extractAll(versionHash, onProgress)

    return versionHash
}

async function completeInstallation(
    versionHash: string,
    onProgress: ProgressCallback
) {
    onProgress({
        type: 'status',
        message: get(_)('typescript.downloader.xmlWriting'),
    })
    await writeAppSettings(versionHash)
}

async function performFullInstallation(
    onProgress: ProgressCallback,
    version?: string
): Promise<string> {
    onProgress({
        type: 'status',
        message: get(_)('typescript.downloader.preparingForDownload'),
    })
    const bestRegion = await resolveBestRegion(onProgress)

    onProgress({
        type: 'status',
        message: get(_)('typescript.downloader.fetchingUrls'),
    })
    const assetsUrls = await getInstallationUrls(bestRegion, version)

    const versionHash = await processAssets(assetsUrls, onProgress)

    await completeInstallation(versionHash, onProgress)

    return versionHash
}

async function checkInstallationExists(version?: string): Promise<boolean> {
    if (!version) return false
    const dataDir = await appDataDir()
    const exePath = await join(
        dataDir,
        'Player',
        'Versions',
        version,
        'RobloxPlayerBeta.exe'
    )
    return await exists(exePath)
}

export async function downloadRoblox(
    onProgress: ProgressCallback,
    version?: string
): Promise<string> {
    const config = await load("config.json")
    const versionStore = await load('versions.json')
    const versionList = (await versionStore.get<string[]>('versions')) ?? []

    if (version) {
        return handleExplicitVersion(onProgress, version, versionList, versionStore)
    }

    return handleLatestVersion(onProgress, versionList, versionStore, config)
}

async function handleExplicitVersion(
    onProgress: ProgressCallback,
    version: string,
    versionList: string[],
    versionStore: Store
): Promise<string> {
    const isMissing = !(await checkInstallationExists(version))
    if (isMissing) {
        await performFullInstallation(onProgress, version)
    }

    await saveVersion(onProgress, version, versionList, versionStore)
    return version
}

async function handleLatestVersion(
    onProgress: ProgressCallback,
    versionList: string[],
    versionStore: Store,
    store: Store
): Promise<string> {
    onProgress({
        type: 'status',
        message: get(_)('typescript.downloader.updateChecking'),
    })

    const needsUpdate = await checkForUpdates({ versions: versionList })
    const isMissing = !(await checkInstallationExists(versionList.at(-1)))
    const installationCfg = await store.get<Installation>("installation")
    const shouldForceInstall = await installationCfg?.forceReinstall

    if (needsUpdate || isMissing || shouldForceInstall) {
        const versionHash = await performFullInstallation(onProgress)
        await saveVersion(onProgress, versionHash, versionList, versionStore)
    }

    return await invoke('get_latest_version_player')
}

async function saveVersion(
    onProgress: ProgressCallback,
    version: string,
    versionList: string[],
    versionStore: Store
) {
    onProgress({
        type: 'status',
        message: get(_)('typescript.downloader.versionSaving'),
    })

    const updatedList = Array.from(new Set([...versionList, version]))
    await versionStore.set('versions', updatedList)
    await versionStore.save()

    onProgress({
        type: 'status',
        message: get(_)('typescript.downloader.installationComplete'),
    })
}

export function getPackageForFile(relativePath: string): string | null {
    const normalized = relativePath.replace(/\\/g, '/').toLowerCase()
    const [packageName] =
        sortedExtractRoots.find(
            ([, prefix]) => prefix === '' || normalized.startsWith(prefix.toLowerCase())
        ) ?? []
    return packageName ?? null
}

export async function restoreFileFromPackage(
    input: string,
    versionGuid: string,
    versionDir: string,
    isPackageInput = false,
    files?: string[]
) {
    const { packageName, prefix } = resolvePackageInfo(input, isPackageInput)

    if (!packageName) {
        info(`No package found for ${input}, skipping restore`)
        return
    }

    const cacheDir = await appCacheDir()
    const zipPath = await join(cacheDir, packageName.toLowerCase())

    if (!(await exists(zipPath))) {
        await downloadMissingPackage(packageName, versionGuid, zipPath)
    }

    const destDir = prefix ? await join(versionDir, prefix) : versionDir
    await ensureDir(destDir)

    if (!files || files.length === 0) {
        await invoke('extract_zip', { zipPath, dest: destDir })
        return
    }

    const strippedFiles = stripPrefixFromFiles(files, prefix)
    await invoke('extract_files_from_zip', {
        zipPath,
        dest: destDir,
        files: strippedFiles,
    })
}

function stripPrefixFromFiles(files: string[], prefix: string): string[] {
    const prefixLower = prefix.toLowerCase()
    return files.map((f) => {
        const normalized = f.replace(/\\/g, '/')
        if (prefixLower && normalized.toLowerCase().startsWith(prefixLower)) {
            return normalized.substring(prefix.length)
        }
        return normalized
    })
}

export function resolvePackageInfo(input: string, isPackageInput: boolean) {
    if (isPackageInput) {
        return {
            packageName: input,
            prefix: extractRoots[input] ?? '',
        }
    }

    const normalized = input.replace(/\\/g, '/').toLowerCase()
    const [packageName, prefix] =
        sortedExtractRoots.find(
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
        throw new Error(get(_)('typescript.downloader.packageDownloadFailed', { values : { packageName, error: res.statusText } }))
    }

    const buffer = await res.arrayBuffer()
    await writeFile(zipPath, new Uint8Array(buffer))
}

export async function getCurrentInstallation(): Promise<{
    version: string
    installPath: string
    exists: boolean
} | null> {
    const versionStore = await load('versions.json')
    const versionList = (await versionStore.get<string[]>('versions')) ?? []
    const latestVersion = versionList.at(-1)

    if (!latestVersion) return null

    const dataDir = await appDataDir()
    const installPath = await join(dataDir, 'Player', 'Versions', latestVersion)
    const exePath = await join(installPath, 'RobloxPlayerBeta.exe')
    const installExists = await exists(exePath)

    return {
        version: latestVersion,
        installPath,
        exists: installExists,
    }
}