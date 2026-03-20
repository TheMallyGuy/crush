<script lang="ts">
    import { invoke } from '@tauri-apps/api/core'
    import { downloadRoblox } from '$lib/downloadRoblox';

    let description = 'Preveiii!!'

    function setDescription(text: string) {
        description = text
    }

    function changeDescription() {
        setDescription('something is working idfk')
    }

    async function downloadBackround() {
        try {
            const url = await invoke('download_roblox')
            console.log('download URL debug thinggy:', url)
        } catch (e) {
            console.error('error:', e)
        }
    }
</script>

<div class="relative h-screen bg-stone-950 text-white selection:bg-stone-800">
    <div
        class="absolute inset-0 flex flex-col items-center justify-center text-center p-3 pb-24"
    >
        <h1 class="text-4xl tracking-tight text-stone-100 font-medium">
            Crush
        </h1>
        <p class="text-stone-400 mt-2">
            {description}
        </p>
    </div>

    <div class="absolute bottom-6 left-0 w-full flex justify-center p-3">
        <div class="w-full max-w-sm">
            <button
                on:click={downloadRoblox}
                // later change to real code at here!!
                class="w-full bg-stone-900 hover:bg-stone-800 active:scale-[0.98] disabled:opacity-50 rounded-lg p-4 flex items-center justify-center gap-3 transition-all border border-stone-800 hover:border-stone-700 text-stone-200"
            >
                <span class="font-medium">Cancel</span>
            </button>
        </div>
    </div>
</div>
