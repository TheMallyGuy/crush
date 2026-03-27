<script lang="ts">
    import { invoke } from '@tauri-apps/api/core'
    import SettingCard from '$lib/components/SettingCard.svelte'
    import { Balloon } from '@lucide/svelte'
    import { onMount } from 'svelte';

    onMount(async () => {
        await invoke("set_rpc", { details: "A roblox boostrapper written from scratch", stateText: "In Installation Route" })
    })

</script>

<h1 class="text-4xl font-bold text-white">Installation</h1>

<div class="flex-col flex gap-4"></div>
