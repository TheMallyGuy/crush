import { appDataDir, join } from "@tauri-apps/api/path";
import { mkdir, exists, writeTextFile, readTextFile } from "@tauri-apps/plugin-fs";

async function ensureDir(path: string) {
    if (!(await exists(path))) {
        await mkdir(path, { recursive: true });
    }
}

function coerceValue(value: string): boolean | number | string {
    if (value === 'true') return true;
    if (value === 'false') return false;
    if (/^-?\d+$/.test(value)) return parseInt(value, 10);
    return value;
}

function decoerceValue(value: unknown): string {
    return String(value);
}

export async function getFastFlags(version_hash: string): Promise<Record<string, string>> {
    const baseDir = await appDataDir();
    const clientSettingFolder = await join(baseDir, "Player", "Versions", version_hash, "ClientSettings");
    const clientSettingFile = await join(clientSettingFolder, "ClientAppSettings.json");
    await ensureDir(clientSettingFolder);

    if (!(await exists(clientSettingFile))) return {};

    const raw = await readTextFile(clientSettingFile);
    const parsed: Record<string, unknown> = JSON.parse(raw);

    return Object.fromEntries(
        Object.entries(parsed).map(([k, v]) => [k, decoerceValue(v)])
    );
}

export async function saveFastFlags(version_hash: string, flags: Record<string, string>): Promise<void> {
    const baseDir = await appDataDir();
    const clientSettingFolder = await join(baseDir, "Player", "Versions", version_hash, "ClientSettings");
    const clientSettingFile = await join(clientSettingFolder, "ClientAppSettings.json");
    await ensureDir(clientSettingFolder);

    const coerced = Object.fromEntries(
        Object.entries(flags).map(([k, v]) => [k, coerceValue(v)])
    );

    await writeTextFile(clientSettingFile, JSON.stringify(coerced, null, 2));
}