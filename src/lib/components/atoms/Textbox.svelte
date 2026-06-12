<script lang="ts">
    import type { Snippet } from 'svelte'

    interface Props {
        value?: string | number
        type?: 'text' | 'password' | 'number' | 'email' | 'tel' | 'url'
        placeholder?: string
        label?: string
        error?: string
        disabled?: boolean
        id?: string
        fullWidth?: boolean
        containerClass?: string
        class?: string
        oninput?: (value: string | number) => void
        onchange?: (value: string | number) => void
        onenter?: (value: string | number) => void
        icon?: Snippet
        action?: Snippet
        [key: string]: unknown
    }

    let {
        value = $bindable(''),
        type = 'text',
        placeholder = '',
        label = '',
        error = '',
        disabled = false,
        id = undefined,
        fullWidth = true,
        containerClass = '',
        class: className = '',
        oninput,
        onchange,
        onenter,
        icon,
        action,
        ...rest
    }: Props = $props()

    function handleInput(event: Event) {
        const target = event.target as HTMLInputElement
        const newValue = type === 'number' ? Number(target.value) : target.value
        value = newValue
        oninput?.(newValue)
    }

    function handleChange() {
        onchange?.(value)
    }

    function handleKeyDown(event: KeyboardEvent) {
        if (event.key === 'Enter') {
            onenter?.(value)
        }
    }
</script>

<div
    class="flex flex-col gap-2 {fullWidth ? 'w-full' : ''} {containerClass}"
>
    {#if label}
        <label
            for={id}
            class="text-[11px] text-stone-500"
        >
            {label}
        </label>
    {/if}

    <div class="relative w-full">
        {#if icon}
            <div
                class="pointer-events-none absolute inset-y-0 left-0 flex items-center pl-4 text-stone-500"
            >
                {@render icon()}
            </div>
        {/if}

        <input
            {id}
            {type}
            {placeholder}
            {disabled}
            {value}
            oninput={handleInput}
            onchange={handleChange}
            onkeydown={handleKeyDown}
            class="block w-full transition-all duration-150 outline-none text-sm border cursor-target
                {icon ? 'pl-11' : 'px-4'}
                py-2.5
                {disabled
                ? 'bg-stone-900/30 border-stone-800/20 text-stone-600 cursor-not-allowed'
                : 'bg-stone-900/50 border-stone-800/40 text-stone-200 placeholder-stone-600 hover:border-stone-700/60'}
                {error
                ? 'border-red-500/40 focus:ring-2 focus:ring-red-500/10 focus:border-red-500/60'
                : 'focus:ring-2 focus:ring-sapphire/20 focus:border-sapphire/40'}
                {className}"
            {...rest}
        />

        {#if action}
            <div class="absolute inset-y-0 right-0 flex items-center pr-1.5">
                {@render action()}
            </div>
        {/if}
    </div>

    {#if error}
        <p class="px-1 text-[11px] font-medium text-red-400/90">
            {error}
        </p>
    {/if}
</div>
