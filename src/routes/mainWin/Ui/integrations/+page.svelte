<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';

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