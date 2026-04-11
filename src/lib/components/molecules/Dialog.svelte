<script lang="ts">
    import { createEventDispatcher } from 'svelte'
    import { fly, fade } from 'svelte/transition'
    import { X } from '@lucide/svelte'

    const dispatch = createEventDispatcher()

    export let open = false
    export let title = ''
    export let description = ''

    let className = ''
    export { className as class }

    function close() {
        dispatch('close')
    }

    function handleKeyDown(event: KeyboardEvent) {
        if (event.key === 'Escape' && open) {
            close()
        }
    }
</script>

<svelte:window on:keydown={handleKeyDown} />

{#if open}
    <div
        class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-sm"
        transition:fade={{ duration: 150 }}
        on:click|self={close}
    >
        <!-- Modal Container -->
        <div
            class="w-full max-w-md bg-anthracite border border-stone-800/40 rounded-2xl p-6 shadow-2xl relative overflow-hidden flex flex-col {className}"
            transition:fly={{ y: 20, duration: 250, opacity: 0 }}
        >
            <!-- Decorative Glow -->
            <div class="absolute -top-12 -left-12 w-24 h-24 bg-sapphire/5 blur-3xl rounded-full pointer-events-none"></div>

            <div class="flex items-start justify-between mb-2">
                <h3 class="text-xl font-semibold text-stone-100">{title}</h3>
                <button
                    on:click={close}
                    class="p-1 rounded-lg text-stone-500 hover:text-stone-300 hover:bg-stone-800/50 transition-colors"
                >
                    <X size={20} />
                </button>
            </div>

            <div class="text-stone-400 text-[15px] leading-relaxed mb-6">
                {#if description}
                    {description}
                {:else}
                    <slot name="description" />
                {/if}
            </div>

            <div class="flex items-center justify-end gap-3 mt-auto">
                <slot name="actions" />
            </div>
        </div>
    </div>
{/if}
