<script>
    import { getCurrentWindow } from '@tauri-apps/api/window'
    import { X, Minus } from '@lucide/svelte'
    const appWindow = getCurrentWindow()

    const minimize = () => appWindow.minimize()
    const close = () => appWindow.close()
</script>

<nav
    class="flex items-center justify-between h-8 text-white select-none fixed top-0 left-0 right-0 z-50 bg-obsidian"
>
    <div
        data-tauri-drag-region
        class="flex-grow h-full flex items-center pl-3 text-xs font-medium cursor-default text-stone-400"
    >
        Crush
    </div>

    <div class="flex h-full">
        <button
            on:click={minimize}
            class="inline-flex items-center justify-center w-11 h-full hover:bg-stone-800 transition-colors text-stone-400 hover:text-stone-100"
            title="Minimize"
        >
            <Minus class="w-4 h-4" />
        </button>

        <button
            on:click={close}
            class="inline-flex items-center justify-center w-11 h-full hover:bg-red-600 transition-colors text-stone-400 hover:text-white"
            title="Close"
        >
            <X class="w-4 h-4" />
        </button>
    </div>
</nav>


<div class="h-8"></div>
