<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from '@tauri-apps/api/core';
  import { load } from '@tauri-apps/plugin-store';

  let firstLaunchValue: boolean | undefined;

  async function checkLaunch() {
    const store = await load('config.json');

    let firstLaunch = await store.get<boolean>('FirstLaunch');

    if (firstLaunch === undefined) {
      await store.set('FirstLaunch', true);
      await store.save();
      return true;
    }

    return false;
  }


  async function openmainwin() {
      if (firstLaunchValue) {
        await invoke("open_main_window", { url : "mainWin/crushHello"}); 
      } else {
        await invoke("open_main_window", { url : "mainWin/Ui"}); 
      }
      setTimeout(() => { // wait before killing to prevent crash
          invoke("kill_window", { windowName: "crushBoostrapChoiceWindow" });
      }, 100);
  }

  onMount(async () => {
    firstLaunchValue = await checkLaunch();
  });
</script>
