<script lang="ts">
    import { createEventDispatcher } from 'svelte'
    import { spring } from 'svelte/motion'

    const dispatch = createEventDispatcher<{ change: string }>()

    export let tabs: { label: string; value: string; icon?: any }[] = []
    export let activeTab: string = tabs[0]?.value || ''

    let tabElements: HTMLButtonElement[] = []
    let containerEl: HTMLDivElement

    const pillStyle = spring(
        { x: 0, width: 0 },
        { stiffness: 0.3, damping: 0.75 }
    )

    function updatePill(index: number) {
        const btn = tabElements[index]
        const container = containerEl
        if (!btn || !container) return
        const containerRect = container.getBoundingClientRect()
        const btnRect = btn.getBoundingClientRect()
        pillStyle.set({
            x: btnRect.left - containerRect.left - 4,
            width: btnRect.width,
        })
    }

    function selectTab(value: string, index: number) {
        activeTab = value
        updatePill(index)
        dispatch('change', value)
    }

    import { tick } from 'svelte'
    $: if (tabElements.length && containerEl) {
        tick().then(() => {
            const idx = tabs.findIndex(t => t.value === activeTab)
            if (idx !== -1) updatePill(idx)
        })
    }
</script>

<div class="flex flex-col gap-6 w-full">
    <div
        bind:this={containerEl}
        class="relative flex items-center gap-1 p-1 bg-stone-900/60 backdrop-blur-md  border border-white/5 w-fit self-center sm:self-start"
    >
        <div
            class="absolute top-1 bottom-1 bg-sapphire pointer-events-none transition-none"
            style="
                transform: translateX({$pillStyle.x}px);
                width: {$pillStyle.width}px;
                z-index: 0;
            "
        ></div>

        {#each tabs as tab, i}
            <button
                bind:this={tabElements[i]}
                type="button"
                class="relative flex items-center justify-center gap-2 px-5 py-2 text-sm font-semibold z-10 cursor-target
                    transition-colors duration-200 group
                    {activeTab === tab.value
                        ? 'text-white'
                        : 'text-stone-400 hover:text-stone-200 hover:bg-white/5'}"
                on:click={() => selectTab(tab.value, i)}
            >
                {#if tab.icon}
                    <svelte:component
                        this={tab.icon}
                        size={16}
                        class="shrink-0 transition-transform duration-200
                            {activeTab === tab.value
                                ? 'scale-110 rotate-0'
                                : 'scale-100 group-hover:scale-110'}"
                    />
                {/if}
                <span
                    class="transition-all duration-200
                        {activeTab === tab.value
                            ? 'tracking-wide'
                            : 'tracking-normal'}"
                >{tab.label}</span>
            </button>
        {/each}
    </div>

    <div class="w-full">
        <slot {activeTab} />
    </div>
</div>