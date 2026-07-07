<script>
    import Button from '$lib/components/atoms/Button.svelte'
    import Switch from '$lib/components/atoms/Switch.svelte'
    import Dropdown from '$lib/components/molecules/Dropdown.svelte'
    import SettingCard from '$lib/components/molecules/SettingCard.svelte'
    import { serverDataHandler } from '$lib/serverSelector'
    import { Earth, MessageSquareCheck, Upload } from '@lucide/svelte'
</script>

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
        title="Improve match making for everyone"
        description="Help Validate server information to improve match making. Reward is a Discord role"
        icon={MessageSquareCheck}
    >
        {#snippet action()}
            <Switch />
        {/snippet}
    </SettingCard>
</div>
<Button onlick={serverDataHandler}></Button>