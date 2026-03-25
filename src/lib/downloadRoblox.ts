import { fetch } from '@tauri-apps/plugin-http'
import { invoke } from '@tauri-apps/api/core'
import { load, Store } from '@tauri-apps/plugin-store'
import { info } from '@tauri-apps/plugin-log'
import { exists, BaseDirectory, writeFile, mkdir } from '@tauri-apps/plugin-fs'
import { appCacheDir, appDataDir, join } from '@tauri-apps/api/path'

const extractRoots: Record<string, string> = {
    'RobloxApp.zip': '',
    'redist.zip': '',
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

export type ProgressEvent =
    | { type: 'status'; message: string }
    | { type: 'download'; file: string; done: number; total: number }
    | { type: 'extract'; file: string; done: number; total: number }

type ProgressCallback = (event: ProgressEvent) => void

async function ensureDir(path: string) {
    const existsDir = await exists(path)

    if (!existsDir) {
        await mkdir(path, { recursive: true })
    }
}

async function downloadAssets(
    assetsUrls: string[],
    onProgress: ProgressCallback,
    limit = 4
) {
    const queue = [...new Set(assetsUrls)]
    const total = queue.length
    let done = 0

    const workers = Array.from({ length: limit }, async () => {
        while (queue.length > 0) {
            const assetUrl = queue.shift()
            if (!assetUrl) return
            const match = assetUrl.match(/version-[^-]+-(.+)$/)
            let fileName = match?.[1] ?? `file-${Date.now()}`
            if (!fileName.endsWith('.zip')) fileName += '.zip'
            fileName = fileName.toLowerCase()

            try {
                const res = await fetch(assetUrl)
                if (!res.ok) throw new Error(`HTTP ${res.status}`)
                const buffer = await res.arrayBuffer()
                await writeFile(fileName, new Uint8Array(buffer), {
                    baseDir: BaseDirectory.AppCache,
                })
                done++
                onProgress({ type: 'download', file: fileName, done, total })
            } catch (err) {
                info(`Failed (${fileName}): ${err}`)
            }
        }
    })

    await Promise.all(workers)
}

async function extractAll(hash_version: string, onProgress: ProgressCallback) {
    const cacheDir = await appCacheDir()
    const dataDir = await appDataDir()
    const installRoot = await join(dataDir, 'Player', 'Versions', hash_version)
    await ensureDir(installRoot)

    const extractRootsLower = Object.fromEntries(
        Object.entries(extractRoots).map(([k, v]) => [k.toLowerCase(), v])
    )

    const entries = Object.entries(extractRootsLower)
    const total = entries.length
    let done = 0

    for (const [zipName, dest] of entries) {
        const zipPath = await join(cacheDir, zipName)
        const destPath = dest ? await join(installRoot, dest) : installRoot

        try {
            const existsZip = await exists(zipName, {
                baseDir: BaseDirectory.AppCache,
            })
            if (!existsZip) {
                done++
                onProgress({ type: 'extract', file: zipName, done, total })
                continue
            }
            await ensureDir(destPath)
            await invoke('extract_zip', { zipPath, dest: destPath })
            done++
            onProgress({ type: 'extract', file: zipName, done, total })
        } catch (err) {
            info(`Failed (${zipName}): ${err}`)
            done++
            onProgress({ type: 'extract', file: zipName, done, total })
        }
    }
}

type Versions = {
    versions: string[]
}

async function checkForUpdates(CurrentVersions: Versions): Promise<boolean> {
    const latest: string = await invoke('get_latest_version_player')

    return !CurrentVersions.versions.includes(latest)
}

export async function downloadRoblox(
    onProgress: ProgressCallback
): Promise<string> {
    const versionStore = await load('versions.json')
    const versionList = (await versionStore.get<string[]>('versions')) ?? []
    let version_hash = 'unknownversion'

    onProgress({ type: 'status', message: 'Checking for updates' })

    const latestVersion = versionList.at(-1)
    const dataDir = await appDataDir()
    let filesExist = false

    if (latestVersion) {
        const exePath = await join(
            dataDir,
            'Player',
            'Versions',
            latestVersion,
            'RobloxPlayerBeta.exe'  
        )
        filesExist = await exists(exePath)
    }

    if (await checkForUpdates({ versions: versionList }) || !filesExist) {
        onProgress({ type: 'status', message: 'Preparing download...' })

        const conf = await load('config.json')
        let bestRegion = await conf.get<string>('bestRegion')

        if (!bestRegion) {
            onProgress({ type: 'status', message: 'Finding best region...' })
            bestRegion = await invoke<string>('get_best_region')
            await conf.set('bestRegion', bestRegion)
            await conf.save()
        }

        onProgress({ type: 'status', message: 'Fetching asset URLs...' })
        const assetsUrls: string[] = await invoke('get_download_deployment_urls', {
            region: bestRegion,
        })

        onProgress({ type: 'status', message: 'Downloading assets...' })
        await downloadAssets(assetsUrls, onProgress)

        onProgress({ type: 'status', message: 'Extracting files...' })
        const match = assetsUrls[0].match(/(version-[^-]+)/)
        version_hash = match?.[1] ?? 'unknownversion'
        await extractAll(version_hash, onProgress)

        onProgress({ type: 'status', message: 'Writing AppSettings.xml...' })
        const xmlPath = await join(
            dataDir,
            'Player',
            'Versions',
            version_hash,
            'AppSettings.xml'
        )
        const xml = `<?xml version="1.0" encoding="UTF-8"?>
<Settings>
\t<ContentFolder>content</ContentFolder>
\t<BaseUrl>http://www.roblox.com</BaseUrl>
</Settings>`
        await writeFile(xmlPath, new TextEncoder().encode(xml))

        onProgress({ type: 'status', message: 'Saving version info...' })
        const updatedList = Array.from(new Set([...versionList, version_hash]))
        await versionStore.set('versions', updatedList)

        onProgress({ type: 'status', message: 'Installation complete!' })
    }

    return await invoke('get_latest_version_player')
}

export function getPackageForFile(relativePath: string): string | null {
    const normalized = relativePath.replace(/\\/g, '/')
    const [packageName] = sortedExtractRoots
        .find(([, prefix]) => prefix === '' || normalized.startsWith(prefix)) ?? []
    return packageName ?? null
}

export async function restoreFileFromPackage(
    input: string, // relativePath or direct packageName
    versionGuid: string,
    versionDir: string,
    isPackageInput = false
) {
    let packageName: string | null = null
    let prefix = ""

    if (isPackageInput) {
        packageName = input
        prefix = extractRoots[packageName] ?? ""
    } else {
        const normalized = input.replace(/\\/g, '/')
        const [pkg, pfx] = sortedExtractRoots
            .find(([, p]) => p === '' || normalized.startsWith(p)) ?? []
        packageName = pkg ?? null
        prefix = pfx ?? ""
    }

    if (!packageName) {
        info(`No package found for ${input}, skipping restore`)
        return
    }

    const cacheDir = await appCacheDir()
    const zipPath  = await join(cacheDir, packageName.toLowerCase())

    if (!await exists(zipPath)) {
        info(`Downloading ${packageName} to restore ${input}...`)
        const url = `https://setup.rbxcdn.com/${versionGuid}-${packageName}`
        const res = await fetch(url)
        if (!res.ok) throw new Error(`Failed to download ${packageName}: ${res.status}`)
        const buffer = await res.arrayBuffer()
        await writeFile(zipPath, new Uint8Array(buffer))
    }

    const destDir = prefix ? await join(versionDir, prefix) : versionDir
    await ensureDir(destDir)
    await invoke('extract_zip', { zipPath, dest: destDir })
}