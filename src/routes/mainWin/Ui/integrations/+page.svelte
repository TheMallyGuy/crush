<script lang="ts">
    import SettingCard from '$lib/components/molecules/SettingCard.svelte'
    import Switch from '$lib/components/atoms/Switch.svelte'
    import { invoke } from '@tauri-apps/api/core'
    import { onMount } from 'svelte'
    import { load } from '@tauri-apps/plugin-store'
    import { Puzzle } from '@lucide/svelte'

    type Integrations = {
        crushRpc: boolean
        serverLocationNotifier: boolean
    }

    let crushRpc = false
    let serverLocationNotifier = false

    async function loadConfig() {
        const store = await load('config.json')
        const savedIntegrations =
            await store.get<Integrations>('integrations')

        if (savedIntegrations) {
            crushRpc = savedIntegrations.crushRpc ?? false
            serverLocationNotifier = savedIntegrations.serverLocationNotifier ?? false
        }
    }

    onMount(async () => {
        await loadConfig()

        await invoke('set_rpc', {
            details: 'A roblox bootstrapper written from scratch',
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

<div class="flex flex-col gap-10 max-w-3xl">
    <header class="flex flex-col gap-2">
        <h1 class="text-clamp-3xl font-black tracking-tight text-white uppercase">
            Integrations
        </h1>
        <p class="text-stone-500 font-medium max-w-xl">
            Configure external services and background processes that enhance your Roblox experience.
        </p>
    </header>

    <div class="flex flex-col gap-4">
        <SettingCard
            title="Server Location Notifier"
            description="Receive desktop notifications when connecting to a new game server."
            icon={Puzzle}
            delay={100}
        >
            <Switch
                slot="action"
                bind:checked={serverLocationNotifier}
                on:change={handleChanges}
            />
        </SettingCard>

        <SettingCard
            title="Discord RPC (Crush)"
            description="Showcase your current game status on Discord using Crush's enhanced Rich Presence."
            icon={Puzzle}
            delay={200}
        >
            <Switch
                slot="action"
                bind:checked={crushRpc}
                on:change={handleChanges}
            />
        </SettingCard>
    </div>
</div>

<style>
    .text-clamp-3xl {
        font-size: clamp(1.875rem, 5vw, 3rem);
    }
</style>
