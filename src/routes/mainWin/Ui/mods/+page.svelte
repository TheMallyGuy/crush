<script lang="ts">
    import { onMount } from "svelte";
    import Button from "$lib/components/Button.svelte";
    import { SquarePen, Trash2, Folder, Plus, Power, PowerOff } from "@lucide/svelte";
    import SortableList from "$lib/components/SortableList.svelte";
    import { createNewMod, loadMods, deleteMod, renameMod, toggleMod, openModFolder, saveModsOrder, type Mod } from "$lib/mods/modManagement";
    import { ask } from "@tauri-apps/plugin-dialog";

    let items: Mod[] = [];

    onMount(async () => {
        items = await loadMods();
    });

    async function handleNewMod() {
        const name = prompt("Enter mod name:");
        if (name) {
            await createNewMod(name);
            items = await loadMods();
        }
    }

    async function handleDelete(id: string, name: string) {
        const confirmed = await ask(`Are you sure you want to delete "${name}"?`, {
            title: 'Delete Mod',
            kind: 'warning',
        });
        
        if (confirmed) {
            await deleteMod(id);
            items = await loadMods();
        }
    }

    async function handleRename(id: string, currentName: string) {
        const newName = prompt("Enter new mod name:", currentName);
        if (newName && newName !== currentName) {
            await renameMod(id, newName);
            items = await loadMods();
        }
    }

    async function handleToggle(id: string) {
        await toggleMod(id);
        items = await loadMods();
    }

    async function handleOpenFolder(name: string) {
        await openModFolder(name);
    }

    async function handleSortChange(e: CustomEvent<Mod[]>) {
        items = e.detail;
        await saveModsOrder(items);
    }
</script>

<div class="flex flex-col gap-8 max-w-2xl">
    <div class="flex items-center justify-between">
        <div>
            <h1 class="text-3xl font-bold tracking-tight text-stone-100">Mods</h1>
            <p class="text-stone-400 mt-1">Manage your Roblox mods</p>
        </div>
        <Button variant="primary" size="md" on:click={handleNewMod}>
            <Plus class="size-4 mr-2" />
            Add a mod
        </Button>
    </div>

    <div class="space-y-4">
        {#if items.length === 0}
            <div class="p-8 border-2 border-dashed border-stone-800 rounded-xl text-center">
                <p class="text-stone-500">No mods installed yet.</p>
                <button on:click={handleNewMod} class="text-blue-500 hover:underline text-sm mt-2">Create your first mod</button>
            </div>
        {:else}
            <SortableList items={items} on:change={handleSortChange} let:item>
                <div class="flex items-center justify-between w-full pr-2">
                    <div class="flex items-center gap-4">
                        <button 
                            on:click={() => handleToggle(item.id)}
                            class="p-2 rounded-lg transition-colors {item.enabled ? 'bg-blue-600/20 text-blue-400 hover:bg-blue-600/30' : 'bg-stone-800 text-stone-500 hover:bg-stone-700'}"
                            title={item.enabled ? 'Disable mod' : 'Enable mod'}
                        >
                            {#if item.enabled}
                                <Power class="size-4" />
                            {:else}
                                <PowerOff class="size-4" />
                            {/if}
                        </button>
                        <div class="flex flex-col gap-0.5">
                            <span class="text-sm font-medium {item.enabled ? 'text-stone-100' : 'text-stone-500'}">{item.name}</span>
                        </div>
                    </div>
                    
                    <div class="flex items-center gap-1.5">
                        <Button size="sm" variant="ghost" on:click={() => handleRename(item.id, item.name)} title="Rename">
                            <SquarePen class="size-4 text-stone-400"/>
                        </Button> 
                        <Button size="sm" variant="ghost" on:click={() => handleOpenFolder(item.name)} title="Open Folder">
                            <Folder class="size-4 text-stone-400"/>
                        </Button> 
                        <Button size="sm" variant="ghost" on:click={() => handleDelete(item.id, item.name)} title="Delete" class="hover:text-red-400 hover:bg-red-400/10">
                            <Trash2 class="size-4"/>
                        </Button>
                    </div>
                </div>
            </SortableList>
        {/if}
    </div>

    <div class="pt-4 border-t border-stone-900">
        <p class="text-sm text-stone-500 italic">Mods layout inspired by Frostrap's mods manager</p>
    </div>
</div>