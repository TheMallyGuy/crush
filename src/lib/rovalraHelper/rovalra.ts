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

async function measureLatency(ip: string): Promise<number> {
  const controller = new AbortController();
  const timeoutId = setTimeout(() => controller.abort(), 1500);

  const start = performance.now();
  try {
    await fetch(`http://${ip}`, {
      method: "HEAD",
      signal: controller.signal,
    });
  } catch {
    // connection refused / error still gives us a timing
  } finally {
    clearTimeout(timeoutId);
  }
  return performance.now() - start;
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

  collected = collected.slice(0, 10);

  // Ping all collected servers in parallel
  const pinged = await Promise.all(
    collected.map(async (server) => ({
      server,
      latency: await measureLatency(server.ip_address),
    }))
  );

  // Sort by lowest latency
  pinged.sort((a, b) => a.latency - b.latency);

  return {
    success: collected.length > 0,
    regionTried: regions,
    servers: pinged.map((p) => p.server),
  };
}
