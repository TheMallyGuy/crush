<script>
    import { getCurrentWindow } from '@tauri-apps/api/window'
    import { X, Minus } from '@lucide/svelte'
    const appWindow = getCurrentWindow()

    const minimize = () => appWindow.minimize()
    const close = () => appWindow.close()
</script>

<nav
    class="flex items-center justify-between h-10 text-stone-400 select-none fixed top-0 left-0 right-0 z-50 bg-obsidian/40 backdrop-blur-sm border-b border-white/5"
>
    <div
        data-tauri-drag-region
        class="flex-grow h-full flex items-center pl-4 text-[11px] font-bold tracking-widest uppercase cursor-default group"
    >
        <span class="group-hover:text-sapphire transition-colors">Crush Control</span>
    </div>

    <div class="flex h-full">
        <button
            on:click={minimize}
            class="inline-flex items-center justify-center w-12 h-full hover:bg-white/5 hover:text-white transition-all active:scale-95"
            title="Minimize"
        >
            <Minus class="w-4 h-4" />
        </button>

        <button
            on:click={close}
            class="inline-flex items-center justify-center w-12 h-full hover:bg-red-500/80 hover:text-white transition-all active:scale-95"
            title="Close"
        >
            <X class="w-4 h-4" />
        </button>
    </div>
</nav>

<div class="h-10"></div>
