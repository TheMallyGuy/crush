import type { RobloxLaunchData } from "./types";

export function parseRobloxDeepLink(url: string): RobloxLaunchData {
    const cleaned = url.replace(/^roblox-player:\d+\+?/, "");

    const raw: Record<string, string> = {};

    for (const part of cleaned.split("+")) {
        const [key, ...rest] = part.split(":");
        if (!key || rest.length === 0) continue;

        const value = rest.join(":");
        raw[key] = decodeURIComponent(value);
    }

    // base result
    const result: RobloxLaunchData = {
        raw,
        launchmode: raw.launchmode,
        gameinfo: raw.gameinfo,
        launchtime: raw.launchtime ? Number(raw.launchtime) : undefined,
        placelauncherurl: raw.placelauncherurl
    };

    if (raw.placelauncherurl) {
        try {
            const urlObj = new URL(raw.placelauncherurl);

            result.request = urlObj.searchParams.get("request") ?? undefined;

            const placeId = urlObj.searchParams.get("placeId");
            const userId = urlObj.searchParams.get("userId");

            result.placeId = placeId ? Number(placeId) : null;
            result.userId = userId ? Number(userId) : null;
            result.joinAttemptId =
                urlObj.searchParams.get("joinAttemptId") ?? null;
        } catch {
            // ignore invalid URL
        }
    }

    return result;
}

export function rebuildDeeplink(
    parsed: RobloxLaunchData,
    placeId: number,
    jobId: string
): string {
    if (!parsed.placelauncherurl) {
        throw new Error("Missing placelauncherurl in RobloxLaunchData");
    }

    const url = new URL(parsed.placelauncherurl);

    url.searchParams.delete("userId");

    url.searchParams.set("request", "RequestGameJob");
    url.searchParams.set("placeId", String(placeId));
    url.searchParams.set("gameId", jobId);

    return [
        "roblox-player:1",
        `launchmode:${parsed.launchmode ?? "play"}`,
        `gameinfo:${parsed.gameinfo ?? ""}`,
        `launchtime:${Date.now()}`,
        `placelauncherurl:${encodeURIComponent(url.toString())}`,

        // keep original extras
        parsed.raw.browsertrackerid
            ? `browsertrackerid:${parsed.raw.browsertrackerid}`
            : null,
        parsed.raw.robloxLocale
            ? `robloxLocale:${parsed.raw.robloxLocale}`
            : null,
        parsed.raw.gameLocale
            ? `gameLocale:${parsed.raw.gameLocale}`
            : null,
        parsed.raw.channel
            ? `channel:${parsed.raw.channel}`
            : null,
        parsed.raw.LaunchExp
            ? `LaunchExp:${parsed.raw.LaunchExp}`
            : null
    ]
        .filter(Boolean)
        .join("+");
}
