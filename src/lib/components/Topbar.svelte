<script>
    import { getCurrentWindow } from '@tauri-apps/api/window'

    const appWindow = getCurrentWindow()

    const minimize = () => appWindow.minimize()
    const toggleMaximize = () => appWindow.toggleMaximize()
    const close = () => appWindow.close()
</script>

<nav
    class="flex items-center justify-between h-8 text-white select-none fixed top-0 left-0 right-0 z-50"
>
    <div
        data-tauri-drag-region
        class="flex-grow h-full flex items-center pl-3 text-xs font-medium cursor-default"
    >
        Crush
    </div>

    <div class="flex h-full">
        <button
            on:click={minimize}
            class="inline-flex items-center justify-center w-11 h-full hover:bg-zinc-800 transition-colors"
            title="Minimize"
        >
            <svg class="w-4 h-4" viewBox="0 0 24 24"
                ><path fill="currentColor" d="M19 13H5v-2h14z" /></svg
            >
        </button>

        <button
            on:click={toggleMaximize}
            class="inline-flex items-center justify-center w-11 h-full hover:bg-zinc-800 transition-colors"
            title="Maximize"
        >
            <svg class="w-3 h-3" viewBox="0 0 24 24"
                ><path
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    d="M3 3h18v18H3z"
                /></svg
            >
        </button>

        <button
            on:click={close}
            class="inline-flex items-center justify-center w-11 h-full hover:bg-red-600 transition-colors"
            title="Close"
        >
            <svg class="w-4 h-4" viewBox="0 0 24 24"
                ><path
                    fill="currentColor"
                    d="M19 6.41L17.59 5L12 10.59L6.41 5L5 6.41L10.59 12L5 17.59L6.41 19L12 13.41L17.59 19L19 17.59L13.41 12L19 6.41Z"
                /></svg
            >
        </button>
    </div>
</nav>

<div class="h-8"></div>
