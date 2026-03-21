<script lang="ts">
    import { onMount } from 'svelte'
    import { loadSavedTheme } from '$lib/theme/themeLoader'

    onMount(async () => {
        await loadSavedTheme()
    })
</script>

<slot />
