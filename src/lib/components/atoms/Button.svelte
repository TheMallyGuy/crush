<script lang="ts">
    import type { Snippet } from 'svelte'

    interface Props {
        type?: 'button' | 'submit' | 'reset'
        variant?: 'primary' | 'secondary' | 'danger' | 'ghost'
        size?: 'sm' | 'md' | 'lg'
        disabled?: boolean
        class?: string
        onclick?: (event: MouseEvent) => void
        children?: Snippet
        [key: string]: unknown
    }

    let {
        type = 'button',
        variant = 'primary',
        size = 'md',
        disabled = false,
        class: className = '',
        onclick,
        children,
        ...rest
    }: Props = $props()

    const variants = {
        primary: 'bg-sapphire text-white hover:bg-sapphire-light shadow-sapphire/20',
        secondary: 'bg-stone-800/40 text-stone-300 hover:bg-stone-700/60 hover:text-white border border-stone-700/40',
        danger: 'bg-red-500/10 text-red-400 hover:bg-red-500 hover:text-white border border-red-500/20',
        ghost: 'bg-transparent text-stone-400 hover:bg-stone-800/60 hover:text-white'
    }

    const sizes = {
        sm: 'px-3 py-1.5 text-xs',
        md: 'px-5 py-2.5 text-sm',
        lg: 'px-8 py-3.5 text-base'
    }
</script>

<button
    {type}
    {disabled}
    {onclick}
    class="inline-flex cursor-target items-center justify-center font-semibold duration-150 active:opacity-10 disabled:opacity-50 disabled:cursor-not-allowed disabled:active:scale-100 {variants[variant]} {sizes[size]} {className}"
    {...rest}
>
    {@render children?.()}
</button>
