<script lang="ts">
    import Button from '$lib/components/atoms/Button.svelte'
    import { Rocket } from '@lucide/svelte'
    import { invoke } from '@tauri-apps/api/core'
    import { getCurrentWindow } from '@tauri-apps/api/window'

    async function launchBoostrap() {
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
            // wait before killing to prevent crash
            getCurrentWindow().close()
        }, 100)
    }
</script>

<div
    class="bg-[#1a1c20] border-t border-l border-[#2a2d32] p-4 px-8 rounded-tl-3xl shadow-2xl flex items-center"
>
    <Button
        variant="primary"
        size="md"
        class="rounded-full px-10 shadow-lg shadow-blue-600/20"
        on:click={launchBoostrap}
    >
        <Rocket class="size-4 mr-2" />
        Launch
    </Button>
</div>