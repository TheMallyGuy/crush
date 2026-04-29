<script lang="ts">
    import { _ } from "svelte-i18n";
    import Button from "$lib/components/atoms/Button.svelte";
    import { goto } from "$app/navigation";
    import SettingCard from "$lib/components/molecules/SettingCard.svelte"
    import Switch from "$lib/components/atoms/Switch.svelte"
    import { load } from "@tauri-apps/plugin-store";
    import { onMount } from "svelte"
    import type { Integrations, DiscordRpc } from "$lib/types";
    import Textbox from "$lib/components/atoms/Textbox.svelte"

    const rovalraTitle = "/RovalraTitle.svg"
    let joinServerForYouValue: boolean = false
    let isDisabled: boolean = true

    async function loadConfig() {
        const store = await load('config.json')
        let savedIntegrations = await store.get<Integrations>('integrations')
            ?? await store.get<Integrations>('intergrations')

        if (savedIntegrations) {
            joinServerForYouValue = savedIntegrations.roValra.joinServerForYouValue ?? false
        }
    }

    async function handleChanges() {
        const store = await load('config.json')

        const savedIntegrations = await store.get<Integrations>('integrations')
            ?? await store.get<Integrations>('intergrations')

        const savedRpc: DiscordRpc = savedIntegrations?.discordRpc ?? {
            enable: false,
            displayAccount: false,
            letJoin: false,
        }

        const newIntegrations: Integrations = {
            discordRpc: {
                enable: savedRpc.enable,
                displayAccount: savedRpc.displayAccount,
                letJoin: savedRpc.letJoin,
            },
            serverLocationNotifier: savedIntegrations?.serverLocationNotifier ?? false,
            roValra: {
                joinServerForYouValue,
            }
        }

        await store.set('integrations', newIntegrations)
        await store.save()
    }

    onMount(async () => {
        await loadConfig()
    })

</script>

<div class="flex flex-col gap-8">
    <div class="flex items-center justify-between">
        <div>
            <h1 class="text-3xl font-bold tracking-tight text-stone-100">
                Interactive API settings
            </h1>
            <p class="text-stone-400 mt-1">
                Configure how Interactive API should act.
            </p>
        </div>
                <div class="flex items-center gap-2">
            <Button variant="secondary" onclick={() => goto('../integrations')}>
                {$_('pages.integrations.gameHistory.backToIntegrations')}
            </Button>
        </div>
    </div>

    <SettingCard
        title="Enable"
        description="Choose whether or not to enable Interactive API."
    >
        <Switch bind:checked={joinServerForYouValue} slot="action" on:change={handleChanges}/>
    </SettingCard>

    <div class="flex flex-col gap-3">
        <div>
            <h1 class="text-3xl font-bold tracking-tight text-stone-100">
                Advanced
            </h1>
            <p class="text-stone-400 mt-1">
                Deeply configure Interactive API.
            </p>
        </div>

        <div class="flex flex-col gap-2">
            <h1 class="text-2xl font-bold tracking-tight text-stone-100">
                Scopes
            </h1>

            <SettingCard
                title="Minimize"
                description="Whenther or not enable mimimize control via Interactive API."
            >
                <Switch slot="action"/>
            </SettingCard>

            <SettingCard
                title="Maximize"
                description="Whenther or not enable maximize control via Interactive API."
            >
                <Switch slot="action"/>
            </SettingCard>

            <SettingCard
                title="Focus"
                description="Whenther or not enable focus control via Interactive API."
            >
                <Switch slot="action"/>
            </SettingCard>

            <SettingCard
                title="Move Window"
                description="Whenther or not enable move window control via Interactive API."
            >
                <Switch slot="action"/>
            </SettingCard>

            <SettingCard
                title="Restore"
                description="Whenther or not enable restore control via Interactive API."
            >
                <Switch slot="action"/>
            </SettingCard>

            <SettingCard
                title="Set window title"
                description="Whenther or not enable set window title control via Interactive API."
            >
                <Switch slot="action"/>
            </SettingCard>

            <SettingCard
                title="Set borderless"
                description="Whenther or not enable borderless of the window control via Interactive API."
            >
                <Switch slot="action"/>
            </SettingCard>
        </div>

        <div class="flex flex-col gap-2">
            <h1 class="text-2xl font-bold tracking-tight text-stone-100">
                Transparency Settings
            </h1>

            <SettingCard
                title="Enable"
                description="Whenther or not transparency control via Interactive API."
            >
                <Switch slot="action"/>
            </SettingCard>

            <SettingCard
                title="Min transparency"
                description="The minimum of how transparency of your window would get."
            >
                <Textbox slot="action"/>
            </SettingCard>

            <SettingCard
                title="Max transparency"
                description="The maximum of how transparency of your window would get."
            >
                <Textbox slot="action"/>
            </SettingCard>
        </div>
    </div>
</div>