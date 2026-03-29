<script>
    import { getCurrentWindow } from '@tauri-apps/api/window'
    import { X, Minus } from '@lucide/svelte'
    const appWindow = getCurrentWindow()

    const minimize = () => appWindow.minimize()
    const close = () => appWindow.close()
</script>

<nav
    class="flex items-center justify-between h-8 text-stone-300 select-none fixed top-0 left-0 right-0 z-50 bg-obsidian/80 backdrop-blur-md border-b border-stone-800/20"
>
    <div
        data-tauri-drag-region
        class="flex-grow h-full flex items-center pl-4 text-[10px] font-bold uppercase tracking-widest cursor-default opacity-60"
    >
        Crush
    </div>

    <div class="flex h-full">
        <button
            on:click={minimize}
            class="inline-flex items-center justify-center w-11 h-full hover:bg-stone-800/50 hover:text-stone-100 transition-all duration-150"
            title="Minimize"
        >
            <Minus class="w-3.5 h-3.5" />
        </button>

        <button
            on:click={close}
            class="inline-flex items-center justify-center w-11 h-full hover:bg-red-600/20 hover:text-red-500 transition-all duration-150"
            title="Close"
        >
            <X class="w-3.5 h-3.5" />
        </button>
    </div>
</nav>

<div class="h-8"></div>
