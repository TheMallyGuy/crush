<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';

  function openmainwin() {
    invoke("open_main_window")
    invoke("kill_window", { windowName: "crushBoostrapChoiceWindow" });
  }
</script>

<h1 class="text-4xl font-bold text-blue-500">
  choice box
</h1>

<button on:click={openmainwin}>
  open main win
</button>