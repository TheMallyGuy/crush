<script lang="ts">
    import { Info } from '@lucide/svelte'
    import type { Component, Snippet } from 'svelte'

    interface Props {
        title?: string
        description?: string
        icon?: Component | string | null
        doTheGrayThing?: boolean
        clickable?: boolean
        disabled?: string | null
        class?: string
        iconSlot?: Snippet
        action?: Snippet
        footer?: Snippet
    }

    let {
        title = '',
        description = '',
        icon = null,
        doTheGrayThing = false,
        clickable = false,
        disabled = null,
        class: className = '',
        iconSlot,
        action,
        footer,
    }: Props = $props()

    let hovered = $state(false)
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
    class="group relative flex w-full flex-col p-3 transition-all duration-150
    {clickable && !disabled
        ? 'cursor-pointer hover:bg-stone-900/50 active:scale-[0.995]'
        : ''}
    {disabled ? 'cursor-not-allowed opacity-50' : ''}
    {className}"
    onmouseenter={() => (hovered = true)}
    onmouseleave={() => (hovered = false)}
>
    {#if disabled && hovered}
        <div
            class="pointer-events-none absolute bottom-full left-1/2 z-50 mb-2 w-max max-w-xs -translate-x-1/2 rounded-lg border border-stone-800 bg-stone-950 px-3 py-2 text-sm font-medium text-white"
        >
            <div class="flex flex-row gap-3">
                <Info size={19} />
                {disabled}
            </div>

            <div
                class="absolute left-1/2 top-full -mt-px h-2 w-2 -translate-x-1/2 rotate-45 border-b border-r border-stone-800 bg-stone-950"
            ></div>
        </div>
    {/if}

    <div class="flex items-center justify-between gap-5">
        <div class="flex items-center gap-5">
            {#if icon || iconSlot}
                <div
                    class="flex h-10 w-10 shrink-0 items-center justify-center text-stone-400 transition-colors duration-150"
                >
                    {#if iconSlot}
                        {@render iconSlot()}
                    {:else if typeof icon === 'string'}
                        <img
                            src={icon}
                            alt=""
                            class="w-10 h-10 object-contain transition-all duration-150 grayscale-0"
                            class:grayscale={doTheGrayThing && !hovered}
                        />
                    {:else if icon}
                        {@const IconComp = icon}
                        <IconComp size={24} />
                    {/if}
                </div>
            {/if}
            <div class="flex flex-col gap-0.5">
                {#if title}
                    <h3
                        class="text-base font-semibold tracking-tight text-stone-100 break-all"
                    >
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
        {@render action?.()}
    </div>
    {#if footer}
        <div class="mt-5 flex flex-col gap-4">
            {@render footer()}
        </div>
    {/if}
</div>
