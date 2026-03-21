import { fetch } from '@tauri-apps/plugin-http';
import { invoke } from '@tauri-apps/api/core';
import { load, Store } from '@tauri-apps/plugin-store';
import { info } from '@tauri-apps/plugin-log';
import { exists, BaseDirectory, writeFile, mkdir } from '@tauri-apps/plugin-fs';
import { appCacheDir, appDataDir, join } from "@tauri-apps/api/path";

const extractRoots: Record<string, string> = {
    "RobloxApp.zip": "",
    "redist.zip": "",
    "shaders.zip": "shaders/",
    "ssl.zip": "ssl/",

    "WebView2.zip": "",
    "WebView2RuntimeInstaller.zip": "WebView2RuntimeInstaller/",

    "content-avatar.zip": "content/avatar/",
    "content-configs.zip": "content/configs/",
    "content-fonts.zip": "content/fonts/",
    "content-sky.zip": "content/sky/",
    "content-sounds.zip": "content/sounds/",
    "content-textures2.zip": "content/textures/",
    "content-models.zip": "content/models/",

    "content-platform-fonts.zip": "PlatformContent/pc/fonts/",
    "content-platform-dictionaries.zip": "PlatformContent/pc/shared_compression_dictionaries/",
    "content-terrain.zip": "PlatformContent/pc/terrain/",
    "content-textures3.zip": "PlatformContent/pc/textures/",

    "extracontent-luapackages.zip": "ExtraContent/LuaPackages/",
    "extracontent-translations.zip": "ExtraContent/translations/",
    "extracontent-models.zip": "ExtraContent/models/",
    "extracontent-textures.zip": "ExtraContent/textures/",
    "extracontent-places.zip": "ExtraContent/places/"
};

export type ProgressEvent =
  | { type: "status"; message: string }
  | { type: "download"; file: string; done: number; total: number }
  | { type: "extract"; file: string; done: number; total: number };

type ProgressCallback = (event: ProgressEvent) => void;


async function ensureDir(path: string) {
    const existsDir = await exists(path);

    if (!existsDir) {
        await mkdir(path, { recursive: true }); 
    }
}

async function downloadAssets(
    assetsUrls: string[],
    onProgress: ProgressCallback,
    limit = 4
) {
    const queue = [...new Set(assetsUrls)];
    const total = queue.length;
    let done = 0;

    const workers = Array.from({ length: limit }, async () => {
        while (queue.length > 0) {
            const assetUrl = queue.shift();
            if (!assetUrl) return;
            const match = assetUrl.match(/version-[^-]+-(.+)$/);
            let fileName = match?.[1] ?? `file-${Date.now()}`;
            if (!fileName.endsWith(".zip")) fileName += ".zip";
            fileName = fileName.toLowerCase();

            try {
                const res = await fetch(assetUrl);
                if (!res.ok) throw new Error(`HTTP ${res.status}`);
                const buffer = await res.arrayBuffer();
                await writeFile(fileName, new Uint8Array(buffer), {
                    baseDir: BaseDirectory.AppCache
                });
                done++;
                onProgress({ type: "download", file: fileName, done, total });
            } catch (err) {
                info(`Failed (${fileName}): ${err}`);
            }
        }
    });

    await Promise.all(workers);
}

async function extractAll(
    hash_version: string,
    onProgress: ProgressCallback
) {
    const cacheDir = await appCacheDir();
    const dataDir = await appDataDir();
    const installRoot = await join(dataDir, "Player", "Versions", hash_version);
    await ensureDir(installRoot);

    const extractRootsLower = Object.fromEntries(
        Object.entries(extractRoots).map(([k, v]) => [k.toLowerCase(), v])
    );

    const entries = Object.entries(extractRootsLower);
    const total = entries.length;
    let done = 0;

    for (const [zipName, dest] of entries) {
        const zipPath = await join(cacheDir, zipName);
        const destPath = dest ? await join(installRoot, dest) : installRoot;

        try {
            const existsZip = await exists(zipName, { baseDir: BaseDirectory.AppCache });
            if (!existsZip) {
                done++;
                onProgress({ type: "extract", file: zipName, done, total });
                continue;
            }
            await ensureDir(destPath);
            await invoke("extract_zip", { zipPath, dest: destPath });
            done++;
            onProgress({ type: "extract", file: zipName, done, total });
        } catch (err) {
            info(`Failed (${zipName}): ${err}`);
            done++;
            onProgress({ type: "extract", file: zipName, done, total });
        }
    }
}

export async function downloadRoblox(onProgress: ProgressCallback) {
    onProgress({ type: "status", message: "Preparing download..." });

    const conf = await load("config.json");
    let bestRegion = await conf.get<string>("bestRegion");
    if (!bestRegion) {
        onProgress({ type: "status", message: "Finding best region..." });
        bestRegion = await invoke<string>("get_best_region");
        await conf.set("bestRegion", bestRegion);
        await conf.save();
    }

    onProgress({ type: "status", message: "Fetching asset URLs..." });
    const assetsUrls: string[] = await invoke("get_download_deployment_urls", { region: bestRegion });

    onProgress({ type: "status", message: "Downloading assets..." });
    await downloadAssets(assetsUrls, onProgress);

    onProgress({ type: "status", message: "Extracting files..." });
    const match = assetsUrls[1].match(/(version-[^-]+)-/);
    const version_hash = match?.[1] ?? "unknownversion";
    await extractAll(version_hash, onProgress);

    onProgress({ type: "status", message: "Writing AppSettings.xml..." });
    const dataDir = await appDataDir();
    const xmlPath = await join(dataDir, "Versions", version_hash, "AppSettings.xml");
    const xml = `<?xml version="1.0" encoding="UTF-8"?>
<Settings>
\t<ContentFolder>content</ContentFolder>
\t<BaseUrl>http://www.roblox.com</BaseUrl>
</Settings>`;
    await writeFile(xmlPath, new TextEncoder().encode(xml));

    onProgress({ type: "status", message: "Saving version info..." });
    const versionStore = await Store.load("versions.json");
    const versionList = (await versionStore.get<string[]>("versions")) ?? [];
    versionList.push(version_hash);
    await versionStore.set("versions", versionList);

    onProgress({ type: "status", message: "Installation complete!" });
}