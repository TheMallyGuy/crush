<script lang="ts">
    import { _ } from "svelte-i18n";
    import Button from "$lib/components/atoms/Button.svelte";
    import { goto } from "$app/navigation";
    import SettingCard from "$lib/components/molecules/SettingCard.svelte"
    import { onMount } from "svelte"
    import Textbox from "$lib/components/atoms/Textbox.svelte"
    import WebSocket from '@tauri-apps/plugin-websocket';
    import { invoke } from "@tauri-apps/api/core"

    let wssServer: string

    async function playGame() {
        await invoke('create_or_focus_window', {
            label: 'CrushBoostrap',
            url: 'boostrapWin',
            title: 'Crush',
            width: 500.0,
            height: 350.0,
            minWidth: 500,
            minHeight: 350.0,
        })
    }

    async function connectAndListen() {
        if (!wssServer) return; // guard empty input
        try {
            const ws = await WebSocket.connect(wssServer);
            await ws.send("launchRoblox,intergration,installations");
            
            const removeListener = ws.addListener((msg) => {
                switch (msg.data) {
                    case "launchRoblox": {
                        playGame()
                    }
                }
            });

        } catch (e) {
            console.error("WebSocket error:", e);
        }
    }

</script>

<div class="flex flex-col gap-8">
    <div class="flex items-center justify-between">
        <div>
            <h1 class="text-3xl font-bold tracking-tight text-stone-100">
                websocket bullshit
            </h1>
            <p class="text-stone-400 mt-1">
                maximum of free will
            </p>
        </div>
                <div class="flex items-center gap-2">
            <Button variant="secondary" onclick={() => goto('../settings')}>
                back
            </Button>
        </div>
    </div>

    <SettingCard
        title="Set a connection"
        description="no"
    >
        <Textbox slot="action" bind:value={wssServer}/>
    </SettingCard>

    <Button on:click={connectAndListen}>
        Connect
    </Button>
</div>