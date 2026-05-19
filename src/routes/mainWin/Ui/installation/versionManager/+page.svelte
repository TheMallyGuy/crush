<script lang="ts">
    import Button from '$lib/components/atoms/Button.svelte'
    import SortableList from '$lib/components/molecules/SortableList.svelte'
    import { Trash2 } from '@lucide/svelte'

    type Versions = {
        id: string
        currentUsing: string
    }

    let items: Versions[] = [
        {
            id: 'New Installation 1',
            currentUsing: 'version-67',
        },
        {
            id: 'New Installation 2',
            currentUsing: 'version-69',
        },
    ]
</script>

<div class="flex flex-col gap-8">
    <div class="flex items-center justify-between">
        <div>
            <h1 class="text-3xl font-bold tracking-tight text-stone-100">
                Installation manager
            </h1>
            <p class="text-stone-400 mt-1">Easily manage your roblox versions.</p>
        </div>
    </div>
    <div class="space-y-4">
        {#if items.length === 0}
            <div class="p-8 border-2 border-dashed border-stone-800 rounded-xl text-center">
                <p class="text-stone-500">No installations yet.</p>
            </div>
        {:else}
            <SortableList {items} let:item>
                <div class="flex items-center justify-between w-full pr-2">
                    <div class="flex flex-col gap-0.5">
                        <span class="text-sm font-medium text-stone-100">{item.id}</span>
                        <span class="text-xs text-stone-400">Using: {item.currentUsing}</span>
                    </div>
                    <div class="flex items-center gap-1.5">
                        <Button
                            size="sm"
                            variant="ghost"
                            class="hover:text-red-400 hover:bg-red-400/10"
                        >
                            <Trash2 class="size-4" />
                        </Button>
                    </div>
                </div>
            </SortableList>
        {/if}
    </div>
</div>