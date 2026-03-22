<script lang="ts">
    import SortableList from '$lib/components/SortableList.svelte'
    import Button from '$lib/components/Button.svelte'
    import { Trash2, Plus } from '@lucide/svelte'

    // Items MUST have a unique 'id' for svelte-dnd-action to work
    let items = [
        {
            id: '1',
            label: 'Fast Flags Optimization',
            description: 'Enable experimental performance flags',
        },
        {
            id: '2',
            label: 'Discord Rich Presence',
            description: 'Show your current game on Discord',
        },
        {
            id: '3',
            label: 'Custom Font: Outfit',
            description: 'Use the sleek Outfit typeface',
        },
        { id: '4', label: 'Unlock FPS', description: 'Removes the 60 FPS cap' },
    ]

    function handleListChange(event: CustomEvent) {
        // This is called whenever the final drop happens
        console.log('New order:', event.detail)
        items = event.detail
    }

    function removeItem(id: string) {
        items = items.filter((i) => i.id !== id)
    }

    function addItem() {
        const newId = Math.random().toString(36).substr(2, 9)
        items = [
            ...items,
            {
                id: newId,
                label: `New Mod ${items.length + 1}`,
                description: 'Custom added modification',
            },
        ]
    }

    function saveOrder() {
        alert('Order saved: ' + items.map((i) => i.label).join(', '))
    }
</script>

<div class="flex flex-col gap-8 max-w-2xl p-6">
    <div class="flex items-center justify-between">
        <div>
            <h1 class="text-3xl font-bold tracking-tight text-stone-100">
                Component Lab
            </h1>
            <p class="text-stone-400 mt-1">
                Testing the new SortableList and Button components.
            </p>
        </div>
        <Button variant="primary" size="sm" on:click={addItem}>
            <Plus slot="icon" size={16} />
            Add Item
        </Button>
    </div>

    <div class="space-y-4">
        <h2
            class="text-sm font-semibold uppercase tracking-wider text-stone-500"
        >
            Active Modifications (Drag to reorder)
        </h2>

        <SortableList {items} on:change={handleListChange} let:item>
            <!-- Content inside SortableList is passed as a slot to each Card -->
            <div class="flex items-center justify-between w-full">
                <div class="flex flex-col gap-0.5">
                    <span class="text-sm font-medium text-stone-100"
                        >{item.label}</span
                    >
                    <span class="text-xs text-stone-500"
                        >{item.description}</span
                    >
                </div>

                <Button
                    variant="ghost"
                    size="sm"
                    class="opacity-0 group-hover:opacity-100 transition-opacity !p-2"
                    on:click={() => removeItem(item.id)}
                >
                    <Trash2
                        size={16}
                        class="text-red-500/70 hover:text-red-500"
                    />
                </Button>
            </div>
        </SortableList>
    </div>

    <div class="flex gap-3 pt-4 border-t border-stone-800">
        <Button variant="primary" on:click={saveOrder}>Save Changes</Button>
        <Button variant="outline" on:click={() => (items = [...items])}
            >Reset</Button
        >
    </div>
</div>
