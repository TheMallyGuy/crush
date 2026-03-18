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

<h1 class="text-4xl font-bold text-white">
    Installation
</h1>


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