<script lang="ts">
    import { createEventDispatcher } from 'svelte'
    import { flip } from 'svelte/animate'
    import { dndzone, SOURCES, TRIGGERS } from 'svelte-dnd-action'
    import Card from './Card.svelte'
    import { GripVertical } from '@lucide/svelte'

    const dispatch = createEventDispatcher()

    export let items: any[] = []

    const flipDurationMs = 200

    function handleDndConsider(e: CustomEvent<{ items: any[] }>) {
        items = e.detail.items
    }

    function handleDndFinalize(e: CustomEvent<{ items: any[] }>) {
        items = e.detail.items
        dispatch('change', items)
    }
</script>

<div
    class="flex flex-col gap-2 min-h-[50px]"
    use:dndzone={{ items, flipDurationMs, dropTargetStyle: {} }}
    on:consider={handleDndConsider}
    on:finalize={handleDndFinalize}
>
    {#each items as item (item.id)}
        <div animate:flip={{ duration: flipDurationMs }} class="outline-none">
            <Card class="!p-3 flex-row items-center gap-3">
                <div
                    class="cursor-grab active:cursor-grabbing text-stone-600 hover:text-stone-400 shrink-0"
                >
                    <GripVertical size={18} />
                </div>
                <div class="flex-grow">
                    <slot {item}>
                        <span class="text-stone-200"
                            >{item.label || item.id}</span
                        >
                    </slot>
                </div>
            </Card>
        </div>
    {/each}
</div>
