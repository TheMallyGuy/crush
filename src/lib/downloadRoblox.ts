import { fetch } from '@tauri-apps/plugin-http';
import { invoke } from '@tauri-apps/api/core';
import { load } from '@tauri-apps/plugin-store';
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

async function ensureDir(path: string) {
    const existsDir = await exists(path);

    if (!existsDir) {
        await mkdir(path, { recursive: true }); 
    }
}

async function extractAll() {
    const cacheDir = await appCacheDir();
    const dataDir = await appDataDir();
    const installRoot = await join(dataDir, "Roblox");
    await ensureDir(installRoot);

    const extractRootsLower = Object.fromEntries(
        Object.entries(extractRoots).map(([k, v]) => [k.toLowerCase(), v])
    );

    for (const [zipName, dest] of Object.entries(extractRootsLower)) {
        const zipPath = await join(cacheDir, zipName);
        const destPath = dest ? await join(installRoot, dest) : installRoot;

        try {
            const existsZip = await exists(zipName, {
                baseDir: BaseDirectory.AppCache
            });

            if (!existsZip) {
                info(`Skipping ${zipName} (missing)`);
                continue;
            }

            await ensureDir(destPath);

            info(`Extracting ${zipName} → ${destPath}`);

            await invoke("extract_zip", {
                zipPath,
                dest: destPath
            });

            info(`Done (${zipName})`);
        } catch (err) {
            info(`Failed (${zipName}): ${err}`);
        }
    }
}

async function downloadAssets(assetsUrls: string[], limit = 4) {
    const queue = [...new Set(assetsUrls)];

    const workers = Array.from({ length: limit }, async () => {
        while (queue.length > 0) {
            const assetUrl = queue.shift();
            if (!assetUrl) return;

            const match = assetUrl.match(/version-[^-]+-(.+)$/);
            let fileName = match?.[1] ?? `file-${Date.now()}`;

            if (!fileName.endsWith(".zip")) {
                fileName += ".zip";
            }

            fileName = fileName.toLowerCase();

            try {
                info(`Fetching ${fileName}`);

                const res = await fetch(assetUrl);
                if (!res.ok) {
                    throw new Error(`HTTP ${res.status}`);
                }

                const buffer = await res.arrayBuffer();
                const bytes = new Uint8Array(buffer);

                await writeFile(fileName, bytes, {
                    baseDir: BaseDirectory.AppCache
                });

                info(`Completed! (${fileName})`);
            } catch (err) {
                info(`Failed (${fileName}): ${err}`);
            }
        }
    });

    await Promise.all(workers);
}

export async function downloadRoblox() {
    info("Preparing for downloading Roblox");

    const conf = await load("config.json");

    let bestRegion = await conf.get<string>("bestRegion");

    if (!bestRegion) {
        info("Best region not found, getting best region");

        bestRegion = await invoke<string>("get_best_region");

        await conf.set("bestRegion", bestRegion);
        await conf.save();
    }

    info("Getting download assets urls");

    let assetsUrls: string[] = await invoke("get_download_deployment_urls", {
        region: bestRegion
    });

    info("Downloading assets...");
    await downloadAssets(assetsUrls);

    info("Begins extracting...");
    await extractAll();
}