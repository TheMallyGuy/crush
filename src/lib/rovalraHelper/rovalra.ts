import { fetch } from "@tauri-apps/plugin-http";
import type { Server, Result } from "$lib/types";

const regionMap: Record<string, string[]> = {
  VN: ["SG", "JP"],
  TH: ["SG", "JP"],
  MY: ["SG", "JP"],
  ID: ["SG", "JP"],
  PH: ["SG", "JP"],
  JP: ["JP", "SG"],
  KR: ["JP", "SG"],
  DE: ["GR", "NL"],
  FR: ["NL", "GR"],
  GB: ["NL", "GR"],
  IT: ["NL", "GR"],
  ES: ["NL", "GR"],
  NL: ["NL", "GR"],
  US: ["US"],
  CA: ["US"],
  MX: ["US"],
  BR: ["US"],
  DEFAULT: ["US"]
};

let cachedCountry: string | null = null;

async function measureJoinLatency(placeId: number, serverId: string) {
  const start = performance.now();
  const controller = new AbortController();
  const timeoutId = setTimeout(() => controller.abort(), 1500);

  try {
    const res = await fetch(
      "https://gamejoin.roblox.com/v1/join-game-instance",
      {
        method: "POST",
        headers: {
          "Content-Type": "application/json"
        },
        body: JSON.stringify({
          placeId,
          gameId: serverId,
          gameJoinAttemptId: crypto.randomUUID()
        }),
        signal: controller.signal
      }
    );

    clearTimeout(timeoutId);

    if (!res.ok) return Infinity;

    const data = await res.json();
    if (!data.joinScript) return Infinity;

    return performance.now() - start;
  } catch {
    return Infinity;
  }
}

export async function getBestServers(placeId: number): Promise<Result> {
  if (!cachedCountry) {
    const geoRes = await fetch("http://ip-api.com/json");
    const geo = await geoRes.json();
    cachedCountry = geo.countryCode || "US";
  }

  const country = cachedCountry!;
  const regions = regionMap[country] || regionMap.DEFAULT;

  let collected: Server[] = [];

  for (const region of regions) {
    try {
      const res = await fetch(
        `https://apis.rovalra.com/v1/servers/region?place_id=${placeId}&country=${region}&cursor=0`
      );
      if (!res.ok) continue;

      const data = await res.json();

      if (data.servers && data.servers.length > 0) {
        collected.push(...data.servers);
        if (collected.length >= 10) break;
      }
    } catch {
      continue;
    }
  }

  collected.sort((a, b) => a.playing - b.playing);

  const candidates = collected.slice(0, 3);

  const pinged = await Promise.all(
    candidates.map(async (server) => ({
      server,
      latency: await measureJoinLatency(placeId, server.server_id),
    }))
  );

  pinged.sort((a, b) => a.latency - b.latency);

  return {
    success: pinged.length > 0,
    regionTried: regions,
    servers: pinged.map((p) => p.server),
  };
}