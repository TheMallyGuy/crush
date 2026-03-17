<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import SettingCard from '$lib/components/SettingCard.svelte';
  import { Balloon } from '@lucide/svelte';

    async function openmainwin() {
        await invoke("open_choice_window");

        setTimeout(() => { // wait before killing to prevent crash
            invoke("kill_window", { windowName: "CrushMainWindow" });
        }, 100);
    }
</script>

<h1 class="text-4xl font-bold text-blue-500">
    something that very cool and tuff too
</h1>

<button on:click={openmainwin}>
  open choice win
</button>

<div class="flex-col flex gap-4">
    <SettingCard title="do balls" description="balls are good" icon={Balloon}>
    </SettingCard>
    <SettingCard title="do balls" description="balls are good" icon={Balloon}>
    </SettingCard>
    <SettingCard title="do balls" description="balls are good" icon={Balloon}>
    </SettingCard>
    <SettingCard title="do balls" description="balls are good" icon={Balloon}>
    </SettingCard>
</div>