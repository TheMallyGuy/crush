<script lang="ts">
    import { _ } from "svelte-i18n";
    import Button from "$lib/components/atoms/Button.svelte";
    import { goto } from "$app/navigation";
    import SettingCard from "$lib/components/molecules/SettingCard.svelte"
    import Switch from "$lib/components/atoms/Switch.svelte"
    import { load } from "@tauri-apps/plugin-store";
    import { onMount } from "svelte"
    import type { Integrations, DiscordRpc } from "$lib/types";

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

    $: isDisabled = !joinServerForYouValue
</script>

<div class="flex flex-col gap-8">
    <div class="flex items-center justify-between">
        <div>
            <img src={rovalraTitle} class="w-70 h-auto" />
            <p class="text-stone-400 mt-1">
                {$_("pages.integrations.roValra.description")}
            </p>
        </div>
                <div class="flex items-center gap-2">
            <Button variant="secondary" onclick={() => goto('../integrations')}>
                {$_('pages.integrations.gameHistory.backToIntegrations')}
            </Button>
        </div>
    </div>

    <SettingCard
        title={$_("pages.integrations.roValra.joinServerForYouCard.title")}
        description={$_("pages.integrations.roValra.joinServerForYouCard.description")}
    >
        <Switch bind:checked={joinServerForYouValue} slot="action" on:change={handleChanges}/>

    </SettingCard>
</div>