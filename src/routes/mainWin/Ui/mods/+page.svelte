<script lang='ts'>
    import Button from "$lib/components/Button.svelte";
    import Card from "$lib/components/Card.svelte";
    import SortableList from "$lib/components/SortableList.svelte";

     let items = [
        { id: '1', label: 'fih', description: 'fih' },
        { id: '2', label: 'tung tung sahur', description: 'brahh' },
        { id: '3', label: 'something idfk', description: 'me happy' },
    ]
</script>




<div class="flex flex-col gap-8 max-w-2xl">
    <div>
        <h1 class="text-3xl font-bold tracking-tight text-stone-100">Mods</h1>
        <p class="text-stone-400 mt-1">Manage your roblox mods</p>
    </div>
    <Button variant="outline", size="md"> Add a mod </Button>

    <div>
        <SortableList items={items}>

        </SortableList>
    </div>

    <p class="text-sm text-stone-400">Mods layout inspired by Frostrap's mods manager</p>
</div>