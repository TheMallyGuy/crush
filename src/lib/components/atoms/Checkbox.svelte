<script lang="ts">
    import { createEventDispatcher } from 'svelte'

    const dispatch = createEventDispatcher()

    export let checked = false
    export let indeterminate = false
    export let disabled = false
    export let id: string | undefined = undefined
    export let label: string = ''
    export let size: 'sm' | 'md' | 'lg' = 'md'

    let className = ''
    export { className as class }

    function toggle() {
        if (!disabled) {
            if (indeterminate) {
                indeterminate = false
                checked = true
            } else {
                checked = !checked
            }
            dispatch('change', { checked, indeterminate })
        }
    }

    function handleKeydown(e: KeyboardEvent) {
        if (e.key === ' ' || e.key === 'Enter') {
            e.preventDefault()
            toggle()
        }
    }

    const sizes = {
        sm: { box: 'w-3.5 h-3.5 rounded', icon: 9 },
        md: { box: 'w-[18px] h-[18px] rounded-[5px]', icon: 11 },
        lg: { box: 'w-[22px] h-[22px] rounded-[6px]', icon: 13 },
    }

    const labelSizes = {
        sm: 'text-[13px]',
        md: 'text-[14px]',
        lg: 'text-[15px]',
    }

    $: isActive = checked || indeterminate
</script>

<label
    class="inline-flex items-center gap-2.5 select-none
        {disabled ? 'opacity-40 cursor-not-allowed' : 'cursor-pointer'}
        {className}"
>
    <div
        role="checkbox"
        aria-checked={indeterminate ? 'mixed' : checked}
        aria-label={label}
        aria-disabled={disabled}
        tabindex={disabled ? -1 : 0}
        {id}
        on:click={toggle}
        on:keydown={handleKeydown}
        class="relative flex items-center justify-center shrink-0 border-[1.5px] transition-all duration-150
            focus:outline-none focus:ring-2 focus:ring-sapphire/50 focus:ring-offset-2 focus:ring-offset-obsidian
            {sizes[size].box}
            {isActive
                ? 'bg-sapphire border-sapphire'
                : 'bg-transparent border-stone-600 hover:border-stone-500 hover:bg-white/[0.04]'}"
    >
        <!-- Check icon -->
        <svg
            viewBox="0 0 12 12"
            fill="none"
            stroke="white"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            width={sizes[size].icon}
            height={sizes[size].icon}
            class="transition-opacity duration-100
                {checked && !indeterminate ? 'opacity-100' : 'opacity-0'}"
        >
            <polyline points="1.5,6 4.5,9 10.5,3" />
        </svg>

        {#if indeterminate}
            <svg
                viewBox="0 0 12 12"
                fill="none"
                stroke="white"
                stroke-width="2"
                stroke-linecap="round"
                width={sizes[size].icon}
                height={sizes[size].icon}
                class="absolute"
            >
                <line x1="2.5" y1="6" x2="9.5" y2="6" />
            </svg>
        {/if}
    </div>

    {#if label || $$slots.default}
        <span class="font-medium text-stone-100 leading-none {labelSizes[size]}">
            <slot>{label}</slot>
        </span>
    {/if}
</label>