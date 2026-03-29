<script lang="ts">
    export let value: string = ''
    export let options: { value: string; label: string }[] = []
    export let placeholder: string = 'Select an option'

    let isOpen = false

    function toggle() {
        isOpen = !isOpen
    }

    function select(option: { value: string; label: string }) {
        value = option.value
        isOpen = false
    }

    function handleClickOutside(event: MouseEvent) {
        const target = event.target as HTMLElement
        if (!target.closest('.dropdown-container')) {
            isOpen = false
        }
    }
</script>

<svelte:window on:click={handleClickOutside} />

<div class="dropdown-container relative w-full max-w-[200px]">
    <button
        type="button"
        on:click|stopPropagation={toggle}
        class="flex w-full items-center justify-between rounded-lg border border-stone-800 bg-stone-900 px-4 py-2 text-sm text-stone-200 transition-all hover:border-stone-700 focus:outline-none"
    >
        <span
            >{options.find((o) => o.value === value)?.label ||
                placeholder}</span
        >
        <svg
            class="h-4 w-4 text-stone-500 transition-transform {isOpen
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
            class="absolute left-0 top-full z-50 mt-2 w-full overflow-hidden rounded-lg border border-stone-800 bg-stone-900 shadow-xl"
        >
            {#each options as option}
                <button
                    type="button"
                    on:click={() => select(option)}
                    class="w-full px-4 py-2 text-left text-sm text-stone-300 transition-colors hover:bg-stone-800 hover:text-white {value ===
                    option.value
                        ? 'bg-stone-800 text-white'
                        : ''}"
                >
                    {option.label}
                </button>
            {/each}
        </div>
    {/if}
</div>
