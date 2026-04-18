<script lang="ts">
    import { createEventDispatcher } from 'svelte'

    const dispatch = createEventDispatcher()

    export let type: 'button' | 'submit' | 'reset' = 'button'
    export let variant:
        | 'primary'
        | 'secondary'
        | 'outline'
        | 'danger'
        | 'ghost' = 'primary'
    export let size: 'sm' | 'md' | 'lg' = 'md'
    export let disabled = false
    export let loading = false
    export let fullWidth = false

    let className = ''
    export { className as class }

    const variants = {
        primary:
            'bg-sapphire/80 text-white hover:bg-sapphire active:scale-[0.98] backdrop-blur-sm',
        secondary:
            'bg-stone-800/40 text-stone-200 hover:bg-stone-700/60 active:scale-[0.98] backdrop-blur-sm border border-stone-700/30',
        outline:
            'border border-stone-700/50 text-stone-300 hover:bg-stone-800/40 hover:border-stone-600 active:scale-[0.98] backdrop-blur-sm',
        danger: 'bg-red-600/60 text-white hover:bg-red-500 active:scale-[0.98] backdrop-blur-sm',
        ghost: 'text-stone-400 hover:bg-stone-800/40 hover:text-stone-200 active:scale-[0.98]',
    }

    const sizes = {
        sm: 'px-3 py-1.5 text-xs font-medium',
        md: 'px-4 py-2 text-sm font-medium',
        lg: 'px-6 py-3 text-base font-semibold',
    }

    function handleClick(event: MouseEvent) {
        if (!disabled && !loading) {
            dispatch('click', event)
        }
    }
</script>

<button
    {type}
    {disabled}
    class="relative inline-flex items-center justify-center gap-2 rounded-lg transition-all duration-150 focus:outline-none focus:ring-2 focus:ring-sapphire/50 focus:ring-offset-2 focus:ring-offset-obsidian disabled:opacity-50 disabled:cursor-not-allowed
    {variants[variant]} 
    {sizes[size]} 
    {fullWidth ? 'w-full' : ''} 
    {className}"
    on:click={handleClick}
    {...$$restProps}
>
    {#if loading}
        <svg
            class="h-4 w-4 animate-spin text-current"
            xmlns="http://www.w3.org/2000/svg"
            fill="none"
            viewBox="0 0 24 24"
        >
            <circle
                class="opacity-25"
                cx="12"
                cy="12"
                r="10"
                stroke="currentColor"
                stroke-width="4"
            ></circle>
            <path
                class="opacity-75"
                fill="currentColor"
                d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
            ></path>
        </svg>
    {/if}

    {#if $$slots.icon && !loading}
        <slot name="icon" />
    {/if}

    <slot />
</button>
