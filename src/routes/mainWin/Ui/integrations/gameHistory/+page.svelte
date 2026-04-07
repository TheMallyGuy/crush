<script lang="ts">
    import { _ } from "svelte-i18n";
    import ImageCard from "$lib/components/molecules/ImageCard.svelte";
    import Button from "$lib/components/atoms/Button.svelte";
    import { Link, Play } from "@lucide/svelte";
    import { load } from "@tauri-apps/plugin-store";
    import { onMount } from "svelte";
    import { fetch } from "@tauri-apps/plugin-http";
    import { page } from "$app/state"
    import { goto } from "$app/navigation"
 
    let isLoading = true;
    let gameHistory: {
        placeId: number;
        instanceId: string | undefined;
        title: string;
        imageUrl: string | null;
        deeplink: string;
        timestamp: Date;
    }[] = [];
 
    type RawEntry = {
        place_id: number;
        instance_id?: string;
        timestamp: string;
    };
 
    type GameCache = {
        universeId: number | null;
        name: string;
        imageUrl: string | null;
        cachedAt: string;
    };
 
    const CACHE_TTL_MS = 1000 * 60 * 60 * 24; // 24 hours
 
    async function getUniverse(
        placeId: number,
        store: Awaited<ReturnType<typeof load>>,
    ): Promise<{ universeId: number } | null> {
        const cacheKey = `gameCache:${placeId}`;
        const cached = await store.get<GameCache>(cacheKey);
 
        if (cached && Date.now() - new Date(cached.cachedAt).getTime() < CACHE_TTL_MS) {
            return cached.universeId !== null ? { universeId: cached.universeId } : null;
        }
 
        return await fetch(`https://apis.roblox.com/universes/v1/places/${placeId}/universe`)
            .then(r => r.json())
            .catch(() => null);
    }
 
    async function getGameDetails(
        placeId: number,
        universeId: number,
        store: Awaited<ReturnType<typeof load>>,
    ): Promise<{ name: string; imageUrl: string | null }> {
        const cacheKey = `gameCache:${placeId}`;
        const cached = await store.get<GameCache>(cacheKey);
 
        if (cached && Date.now() - new Date(cached.cachedAt).getTime() < CACHE_TTL_MS) {
            return { name: cached.name, imageUrl: cached.imageUrl };
        }
 
        const [nameRes, iconRes] = await Promise.all([
            fetch(`https://games.roblox.com/v1/games?universeIds=${universeId}`)
                .then(r => r.json())
                .catch(() => null),
            fetch(`https://thumbnails.roblox.com/v1/games/icons?universeIds=${universeId}&returnPolicy=PlaceHolder&size=512x512&format=Png&isCircular=false`)
                .then(r => r.json())
                .catch(() => null),
        ]);
 
        const details = {
            name: nameRes?.data?.[0]?.name ?? "Unknown Game",
            imageUrl: iconRes?.data?.[0]?.imageUrl ?? null,
        };
 
        await store.set(cacheKey, {
            universeId,
            name: details.name,
            imageUrl: details.imageUrl,
            cachedAt: new Date().toISOString(),
        } satisfies GameCache);
        await store.save();
 
        return details;
    }

    async function handleClearHistory() {
        const store = await load("config.json");
        await store.set("gameHistory", []);
        await store.save();

        location.reload()
    }
 
    onMount(async () => {
        const store = await load("config.json");
        const rawData = await store.get<RawEntry[]>("gameHistory");
 
        const seen = new Map<number, RawEntry>();
        for (const entry of rawData ?? []) {
            const existing = seen.get(entry.place_id);
            if (!existing || entry.timestamp > existing.timestamp) {
                seen.set(entry.place_id, entry);
            }
        }
 
        const resolved = await Promise.all(
            [...seen.values()].map(async (entry) => {
                const universeData = await getUniverse(entry.place_id, store);
                const details = universeData
                    ? await getGameDetails(entry.place_id, universeData.universeId, store)
                    : { name: $_('pages.integrations.gameHistory.gameHistoryCard.titleUnknown'), imageUrl: null };
 
                return {
                    placeId: entry.place_id,
                    instanceId: entry.instance_id,
                    title: details.name,
                    imageUrl: details.imageUrl,
                    deeplink: `https://deeplink.multicrew.dev?placeId=${entry.place_id}&jobId=${entry.instance_id ?? ""}`,
                    timestamp: new Date(entry.timestamp),
                };
            })
        );
 
        gameHistory = resolved.sort((a, b) => b.timestamp.getTime() - a.timestamp.getTime());
        isLoading = false;
    });
</script>


<div class="flex flex-col gap-8 max-w-2xl">
    <div class="flex items-center justify-between">
        <div>
            <h1 class="text-3xl font-bold tracking-tight text-stone-100">
                {$_('pages.integrations.gameHistory.title')}
            </h1>
            <p class="text-stone-400 mt-1">
                {$_('pages.integrations.gameHistory.description')}
            </p>
        </div>
        <div class="flex items-center gap-2">
            <Button variant="danger" onclick={handleClearHistory}>
                {$_('pages.integrations.gameHistory.clearHistory')}
            </Button>
            <Button variant="secondary" onclick={() => goto('../integrations')}>
                {$_('pages.integrations.gameHistory.backToIntegrations')}
            </Button>
        </div>
    </div>
    {#if isLoading}
        <p>{$_('pages.integrations.gameHistory.gameHistoryLoading')}</p>
    {:else}
        {#each gameHistory as game}
            <ImageCard
                title={game.title}
                description={$_('pages.integrations.gameHistory.gameHistoryCard.lastPlayed', { values : { time: game.timestamp.toLocaleString() }})}
                image={game.imageUrl ?? undefined}
            >
                <Button variant="primary">
                    <Play class="mr-2" size={16} />
                    {$_('pages.integrations.gameHistory.gameHistoryCard.play')}
                </Button>
                <Button variant="secondary" onclick={() => navigator.clipboard.writeText(game.deeplink)}>
                    <Link class="mr-2" size={16} />
                    {$_('pages.integrations.gameHistory.gameHistoryCard.deeplink')}
                </Button>
            </ImageCard>
        {/each}
    {/if}


</div>