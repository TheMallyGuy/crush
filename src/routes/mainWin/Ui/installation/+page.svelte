<script lang="ts">
    import { invoke } from '@tauri-apps/api/core'
    import SettingCard from '$lib/components/molecules/SettingCard.svelte'
    import Textbox from '$lib/components/atoms/Textbox.svelte';
    import { Balloon, Rocket } from '@lucide/svelte'
    import { onMount } from 'svelte'
    import { load } from '@tauri-apps/plugin-store'

    let version = ""
    let config

    onMount(async () => {
        await invoke('set_rpc', {
            details: 'A roblox boostrapper written from scratch',
            stateText: 'In Installation Route',
        })

        config = await load("config.json")
    })

    async function handleVersionChange() {
        
    }
</script>

<h1 class="text-4xl font-bold text-white">Installation</h1>

<div class="flex flex-col gap-3">
    <SettingCard title="Version" description="Skip update check and download the version." icon={Rocket}>
        <Textbox slot="action" class="w-48 h-8 text-sm" bind:value={version} />    
    </SettingCard>
</div>