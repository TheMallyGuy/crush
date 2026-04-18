<script lang="ts">
    import type { Component } from 'svelte'
    import { createEventDispatcher } from 'svelte'
    const dispatch = createEventDispatcher()
    export let title = ''
    export let description = ''
    export let icon: Component | null = null
    export let disabled = false
    export let size: 'sm' | 'md' = 'md'
    let className = ''
    export { className as class }
    function handleClick(event: MouseEvent) {
        if (!disabled) {
            dispatch('click', event)
        }
    }
</script>
<button
    type="button"
    {disabled}
    class="group relative flex w-full items-start rounded-xl bg-anthracite/40 transition-all duration-150 border border-stone-800/20 text-left outline-none focus-visible:ring-2 focus-visible:ring-sapphire/50 disabled:opacity-50 disabled:cursor-not-allowed
    {size === 'sm' ? 'gap-3 p-3' : 'gap-4 p-6'}
    {!disabled ? 'cursor-pointer hover:bg-stone-900/50 hover:border-stone-700/40 active:scale-[0.995]' : ''}
    {className}"
    on:click={handleClick}
    {...$$restProps}
>
    {#if icon || $$slots.icon}
        <div class="shrink-0 text-stone-400 group-hover:text-sapphire transition-colors duration-150 mt-0.5">
            <slot name="icon">
                <svelte:component this={icon} size={size === 'sm' ? 16 : 24} />
            </slot>
        </div>
    {/if}
    <div class="flex flex-col {size === 'sm' ? 'gap-0.5' : 'gap-1.5'}">
        {#if title || $$slots.title}
            <h3
                class="font-bold tracking-tight text-stone-100 group-hover:text-white transition-colors duration-150
                {size === 'sm' ? 'text-sm' : 'text-lg'}"
            >
                <slot name="title">{title}</slot>
            </h3>
        {/if}
        {#if description || $$slots.description}
            <p class="font-medium text-stone-500 leading-relaxed group-hover:text-stone-400 transition-colors duration-150
            {size === 'sm' ? 'text-xs' : 'text-sm'}">
                <slot name="description">{description}</slot>
            </p>
        {/if}
    </div>
    <slot />
</button>