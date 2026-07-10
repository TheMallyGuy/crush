<script lang="ts">
    import LoadingOverlay from '$lib/components/atoms/LoadingOverlay.svelte'
    import Switch from '$lib/components/atoms/Switch.svelte'
    import Button from '$lib/components/atoms/Button.svelte'
    import Dropdown from '$lib/components/molecules/Dropdown.svelte'
    import SettingCard from '$lib/components/molecules/SettingCard.svelte'
    import DiscordUserInfo from '$lib/components/molecules/DiscordUserInfo.svelte'
    import {
        Earth,
        MessageSquareCheck,
        MessageSquareHeart,
        Upload,
    } from '@lucide/svelte'
    import { serverService } from '$lib/stores/discordAuth.svelte'
    import { serverManagementSettings } from '$lib/stores/serverManagement.svelte'
    import { fetchRegions } from '$lib/serverSelector'
    import { onMount } from 'svelte'
    import Dialog from '$lib/components/molecules/Dialog.svelte'
    import Textbox from '$lib/components/atoms/Textbox.svelte'

    let openSub = $state(false)
    let serverAmount: number = $state(0)

    let regionOptions = $state<{ value: string; label: string }[]>([
        { value: '', label: 'Any region' },
    ])

    onMount(async () => {
        serverService.init()
        if (!serverManagementSettings.loaded) {
            await serverManagementSettings.init()
        }

        try {
            const regions = await fetchRegions()
            regionOptions = [
                { value: '', label: 'Any region' },
                ...regions.map((region) => ({ value: region, label: region })),
            ]
        } catch (e) {
            console.error('[serverManagement] failed to fetch regions', e)
        }
    })

    $effect(() => {
        if (!serverManagementSettings.loaded) return

        void serverManagementSettings.autoSubmit
        void serverManagementSettings.preferredRegion
        void serverManagementSettings.autoValidate
        void serverService.isLoggedIn

        void serverManagementSettings.save()
        serverManagementSettings.syncAutoValidateTimer()
    })
</script>

<LoadingOverlay
    blur={true}
    message="Check your broswer!"
    visible={serverService.inAuth}
/>

<Dialog
    title="Submission servers"
    description="Be the one whos ranking #1 on the leaderboard (Discord). Note that this action use your cookie, but the cookie never get send to the server."
    onclose={() => {
        openSub = !openSub
    }}
    bind:open={openSub}
>
    <Textbox
        type="number"
        label="How many servers? (100 is ideal)"
        bind:value={serverAmount}
        placeholder="100"
    />

    {#snippet actions()}
        <Button>Begin</Button>
    {/snippet}
</Dialog>

<div class="flex flex-col gap-8">
    <div class="flex items-center justify-between">
        <div>
            <h1 class="text-3xl font-bold tracking-tight text-stone-100">
                Server Stuff
            </h1>
        </div>
    </div>

    <SettingCard
        title="Submission your server data to mr mally"
        description="Submit your server data in order for this feature to work. We collect server data for match making"
        icon={Upload}
    >
        {#snippet action()}
            <Switch bind:checked={serverManagementSettings.autoSubmit} />
        {/snippet}
    </SettingCard>

    <SettingCard
        title="Perfered Region"
        description="this is where this feature is in the room"
        icon={Earth}
    >
        {#snippet action()}
            <Dropdown
                bind:value={serverManagementSettings.preferredRegion}
                options={regionOptions}
                placeholder="Any region"
            />
        {/snippet}
    </SettingCard>

    <SettingCard title="Login to Validate" description="">
        {#snippet action()}
            {#if serverService.isLoggedIn}
                <Button
                    variant="ghost"
                    size="sm"
                    onclick={() => serverService.logout()}
                >
                    Log out
                </Button>
            {:else}
                <Button
                    variant="secondary"
                    size="sm"
                    disabled={serverService.inAuth}
                    onclick={() => serverService.auth()}
                >
                    Login with Discord
                </Button>
            {/if}
        {/snippet}
    </SettingCard>

    <SettingCard
        title="Validate servers"
        description="Help Validate server information to improve match making. Reward is a Discord role"
        icon={MessageSquareCheck}
    >
        {#snippet action()}
            <Switch
                bind:checked={serverManagementSettings.autoValidate}
                disabled={!serverService.isLoggedIn}
            />
        {/snippet}
    </SettingCard>

    <SettingCard
        title="Voluntarily submission Servers"
        description="holy goat thanks"
        icon={MessageSquareHeart}
    >
        {#snippet action()}
            <Button
                variant="secondary"
                disabled={!serverService.isLoggedIn}
                onclick={() => {
                    openSub = !openSub
                }}>Send some data</Button
            >
        {/snippet}
    </SettingCard>

    <DiscordUserInfo class="-mt-2 ml-3" />
</div>
