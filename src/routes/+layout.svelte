<script lang="ts">
    import { onMount } from 'svelte'
    import { loadSavedTheme } from '$lib/theme/themeLoader'
    import { check } from '@tauri-apps/plugin-updater'
    import { listen } from '@tauri-apps/api/event'
    import { deepLinkUrl } from '$lib/stores/deeplink'
    import { goto } from '$app/navigation'
    import { getCurrentWindow } from '@tauri-apps/api/window'

    onMount(async () => {
        const win = getCurrentWindow()
        await loadSavedTheme()

        await listen<string>('deep-link-received', (event) => {
            deepLinkUrl.set(event.payload)
            // Only the main choice window handles deep link redirects to the bootstrapper.
            // This prevents secondary windows (like the Config window) from being hijacked and forced to redirect.
            if (
                win.label === 'crushBoostrapChoiceWindow' &&
                !window.location.pathname.includes('/boostrapWin')
            ) {
                goto('/boostrapWin')
            }
        })

        const update = await check()
        if (update) {
            await update.downloadAndInstall()
            return
        }

        // Automatic startup redirect removed in favor of explicit 'url' in tauri.conf.json.
        // This ensures secondary windows (like Config) open their intended routes without layout interference.
    })
</script>

<slot />
