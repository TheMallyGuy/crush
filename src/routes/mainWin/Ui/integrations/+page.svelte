<script lang="ts">
    import SettingCard from '$lib/components/molecules/SettingCard.svelte'
    import Switch from '$lib/components/atoms/Switch.svelte'
    import { Bell, Plug } from '@lucide/svelte';
    import { invoke } from '@tauri-apps/api/core'
    import { onMount } from 'svelte'
    import { load } from '@tauri-apps/plugin-store'
    import { type Intergrations } from '$lib/types';
    import { _ } from 'svelte-i18n';

    let crushRpc = false
    let serverLocationNotifier = false

    async function loadConfig() {
        const store = await load('config.json')
        const savedIntergrations =
            await store.get<Intergrations>('intergrations')

        if (savedIntergrations) {
            crushRpc = savedIntergrations.crushRpc
            serverLocationNotifier = savedIntergrations.serverLocationNotifier
        }
    }

    onMount(async () => {
        await loadConfig()

        await invoke('set_rpc', {
            details: 'A roblox bootrapper written from scratch',
            stateText: 'In Intergrations Route',
        })
    })

    async function handleChanges() {
        const store = await load('config.json')

        const newIntergrations: Intergrations = {
            crushRpc,
            serverLocationNotifier,
        }

        await store.set('intergrations', newIntergrations)

        await store.save()
    }
</script>

<div class="flex flex-col gap-8 max-w-2xl">
    <div class="flex items-center justify-between">
        <div>
            <h1 class="text-3xl font-bold tracking-tight text-stone-100">
                {$_('integrations.integrations')}
            </h1>
            <p class="text-stone-400 mt-1">
                {$_('integrations.description')}
            </p>
        </div>
    </div>

    <div class="flex flex-col gap-3">
        <SettingCard
            title={$_('integrations.serverNotifierCard.title')}
            description={$_('integrations.serverNotifierCard.description')}
            icon={Bell}
        >
            <Switch
                slot="action"
                bind:checked={serverLocationNotifier}
                on:change={handleChanges}
            />
        </SettingCard>
        <SettingCard
            title={$_('integrations.rpcCard.title')}
            description={$_('integrations.rpcCard.description')}
            icon={Plug}
        >
            <Switch
                slot="action"
                bind:checked={crushRpc}
                on:change={handleChanges}
            />
        </SettingCard>
    </div>
</div>
