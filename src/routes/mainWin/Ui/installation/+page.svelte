<script lang="ts">
    import { invoke } from '@tauri-apps/api/core'
    import SettingCard from '$lib/components/molecules/SettingCard.svelte'
    import Textbox from '$lib/components/atoms/Textbox.svelte';
    import { Balloon, Rocket } from '@lucide/svelte'
    import { onMount } from 'svelte'
    import { load } from '@tauri-apps/plugin-store'
    import type { Installation } from '$lib/types';
    import { _ } from 'svelte-i18n';

    let version:string
    let config

    async function loadConfig() {
        const store = await load('config.json')
        const savedInstallation =
            await store.get<Installation>('installation')

        if (savedInstallation) {
            version = savedInstallation.version ?? 'latest'
        }
    }

    onMount(async () => {
        await invoke('set_rpc', {
            details: 'A roblox boostrapper written from scratch',
            stateText: 'In Installation Route',
        })

        await loadConfig()
    })

    async function handleChanges() {
        const store = await load('config.json')

        const newInstallation: Installation = {
            version,
        }

        await store.set('installation', newInstallation)

        await store.save()
    }
</script>

<h1 class="text-4xl font-bold text-white">Installation</h1>

<div class="flex flex-col gap-3">
    <SettingCard title={$_('installations.customVersion.title')} description={$_('installations.customVersion.description')} icon={Rocket}>
        <Textbox slot="action" class="w-48 h-8 text-sm" bind:value={version} on:change={handleChanges} />    
    </SettingCard>
</div>