<script lang="ts">
    import type { Component } from 'svelte'
    export let title = ''
    export let description = ''
    export let icon: Component | string | null = null
    export let iconHover: string | null = null
    export let clickable = false
    let className = ''
    export { className as class }

    let hovered = false
</script>

<div
    class="group relative flex w-full flex-col rounded-xl bg-anthracite/40 p-5 transition-all duration-150 border border-stone-800/20
    {clickable
        ? 'cursor-pointer hover:bg-stone-900/50 active:scale-[0.995]'
        : ''}
    {className}"
    on:mouseenter={() => (hovered = true)}
    on:mouseleave={() => (hovered = false)}
>
    <div class="flex items-center justify-between gap-5">
        <div class="flex items-center gap-5">
            {#if icon || $$slots.icon}
                <div
                    class="flex h-10 w-10 shrink-0 items-center justify-center text-stone-400 transition-colors duration-150"
                >
                    <slot name="icon">
                        {#if typeof icon === 'string'}
                            <img
                                src={iconHover && hovered ? iconHover : icon}
                                alt=""
                                class="w-10 h-10 object-contain transition-opacity duration-150"
                            />
                        {:else if icon}
                            <svelte:component this={icon} size={24} />
                        {/if}
                    </slot>
                </div>
            {/if}
            <div class="flex flex-col gap-0.5">
                {#if title || $$slots.title}
                    <h3
                        class="text-base font-semibold tracking-tight text-stone-100"
                    >
                        <slot name="title">{title}</slot>
                    </h3>
                {/if}
                {#if description || $$slots.description}
                    <p class="text-sm font-medium text-stone-500">
                        <slot name="description">{description}</slot>
                    </p>
                {/if}
            </div>
        </div>
        <slot name="action" />
    </div>
    {#if $$slots.footer}
        <div class="mt-5 flex flex-col gap-4">
            <slot name="footer" />
        </div>
    {/if}
</div>
