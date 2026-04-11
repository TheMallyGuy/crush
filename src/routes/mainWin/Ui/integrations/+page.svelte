<script lang="ts">
    import SettingCard from '$lib/components/molecules/SettingCard.svelte'
    import Button from '$lib/components/atoms/Button.svelte';
    import Switch from '$lib/components/atoms/Switch.svelte'
    import { Bell, Plug, History } from '@lucide/svelte';
    import { invoke } from '@tauri-apps/api/core'
    import { onMount } from 'svelte'
    import { load } from '@tauri-apps/plugin-store'
    import { type Integrations } from '$lib/types';
    import { _ } from 'svelte-i18n';
    import { goto } from '$app/navigation'

    let crushRpc = false
    let serverLocationNotifier = false
    const roValaraLogoColored = "/RovalraColored.svg"
    const roValaraLogo = "/Rovalra.svg"


    async function loadConfig() {
        const store = await load('config.json')
        let savedIntegrations = await store.get<Integrations>('integrations')

        if (!savedIntegrations) {
            savedIntegrations = await store.get<Integrations>('intergrations')
        }

        if (savedIntegrations) {
            crushRpc = savedIntegrations.crushRpc
            serverLocationNotifier = savedIntegrations.serverLocationNotifier
        }
    }

    onMount(async () => {
        await loadConfig()

        await invoke('set_rpc', {
            details: $_('rpc.general'),
            stateText: $_('rpc.integrations'),
        })
    })

    async function handleChanges() {
        const store = await load('config.json')

        const newIntegrations: Integrations = {
            crushRpc,
            serverLocationNotifier,
        }

        await store.set('integrations', newIntegrations)

        await store.save()
    }
</script>

<div class="flex flex-col gap-8">
    <div class="flex items-center justify-between">
        <div>
            <h1 class="text-3xl font-bold tracking-tight text-stone-100">
                {$_('pages.integrations.integrations')}
            </h1>
            <p class="text-stone-400 mt-1">
                {$_('pages.integrations.description')}
            </p>
        </div>
    </div>

    <div class="flex flex-col gap-3">
        <SettingCard
            title={$_('pages.integrations.serverNotifierCard.title')}
            description={$_('pages.integrations.serverNotifierCard.description')}
            icon={Bell}
        >
            <Switch
                slot="action"
                bind:checked={serverLocationNotifier}
                on:change={handleChanges}
            />
        </SettingCard>
        <SettingCard
            title={$_('pages.integrations.rpcCard.title')}
            description={$_('pages.integrations.rpcCard.description')}
            icon={Plug}
        >
            <Switch
                slot="action"
                bind:checked={crushRpc}
                on:change={handleChanges}
            />
        </SettingCard>

        <SettingCard
            title={$_('pages.integrations.gameHistoryCard.title')}
            description={$_('pages.integrations.gameHistoryCard.description')}
            icon={History}
        >
            <Button
                slot="action"
                variant="secondary"
                on:click={() => {
                    goto('integrations/gameHistory')
                }}
            >
                {$_('pages.integrations.gameHistoryCard.button')}
            </Button>
        </SettingCard>
        
        <SettingCard
            title="Configure RoValra"
            description="Crush have a built-in RoValra intergration. RoValra's Privacy Policy applies."

            icon={roValaraLogo}
            iconHover={roValaraLogoColored}
        >
        <Button slot="action" variant="secondary" on:click={() => {goto("integrations/roValra")}}> 
            Open
        </Button>
        </SettingCard>
    </div>
</div>
