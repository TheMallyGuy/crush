<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  
  async function openmainwin() {
      await invoke("open_main_window");

      setTimeout(() => { // wait before killing to prevent crash
          invoke("kill_window", { windowName: "crushBoostrapChoiceWindow" });
      }, 100);
  }
</script>

<h1 class="text-4xl font-bold text-blue-500">
  choice box
</h1>

<button on:click={openmainwin}>
  open main win
</button>