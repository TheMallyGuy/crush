<script lang="ts">
    import Button from '../atoms/Button.svelte'
    import { Rocket } from '@lucide/svelte'
    import { invoke } from '@tauri-apps/api/core'
    import { getCurrentWindow } from '@tauri-apps/api/window'
    import { createEventDispatcher } from 'svelte'

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

    const dispatch = createEventDispatcher()
</script>

<div
    class="bg-anthracite/50 backdrop-blur-xl border border-white/5 p-5 px-10 rounded-tl-3xl shadow-[0_-10px_40px_-15px_rgba(0,0,0,0.5)] flex items-center mb-0 mr-0 transition-all group hover:bg-anthracite/70"
>
    <Button
        variant="primary"
        size="lg"
        class="rounded-full px-12 shadow-sapphire group-hover:scale-105 active:scale-95 transition-transform font-bold tracking-widest uppercase text-xs"
        on:click={launchBoostrap}
    >
        <Rocket class="size-4 mr-3" />
        Launch
    </Button>
</div>
