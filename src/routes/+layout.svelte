<script lang="ts">
    import { onMount } from 'svelte'
    import { loadSavedTheme } from '$lib/theme/themeLoader'
    import { background } from '$lib/stores/background.svelte'
    import { listen } from '@tauri-apps/api/event'
    import { deepLinkUrl } from '$lib/stores/deeplink'
    import { goto } from '$app/navigation'
    import { getCurrentWindow } from '@tauri-apps/api/window'
    import { invoke } from '@tauri-apps/api/core'
    import { sendNotification } from '@tauri-apps/plugin-notification'

    onMount(async () => {
        const win = getCurrentWindow()
        await loadSavedTheme()
        await background.init()

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

        // On startup, we only redirect to the bootstrapper if a deep link was actually received.
        // This prevents the app from launching into Roblox instantly every time it's opened normally.
        const urls = await invoke<string[]>('plugin:deep-link|get_current').catch(
            () => []
        )
        if (
            win.label === 'crushBoostrapChoiceWindow' &&
            urls.length > 0 &&
            !window.location.pathname.includes('/boostrapWin')
        ) {
            deepLinkUrl.set(urls[0])
            goto('/boostrapWin')
        }
    })
</script>

<slot />
