<script lang="ts">
    import Button from '$lib/components/atoms/Button.svelte'
    import { deepLinkUrl } from '$lib/stores/deeplink'
    import { invoke } from '@tauri-apps/api/core'
    import { getCurrentWindow } from '@tauri-apps/api/window'
    import { _ } from 'svelte-i18n';
    async function launchBoostrap() {
        deepLinkUrl.set(null)
        localStorage.removeItem('deepLinkUrl')
        await invoke('create_or_focus_window', {
            label: 'CrushBoostrap',
            url: 'boostrapWin',
            title: 'Crush',
            width: 500.0,
            height: 350.0,
            minWidth: 500,
            minHeight: 350.0,
        })
        setTimeout(() => {
            getCurrentWindow().close()
        }, 100)
    }
</script>
<div class="p-4 px-8 rounded-tl-3xl flex items-center transparent">
    <Button
        variant="primary"
        size="sm"
        class="rounded-full px-4 shadow-lg h-9"
        on:click={launchBoostrap}
    >
        {$_('elements.launchBar')}
    </Button>
</div>