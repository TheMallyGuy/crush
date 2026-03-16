<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';

  async function openchoicewin() {
      await invoke("open_choice_window");

      setTimeout(() => { // wait before killing to prevent crash
          invoke("kill_window", { windowName: "CrushMainWindow" });
      }, 100);
  }
</script>

<div class="flex h-screen w-full flex-col items-center justify-center gap-4 font-medium">
  <h1 class="text-[50px] text-center leading-tight">you just discovered a super super super rare error</h1>
  <button class="px-4 py-2 bg-[#121417]" on:click={openchoicewin}>
    go back to choice menu
  </button>
</div>