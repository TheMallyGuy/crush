<script lang="ts">
    import type { Component, Snippet } from 'svelte'
    import { ChevronDown } from '@lucide/svelte'
    import { slide } from 'svelte/transition'

    interface Props {
        title?: string
        description?: string
        icon?: Component | string | null
        isOpen?: boolean
        class?: string
        iconSlot?: Snippet
        action?: Snippet
        footer?: Snippet
        children?: Snippet
    }

    let {
        title = '',
        description = '',
        icon = null,
        isOpen = $bindable(false),
        class: className = '',
        iconSlot,
        action,
        footer,
        children,
    }: Props = $props()

    function toggle() {
        isOpen = !isOpen
    }
</script>

<div
    class="group relative flex w-full flex-col p-3 transition-all duration-150 {className}"
>
    <button
        type="button"
        class="flex items-center justify-between gap-5 text-left cursor-pointer w-full focus:outline-none"
        onclick={toggle}
    >
        <div class="flex items-center gap-5">
            {#if icon || iconSlot}
                <div class="flex h-10 w-10 shrink-0 items-center justify-center text-stone-400 transition-colors duration-150 overflow-hidden">
                    {#if iconSlot}
                        {@render iconSlot()}
                    {:else if typeof icon === 'string'}
                        <img src={icon} alt="" class="w-full h-full object-cover" />
                    {:else if icon}
                        {@const IconComp = icon}
                        <IconComp size={24} />
                    {/if}
                </div>
            {/if}
            <div class="flex flex-col gap-0.5">
                {#if title}
                    <h3 class="text-base font-semibold tracking-tight text-stone-100">
                        {title}
                    </h3>
                {/if}
                {#if description}
                    <p class="text-sm font-medium text-stone-300">
                        {description}
                    </p>
                {/if}
            </div>
        </div>
        <div class="flex items-center gap-3">
            {@render action?.()}
            <div class="text-stone-500 transition-transform duration-200 {isOpen ? 'rotate-180' : ''}">
                <ChevronDown size={20} />
            </div>
        </div>
    </button>
    {#if isOpen}
        <div transition:slide={{ duration: 200 }}>
            <div class="mt-5 flex flex-col gap-4">
                {@render children?.()}
                {#if footer}
                    <div class="flex flex-col gap-4">
                        {@render footer()}
                    </div>
                {/if}
            </div>
        </div>
    {/if}
</div>
