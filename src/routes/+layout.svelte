<script lang="ts">
    import { onMount } from 'svelte'
    import { loadSavedTheme } from '$lib/theme/themeLoader'
    import { check } from '@tauri-apps/plugin-updater'
    import { listen } from '@tauri-apps/api/event'
    import { deepLinkUrl } from '$lib/stores/deeplink'
    import { goto } from '$app/navigation'

    onMount(async () => {
        await loadSavedTheme()

        await listen<string>('deep-link-received', (event) => { // @pochita there is a chance (or bug) thats when you open deep links and then click on config or any page from choice win will get again rediect to the boostrap win. find a solution to reslove this.
            deepLinkUrl.set(event.payload)
            if (!window.location.pathname.startsWith('/mainWin/')) {
                goto('/boostrapWin')
            }
        })

        const update = await check()
        if (update) {
            await update.downloadAndInstall()
            return
        }

        if (window.location.pathname === '/') {
            goto('/boostrapWin')
        }
    })
</script>

<slot />
