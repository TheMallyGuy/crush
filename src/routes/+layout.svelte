<script lang="ts">
    import { onMount } from 'svelte'
    import { loadSavedTheme } from '$lib/theme/themeLoader'
    import { check } from '@tauri-apps/plugin-updater'
    import { listen } from '@tauri-apps/api/event'
    import { deepLinkUrl } from '$lib/stores/deeplink'
    import { goto } from '$app/navigation'

    onMount(async () => {
        await loadSavedTheme()

        await listen<string>('deep-link-received', (event) => {
            deepLinkUrl.set(event.payload)
            goto('/boostrapWin')
        })

        const update = await check()
        if (update) {
            await update.downloadAndInstall()
            return
        }

        goto('/boostrapWin')
    })
</script>

<slot />
