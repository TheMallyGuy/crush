<script lang="ts">
    import { createEventDispatcher } from 'svelte'

    export let value: string | number = ''
    export let type:
        | 'text'
        | 'password'
        | 'number'
        | 'email'
        | 'tel'
        | 'url' = 'text'
    export let placeholder = ''
    export let label = ''
    export let error = ''
    export let disabled = false
    export let id: string | undefined = undefined
    export let fullWidth = true
    export let containerClass = ''

    let className = ''
    export { className as class }

    const dispatch = createEventDispatcher()

    function handleInput(event: Event) {
        const target = event.target as HTMLInputElement
        const newValue = type === 'number' ? Number(target.value) : target.value
        value = newValue
        dispatch('input', newValue)
    }

    function handleChange(event: Event) {
        dispatch('change', value)
    }

    function handleKeyDown(event: KeyboardEvent) {
        if (event.key === 'Enter') {
            dispatch('enter', value)
        }
    }
</script>

<div
    class="flex flex-col gap-2 {fullWidth ? 'w-full' : ''} {containerClass}"
>
    {#if label}
        <label
            for={id}
            class="text-[11px] text-muted-foreground"
        >
            {label}
        </label>
    {/if}

    <div class="relative w-full">
        {#if $$slots.icon}
            <div
                class="pointer-events-none absolute inset-y-0 left-0 flex items-center pl-4 text-muted-foreground"
            >
                <slot name="icon" />
            </div>
        {/if}

        <input
            {id}
            {type}
            {placeholder}
            {disabled}
            {value}
            on:input={handleInput}
            on:change={handleChange}
            on:keydown={handleKeyDown}
            class="block w-full transition-all duration-150 outline-none text-sm rounded-xl border
                {$$slots.icon ? 'pl-11' : 'px-4'}
                py-2.5
                {disabled
                ? 'bg-muted/30 border-border text-muted-foreground cursor-not-allowed'
                : 'bg-secondary border-border text-foreground placeholder:text-muted-foreground hover:border-accent'}
                {error
                ? 'border-red-500/40 focus:ring-2 focus:ring-red-500/10 focus:border-red-500/60'
                : 'focus:ring-2 focus:ring-sapphire/20 focus:border-sapphire/40'}
                {className}"
            {...$$restProps}
        />

        {#if $$slots.action}
            <div class="absolute inset-y-0 right-0 flex items-center pr-1.5">
                <slot name="action" />
            </div>
        {/if}
    </div>

    {#if error}
        <p class="px-1 text-[11px] font-medium text-red-400/90">
            {error}
        </p>
    {/if}
</div>
