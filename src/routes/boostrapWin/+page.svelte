<script lang="ts">
    import { downloadRoblox, type ProgressEvent } from '$lib/downloadRoblox'
    import { getCurrentWindow } from '@tauri-apps/api/window'
    import { onMount } from 'svelte'

    let status = 'Preparing...'
    let downloadFile = '', downloadDone = 0, downloadTotal = 0
    let extractFile = '', extractDone = 0, extractTotal = 0
    let done = false

    function handleProgress(e: ProgressEvent) {
        if (e.type === 'status') {
            status = e.message
        } else if (e.type === 'download') {
            downloadFile = e.file
            downloadDone = e.done
            downloadTotal = e.total
        } else if (e.type === 'extract') {
            extractFile = e.file
            extractDone = e.done
            extractTotal = e.total
        }
    }

    async function cancel() {
        await getCurrentWindow().close()
    }

    onMount(async () => {
        await downloadRoblox(handleProgress)
        done = true
        status = 'Installation complete'
    })
</script>

<div class="relative h-screen bg-stone-950 text-white selection:bg-stone-800">
    <div class="absolute inset-0 flex flex-col items-center justify-center text-center p-3 pb-24 gap-6">
        <div>
            <h1 class="text-4xl tracking-tight text-stone-100 font-medium">Crush</h1>
            <p class="text-stone-400 mt-2">{status}</p>
        </div>

        {#if !done}
            <div class="w-full max-w-sm flex flex-col gap-3">
                {#if downloadTotal > 0}
                    <div class="flex flex-col gap-1.5">
                        <div class="flex justify-between text-xs text-stone-400">
                            <span class="truncate max-w-[70%]">{downloadFile}</span>
                            <span>{downloadDone}/{downloadTotal}</span>
                        </div>
                        <div class="w-full h-1 bg-stone-800 rounded-full overflow-hidden">
                            <div
                                class="h-full bg-stone-200 rounded-full transition-all duration-300"
                                style="width: {(downloadDone / downloadTotal) * 100}%"
                            />
                        </div>
                    </div>
                {/if}

                {#if extractTotal > 0}
                    <div class="flex flex-col gap-1.5">
                        <div class="flex justify-between text-xs text-stone-400">
                            <span class="truncate max-w-[70%]">{extractFile}</span>
                            <span>{extractDone}/{extractTotal}</span>
                        </div>
                        <div class="w-full h-1 bg-stone-800 rounded-full overflow-hidden">
                            <div
                                class="h-full bg-stone-200 rounded-full transition-all duration-300"
                                style="width: {(extractDone / extractTotal) * 100}%"
                            />
                        </div>
                    </div>
                {/if}
            </div>
        {/if}
    </div>

    <div class="absolute bottom-6 left-0 w-full flex justify-center p-3">
        <div class="w-full max-w-sm">
            {#if done}
                <button class="w-full bg-stone-900 hover:bg-stone-800 active:scale-[0.98] rounded-lg p-4 flex items-center justify-center gap-3 transition-all border border-stone-800 hover:border-stone-700 text-stone-200">
                    <span class="font-medium">Launch</span>
                </button>
            {:else}
                <button
                    on:click={cancel}
                    class="w-full bg-stone-900 hover:bg-stone-800 active:scale-[0.98] rounded-lg p-4 flex items-center justify-center gap-3 transition-all border border-stone-800 hover:border-stone-700 text-stone-200"
                >
                    <span class="font-medium">Cancel</span>
                </button>
            {/if}
        </div>
    </div>
</div>