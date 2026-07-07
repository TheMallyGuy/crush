<script lang="ts">
    import Button from '$lib/components/atoms/Button.svelte'
    import ExpandableSettingCard from '$lib/components/molecules/ExpandableSettingCard.svelte'
    import { serverInfo } from '$lib/stores/serverInfo.svelte'
    import { writeText } from '@tauri-apps/plugin-clipboard-manager'
    import { fetch } from '@tauri-apps/plugin-http'
    import { _ } from 'svelte-i18n'

    let gameName: string = $state('Unknown game')

    let serverInstanceId = $derived(serverInfo.serverId ?? 'Unknown')
    let gameId = $derived(serverInfo.gameId ?? 0)
    let gameRegion = $derived(serverInfo.regionInfo ?? '')
    let isPrivateServer = $derived(serverInfo.isPrivateServer)
    let accessCode = $derived(serverInfo.accessCode)
    let serverInviteLink = $derived(
        `https://deeplink.multicrew.dev?placeId=${gameId}&jobId=${serverInstanceId}`
    )

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

    $effect(() => {
        const placeId = gameId
        if (!placeId) return

        gameName = 'Unknown game'
        getUniverse(placeId).then(async (universeData) => {
            if (!universeData) {
                gameName = 'Unknown Game'
                return
            }
            const details = await getGameDetails(placeId, universeData.universeId)
            gameName = details.name
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
                {isPrivateServer
                    ? $_('pages.serverInfomationPage.infomationCard.serverType.private')
                    : $_('pages.serverInfomationPage.infomationCard.serverType.public')}
            </p>
            <p>
                {$_('pages.serverInfomationPage.infomationCard.serverRegion', { values: { region: gameRegion || $_("pages.serverInfomationPage.infomationCard.waitRegion") } })}
            </p>
            <p>{$_('pages.serverInfomationPage.infomationCard.uptime')}</p>
            <p>
                {$_('pages.serverInfomationPage.infomationCard.instanceId', {
                    values: { id: serverInstanceId },
                })}
            </p>
            {#if isPrivateServer}
                <p class="flex items-center gap-2">
                    {$_('pages.serverInfomationPage.infomationCard.inviteLink')} <Button
                        variant="secondary"
                        disabled={true}>Copy to clipboard</Button
                    >
                </p>
                <p class="text-sm opacity-70">
                    {$_('pages.serverInfomationPage.infomationCard.inviteUnavailable')}
                </p>
                {#if accessCode}
                    <p class="flex items-center gap-2">
                        {$_('pages.serverInfomationPage.infomationCard.accessCode')} <Button
                            variant="secondary"
                            onclick={async () => {
                                await copyToClipboard(accessCode)
                            }}>Copy to clipboard</Button
                        >
                    </p>
                {/if}
            {:else}
                <p class="flex items-center gap-2">
                    {$_('pages.serverInfomationPage.infomationCard.inviteLink')} <Button
                        variant="secondary"
                        onclick={async () => {
                            await copyToClipboard(serverInviteLink)
                        }}>Copy to clipboard</Button
                    >
                </p>
            {/if}
        </div>
    </ExpandableSettingCard>
</div>
