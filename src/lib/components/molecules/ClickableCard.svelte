<script lang="ts">
    import type { Component, Snippet } from 'svelte'

    interface Props {
        title?: string
        description?: string
        icon?: Component | null
        disabled?: boolean
        size?: 'sm' | 'md'
        class?: string
        onclick?: (event: MouseEvent) => void
        children?: Snippet
        [key: string]: unknown
    }

    let {
        title = '',
        description = '',
        icon = null,
        disabled = false,
        size = 'md',
        class: className = '',
        onclick,
        children,
        ...rest
    }: Props = $props()

    function handleClick(event: MouseEvent) {
        if (!disabled) {
            onclick?.(event)
        }
    }
</script>

<button
    type="button"
    {disabled}
    class="group cursor-target relative flex w-full items-start bg-anthracite/40 transition-all duration-150 border border-stone-800/20 text-left outline-none focus-visible:ring-2 focus-visible:ring-sapphire/50 disabled:opacity-50 disabled:cursor-not-allowed
    {size === 'sm' ? 'gap-3 p-3' : 'gap-4 p-6'}
    {!disabled ? 'cursor-pointer hover:bg-stone-900/50 hover:border-stone-700/40 active:scale-[0.995]' : ''}
    {className}"
    onclick={handleClick}
    {...rest}
>
    {#if icon}
        {@const IconComp = icon}
        <div class="shrink-0 text-stone-400 group-hover:text-sapphire transition-colors duration-150 mt-0.5">
            <IconComp size={size === 'sm' ? 16 : 24} />
        </div>
    {/if}
    <div class="flex flex-col {size === 'sm' ? 'gap-0.5' : 'gap-1.5'}">
        {#if title}
            <h3
                class="font-bold tracking-tight text-stone-100 group-hover:text-white transition-colors duration-150
                {size === 'sm' ? 'text-sm' : 'text-lg'}"
            >
                {title}
            </h3>
        {/if}
        {#if description}
            <p class="font-medium text-stone-300 leading-relaxed group-hover:text-stone-200 transition-colors duration-150
            {size === 'sm' ? 'text-xs' : 'text-sm'}">
                {description}
            </p>
        {/if}
    </div>
    {@render children?.()}
</button>
