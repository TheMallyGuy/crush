<script lang="ts">
    import type { Component, Snippet } from 'svelte'

    interface Props {
        title?: string
        description?: string
        icon?: Component | string | null
        doTheGrayThing?: boolean
        clickable?: boolean
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
    {clickable
        ? 'cursor-pointer hover:bg-stone-900/50 active:scale-[0.995]'
        : ''}
    {className}"
    onmouseenter={() => (hovered = true)}
    onmouseleave={() => (hovered = false)}
>
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
