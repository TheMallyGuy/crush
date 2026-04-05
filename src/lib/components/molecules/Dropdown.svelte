<script lang="ts">
    import { createEventDispatcher } from 'svelte'
    export let value: string = ''
    export let options: { value: string; label: string }[] = []
    export let placeholder: string = 'Select an option'
    let isOpen = false
    const dispatch = createEventDispatcher()
    function toggle() {
        isOpen = !isOpen
    }
    function select(option: { value: string; label: string }) {
        value = option.value
        isOpen = false
        dispatch('change', value)
    }
    function handleClickOutside(event: MouseEvent) {
        const target = event.target as HTMLElement
        if (!target.closest('.dropdown-container')) {
            isOpen = false
        }
    }
</script>
<svelte:window on:click={handleClickOutside} />

<style>
    .scrollbar-hide {
        scrollbar-width: none;
        -ms-overflow-style: none;
    }
    .scrollbar-hide::-webkit-scrollbar {
        display: none;
    }
</style>
<div class="dropdown-container relative w-full max-w-50">
    <button
        type="button"
        on:click|stopPropagation={toggle}
        class="flex w-full items-center justify-between rounded-lg border border-stone-800/40 bg-stone-900/50 px-4 py-2 text-sm text-stone-200 transition-all duration-150 hover:border-stone-700/60 focus:outline-none focus:ring-2 focus:ring-sapphire/20"
    >
        <span class="font-medium"
            >{options.find((o) => o.value === value)?.label ||
                placeholder}</span
        >
        <svg
            class="h-4 w-4 text-stone-500 transition-transform duration-150 {isOpen
                ? 'rotate-180'
                : ''}"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
        >
            <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M19 9l-7 7-7-7"
            />
        </svg>
    </button>
    {#if isOpen}
        <div
            class="absolute left-0 top-full z-50 mt-2 w-full rounded-lg border border-stone-800/40 bg-stone-900 shadow-2xl transition-all scrollbar-hide {options.length > 3
                ? 'max-h-40 overflow-y-auto'
                : 'overflow-hidden'}"
        >
            {#each options as option}
                <button
                    type="button"
                    on:click={() => select(option)}
                    class="w-full px-4 py-2.5 text-left text-sm text-stone-400 transition-colors duration-150 hover:bg-stone-800/50 hover:text-stone-100 {value ===
                    option.value
                        ? 'bg-stone-800/80 text-sapphire font-semibold'
                        : ''}"
                >
                    {option.label}
                </button>
            {/each}
        </div>
    {/if}
</div>