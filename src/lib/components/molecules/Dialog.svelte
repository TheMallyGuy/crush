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
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <div
        class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/40 backdrop-blur-md"
        transition:fade={{ duration: 150 }}
        on:click|self={close}
    >
        <div
            class="w-full max-w-md bg-anthracite/80 backdrop-blur-xl border border-stone-800/40 rounded-xl p-6 shadow-2xl relative overflow-hidden flex flex-col {className}"
            transition:fly={{ y: 20, duration: 250, opacity: 0 }}
        >
            <div
                class="absolute -top-12 -left-12 w-32 h-32 bg-sapphire/10 blur-3xl rounded-full pointer-events-none"
            ></div>
            <div
                class="absolute -bottom-12 -right-12 w-32 h-32 bg-sapphire/5 blur-3xl rounded-full pointer-events-none"
            ></div>

            <div class="flex items-start justify-between mb-2">
                <h3 class="text-xl font-bold tracking-tight text-stone-100">
                    {title}
                </h3>
                <button
                    on:click={close}
                    class="p-1 rounded-lg text-stone-500 hover:text-stone-300 hover:bg-stone-800/50 transition-all duration-150 active:scale-95"
                >
                    <X size={20} />
                </button>
            </div>

            <div class="text-stone-500 font-medium text-sm leading-relaxed mb-4">
                {#if description}
                    {description}
                {:else}
                    <slot name="description" />
                {/if}
            </div>

            <div class="flex flex-col gap-4 mb-6">
                <slot />
            </div>

            <div class="flex items-center justify-end gap-3 mt-auto">
                <slot name="actions" />
            </div>
        </div>
    </div>
{/if}
