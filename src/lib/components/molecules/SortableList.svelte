<script lang="ts">
    import type { Snippet } from 'svelte'
    import { flip } from 'svelte/animate'
    import {
        dndzone,
        SHADOW_ITEM_MARKER_PROPERTY_NAME,
    } from 'svelte-dnd-action'
    import Card from '$lib/components/molecules/Card.svelte'
    import { GripVertical } from '@lucide/svelte'

    interface Props {
        items?: any[]
        onchange?: (items: any[]) => void
        children?: Snippet<[any]>
    }

    let { items = $bindable([]), onchange, children }: Props = $props()

    const flipDurationMs = 150

    function handleDndConsider(e: CustomEvent<{ items: any[] }>) {
        items = e.detail.items
    }
    function handleDndFinalize(e: CustomEvent<{ items: any[] }>) {
        items = e.detail.items
        onchange?.(items)
    }
</script>

<div
    class="flex flex-col gap-2 min-h-12.5"
    use:dndzone={{
        items,
        flipDurationMs,
        dropTargetStyle: {},
        morphDisabled: true,
        dropAnimationDisabled: true,
    }}
    onconsider={handleDndConsider}
    onfinalize={handleDndFinalize}
>
    {#each items as item (item.id)}
        <div
            animate:flip={{ duration: flipDurationMs }}
            class="outline-none"
            style={item[SHADOW_ITEM_MARKER_PROPERTY_NAME] ? 'opacity: 0' : ''}
        >
            <Card class="p-3! flex-row items-center gap-3">
                <div
                    class="cursor-grab active:cursor-grabbing text-stone-600 hover:text-stone-400 shrink-0"
                >
                    <GripVertical size={18} />
                </div>
                <div class="grow">
                    {#if children}
                        {@render children(item)}
                    {:else}
                        <span class="text-stone-200 font-medium"
                            >{item.label || item.id}</span
                        >
                    {/if}
                </div>
            </Card>
        </div>
    {/each}
</div>
