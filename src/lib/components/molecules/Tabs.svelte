<script lang="ts">
    import { createEventDispatcher } from 'svelte'

    const dispatch = createEventDispatcher<{ change: string }>()

    export let tabs: { label: string; value: string; icon?: any }[] = []
    export let activeTab: string = tabs[0]?.value || ''

    function selectTab(value: string) {
        activeTab = value
        dispatch('change', value)
    }
</script>

<div class="flex flex-col gap-6 w-full">
    <div
        class="flex items-center gap-1 p-1 bg-muted/60 backdrop-blur-md rounded-2xl border border-border w-fit self-center sm:self-start"
    >
        {#each tabs as tab}
            <button
                type="button"
                class="relative flex items-center justify-center gap-2 px-5 py-2 rounded-xl text-sm font-semibold transition-all duration-200 group
                {activeTab === tab.value
                    ? 'text-white'
                    : 'text-muted-foreground hover:text-foreground hover:bg-accent'}"
                on:click={() => selectTab(tab.value)}
            >
                {#if activeTab === tab.value}
                    <div
                        class="absolute inset-0 bg-sapphire rounded-xl -z-10"
                    ></div>
                {/if}

                {#if tab.icon}
                    <svelte:component 
                        this={tab.icon} 
                        size={16} 
                        class="shrink-0 transition-transform duration-200 {activeTab === tab.value ? 'scale-110' : 'group-hover:scale-110'}" 
                    />
                {/if}
                <span>{tab.label}</span>
            </button>
        {/each}
    </div>

    <div class="w-full">
        <slot {activeTab} />
    </div>
</div>
