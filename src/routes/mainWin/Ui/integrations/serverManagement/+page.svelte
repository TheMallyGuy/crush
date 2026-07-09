<script lang="ts">
    import LoadingOverlay from '$lib/components/atoms/LoadingOverlay.svelte'
    import Switch from '$lib/components/atoms/Switch.svelte'
    import Button from '$lib/components/atoms/Button.svelte'
    import Dropdown from '$lib/components/molecules/Dropdown.svelte'
    import SettingCard from '$lib/components/molecules/SettingCard.svelte'
    import { Earth, MessageSquareCheck, Upload } from '@lucide/svelte'
    import { serverService } from './page.svelte'
    import { onMount } from 'svelte'

    onMount(() => {
        serverService.init()
    })

    function avatarUrl(user: { id: string; avatar: string | null } | null) {
        if (!user?.avatar) return null
        return `https://cdn.discordapp.com/avatars/${user.id}/${user.avatar}.png?size=64`
    }
</script>

<LoadingOverlay
    blur={true}
    message="Check your broswer!"
    visible={serverService.inAuth}
/>

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
            <Switch />
        {/snippet}
    </SettingCard>

    <SettingCard
        title="Perfered Region"
        description="this is where this feature is in the room"
        icon={Earth}
    >
        {#snippet action()}
            <Dropdown></Dropdown>
        {/snippet}
    </SettingCard>

    <SettingCard
        title="Improve match making for everyone [Login required]"
        description="Help Validate server information to improve match making. Reward is a Discord role"
        icon={MessageSquareCheck}
    >
        {#snippet action()}
            {#if serverService.isLoggedIn}
                <div class="flex items-center gap-3">
                    <Switch />
                    <Button
                        variant="ghost"
                        size="sm"
                        onclick={() => serverService.logout()}
                    >
                        Log out
                    </Button>
                </div>
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

        {#snippet footer()}
            {#if serverService.isLoggedIn && serverService.user}
                <div class="flex items-center gap-2 text-sm text-stone-400">
                    {#if avatarUrl(serverService.user)}
                        <img
                            src={avatarUrl(serverService.user)}
                            alt=""
                            class="w-5 h-5 rounded-full"
                        />
                    {/if}
                    <span
                        >Logged in as <b class="text-stone-200"
                            >{serverService.user.username}</b
                        ></span
                    >
                    <span class="text-stone-600">·</span>
                    <span>{serverService.validatedCount} validated</span>
                </div>
            {:else if serverService.authError}
                <p class="text-sm text-red-400">{serverService.authError}</p>
            {/if}
        {/snippet}
    </SettingCard>
</div>
