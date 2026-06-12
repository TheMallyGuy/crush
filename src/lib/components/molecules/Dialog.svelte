<script lang="ts">
    import type { Snippet } from 'svelte'
    import { fly, fade } from 'svelte/transition'
    import { X } from '@lucide/svelte'

    interface Props {
        open?: boolean
        title?: string
        description?: string
        class?: string
        onclose?: () => void
        children?: Snippet
        actions?: Snippet
        descriptionSlot?: Snippet
    }

    let {
        open = $bindable(false),
        title = '',
        description = '',
        class: className = '',
        onclose,
        children,
        actions,
        descriptionSlot,
    }: Props = $props()

    function close() {
        onclose?.()
    }

    function handleKeyDown(event: KeyboardEvent) {
        if (event.key === 'Escape' && open) {
            close()
        }
    }
</script>

<svelte:window onkeydown={handleKeyDown} />

{#if open}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <div
        class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/40 backdrop-blur-md"
        transition:fade={{ duration: 150 }}
        onclick={(e) => {
            if (e.target === e.currentTarget) close()
        }}
    >
        <div
            class="w-full max-w-md bg-anthracite/80 backdrop-blur-xl border border-stone-800/40 p-6 relative overflow-hidden flex flex-col {className}"
            transition:fly={{ y: 20, duration: 250, opacity: 0 }}
        >
            <div class="flex items-start justify-between mb-2">
                <h3 class="text-xl font-bold tracking-tight text-stone-100">
                    {title}
                </h3>
                <button
                    onclick={close}
                    class="p-1 text-stone-500 hover:text-stone-300 hover:bg-stone-800/50 transition-all duration-150 active:scale-95 cursor-target"
                >
                    <X size={20} />
                </button>
            </div>

            <div
                class="text-stone-300 font-medium text-sm leading-relaxed mb-4"
            >
                {#if description}
                    {description}
                {:else}
                    {@render descriptionSlot?.()}
                {/if}
            </div>

            <div class="flex flex-col gap-4 mb-6">
                {@render children?.()}
            </div>

            <div class="flex items-center justify-end gap-3 mt-auto">
                {@render actions?.()}
            </div>
        </div>
    </div>
{/if}
