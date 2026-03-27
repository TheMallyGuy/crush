<script lang="ts">
    import SettingCard from "$lib/components/SettingCard.svelte";
    import Switch from "$lib/components/Switch.svelte"
    import { Bell } from "@lucide/svelte";
    import { invoke } from "@tauri-apps/api/core"
    import { onMount } from "svelte";

    onMount(async () => {
        await invoke("set_rpc", { details: "A roblox boostrapper written from scratch", stateText: "In Intergrations Route" })
    })

    function handleChanges() {
        // doesnt do anything yet
    }

</script>

<div class="flex flex-col gap-8 max-w-2xl">
    <div class="flex items-center justify-between">
        <div>
            <h1 class="text-3xl font-bold tracking-tight text-stone-100">
                Intergrations
            </h1>
            <p class="text-stone-400 mt-1">Config intergrations outside roblox</p>
        </div>
    </div>

    <div class="flex flex-col gap-3">
        <SettingCard
            title="Server Location Notifier"
            description="Get notify when client connect to a server."
        >
            <Switch slot="action"/>
        </SettingCard>
            <SettingCard
            title="Discord RPC (Crush)"
            description="Replace the Roblox Rich Presence with Crush's"
        >
            <Switch slot="action"/>
        </SettingCard>
    </div>
</div>