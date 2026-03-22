<script lang="ts">
    import { onMount } from 'svelte'
    import { loadSavedTheme } from '$lib/theme/themeLoader'
    import { check } from '@tauri-apps/plugin-updater'

    onMount(async () => {
        await loadSavedTheme()
        let update = await check()

        if (update) {
            await update.downloadAndInstall()
        }
    })
</script>

<slot />
