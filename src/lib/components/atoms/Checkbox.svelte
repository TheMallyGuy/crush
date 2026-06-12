<script lang="ts">
    import type { Snippet } from 'svelte'

    interface Props {
        checked?: boolean
        indeterminate?: boolean
        disabled?: boolean
        id?: string
        label?: string
        size?: 'sm' | 'md' | 'lg'
        class?: string
        onchange?: (state: { checked: boolean; indeterminate: boolean }) => void
        children?: Snippet
    }

    let {
        checked = $bindable(false),
        indeterminate = $bindable(false),
        disabled = false,
        id = undefined,
        label = '',
        size = 'md',
        class: className = '',
        onchange,
        children,
    }: Props = $props()

    function toggle() {
        if (!disabled) {
            if (indeterminate) {
                indeterminate = false
                checked = true
            } else {
                checked = !checked
            }
            onchange?.({ checked, indeterminate })
        }
    }

    function handleKeydown(e: KeyboardEvent) {
        if (e.key === ' ' || e.key === 'Enter') {
            e.preventDefault()
            toggle()
        }
    }

    const sizes = {
        sm: { box: 'w-3.5 h-3.5', icon: 9 },
        md: { box: 'w-[18px] h-[18px]', icon: 11 },
        lg: { box: 'w-[22px] h-[22px]', icon: 13 },
    }

    const labelSizes = {
        sm: 'text-[13px]',
        md: 'text-[14px]',
        lg: 'text-[15px]',
    }

    let isActive = $derived(checked || indeterminate)
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
        onclick={toggle}
        onkeydown={handleKeydown}
        class="relative flex items-center justify-center shrink-0 border-[1.5px] transition-all duration-150
            focus:outline-none focus:ring-2 focus:ring-sapphire/50 focus:ring-offset-2 focus:ring-offset-obsidian cursor-target
            {sizes[size].box}
            {isActive
                ? 'bg-sapphire border-sapphire'
                : 'bg-stone-900 border-stone-600 hover:border-stone-500 hover:bg-stone-800'}"
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

    {#if label || children}
        <span class="font-medium text-stone-100 leading-none {labelSizes[size]}">
            {#if children}{@render children()}{:else}{label}{/if}
        </span>
    {/if}
</label>
