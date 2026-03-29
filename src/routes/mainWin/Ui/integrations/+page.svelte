<script lang="ts">
    import SettingCard from '$lib/ui/molecules/SettingCard.svelte'
    import Switch from '$lib/ui/atoms/Switch.svelte'
    import { invoke } from '@tauri-apps/api/core'
    import { onMount } from 'svelte'
    import { load } from '@tauri-apps/plugin-store'

    type Integrations = {
        crushRpc: boolean
        serverLocationNotifier: boolean
    }

    type Config = {
        FirstLaunch: string
        bestRegion: string
        integrations: Integrations
    }

    let crushRpc = false
    let serverLocationNotifier = false

    async function loadConfig() {
        const store = await load('config.json')
        const savedIntegrations =
            await store.get<Integrations>('integrations')

        if (savedIntegrations) {
            crushRpc = savedIntegrations.crushRpc
            serverLocationNotifier = savedIntegrations.serverLocationNotifier
        }
    }

    onMount(async () => {
        await loadConfig()

        await invoke('set_rpc', {
            details: 'A roblox bootrapper written from scratch',
            stateText: 'In Integrations Route',
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

<div class="flex flex-col gap-8 max-w-2xl">
    <div class="flex items-center justify-between">
        <div>
            <h1 class="text-3xl font-bold tracking-tight text-stone-100">
                Integrations
            </h1>
            <p class="text-stone-400 mt-1">
                Config integrations outside roblox
            </p>
        </div>
    </div>

    <div class="flex flex-col gap-3">
        <SettingCard
            title="Server Location Notifier"
            description="Get notify when client connect to a server."
        >
            <Switch
                slot="action"
                bind:checked={serverLocationNotifier}
                on:change={handleChanges}
            />
        </SettingCard>
        <SettingCard
            title="Discord RPC (Crush)"
            description="Replace the Roblox Rich Presence with Crush's"
        >
            <Switch
                slot="action"
                bind:checked={crushRpc}
                on:change={handleChanges}
            />
        </SettingCard>
    </div>
</div>
