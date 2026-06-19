<script lang="ts">
    import Button from '$lib/components/atoms/Button.svelte'
    import ExpandableSettingCard from '$lib/components/molecules/ExpandableSettingCard.svelte'
    import type { ServerInfoFromBackend } from '$lib/types'
    import { listen } from '@tauri-apps/api/event'
    import { writeText } from '@tauri-apps/plugin-clipboard-manager'
    import { fetch } from '@tauri-apps/plugin-http'
    import { _ } from 'svelte-i18n'
    import { invoke } from '@tauri-apps/api/core'
    import { onMount } from 'svelte'

    let serverInstanceId: string = 'Unknown'
    let gameId: number = 1234
    let gameRegion: string = 'Unknown'
    let serverInviteLink: string
    let gameName: string = 'Unknown game'

    async function getUniverse(
        placeId: number
    ): Promise<{ universeId: number } | null> {
        return await fetch(
            `https://apis.roblox.com/universes/v1/places/${placeId}/universe`
        )
            .then((r) => r.json())
            .catch(() => null)
    }

    async function getGameDetails(
        placeId: number,
        universeId: number
    ): Promise<{ name: string; imageUrl: string | null }> {
        const [nameRes, iconRes] = await Promise.all([
            fetch(`https://games.roblox.com/v1/games?universeIds=${universeId}`)
                .then((r) => r.json())
                .catch(() => null),
            fetch(
                `https://thumbnails.roblox.com/v1/games/icons?universeIds=${universeId}&returnPolicy=PlaceHolder&size=512x512&format=Png&isCircular=false`
            )
                .then((r) => r.json())
                .catch(() => null),
        ])

        return {
            name: nameRes?.data?.[0]?.name ?? 'Unknown Game',
            imageUrl: iconRes?.data?.[0]?.imageUrl ?? null,
        }
    }

    async function copyToClipboard(url: any) {
        await writeText(url)
    }

    async function applyServerInfo(info: ServerInfoFromBackend) {
        serverInstanceId = info.server_id
        gameId = info.game_id
        gameRegion = info.region_info
        serverInviteLink = `https://deeplink.multicrew.dev?placeId=${gameId}&jobId=${serverInstanceId}`

        const universeData = await getUniverse(gameId)
        if (!universeData) {
            gameName = 'Unknown Game'
            return
        }
        const details = await getGameDetails(gameId, universeData.universeId)
        gameName = details.name
    }

    onMount(async () => {
        const stored = await invoke<ServerInfoFromBackend | null>('get_server_info')
        if (stored) await applyServerInfo(stored)

        listen<ServerInfoFromBackend>('serverInfomation', async (event) => {
            await applyServerInfo(event.payload)
        })
    })
</script>

<div class="flex flex-col gap-4">
    <h1 class="text-3xl">Server Infomation</h1>

    <ExpandableSettingCard
        title={gameName}
        description={$_("pages.serverInfomation.infomationCard.description")}
        isOpen={true}
    >
        <div class="flex flex-col gap-3 p-4">
            <p>
                {$_('pages.serverInfomationPage.infomationCard.serverRegion', { values: { region: gameRegion || $_("pages.serverInfomationPage.infomationCard.waitRegion") } })}
            </p>
            <p>{$_('pages.serverInfomationPage.infomationCard.uptime')}</p>
            <p> 
                {$_('pages.serverInfomationPage.infomationCard.instanceId', {
                    values: { id: serverInstanceId },
                })}
            </p>
            <p class="flex items-center gap-2">
                {$_('pages.serverInfomationPage.infomationCard.inviteLink')}d <Button
                    variant="secondary"
                    onclick={async () => {
                        await copyToClipboard(serverInviteLink)
                    }}>Copy to clipboard</Button
                >
            </p>
        </div>
    </ExpandableSettingCard>
</div>
