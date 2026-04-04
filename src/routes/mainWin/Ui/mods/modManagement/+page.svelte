<script lang="ts">
    import { onMount } from 'svelte'
    import Button from '$lib/components/atoms/Button.svelte'
    import {
        SquarePen,
        Trash2,
        Folder,
        Plus,
        Power,
        PowerOff,
    } from '@lucide/svelte'
    import SortableList from '$lib/components/molecules/SortableList.svelte'
    import {
        createNewMod,
        loadMods,
        deleteMod,
        renameMod,
        toggleMod,
        openModFolder,
        saveModsOrder,
    } from '$lib/mods/modManagement'
    import type { Mod } from '$lib/types'
    import { ask } from '@tauri-apps/plugin-dialog'
    import { invoke } from '@tauri-apps/api/core'
    import { _ } from 'svelte-i18n'

    let items: Mod[] = []

    onMount(async () => {
        await invoke('set_rpc', {
            details: 'A roblox boostrapper written from scratch',
            stateText: 'In Mods Route',
        })

        items = await loadMods()
    })

    async function handleNewMod() {
        const name = prompt('Enter mod name:')
        if (name) {
            const exists = items.some(
                (mod) => mod.name.toLowerCase() === name.toLowerCase()
            )
            if (exists) {
                alert(`A mod named "${name}" already exists.`)
                return
            }
            await createNewMod(name)
            items = await loadMods()
        }
    }

    async function handleDelete(id: string, name: string) {
        const confirmed = await ask(
            `Are you sure you want to delete "${name}"?`,
            {
                title: 'Delete Mod',
                kind: 'warning',
            }
        )

        if (confirmed) {
            await deleteMod(id)
            items = await loadMods()
        }
    }

    async function handleRename(id: string, currentName: string) {
        const newName = prompt('Enter new mod name:', currentName)
        if (newName && newName !== currentName) {
            await renameMod(id, newName)
            items = await loadMods()
        }
    }

    async function handleToggle(id: string) {
        await toggleMod(id)
        items = await loadMods()
    }

    async function handleOpenFolder(name: string) {
        await openModFolder(name)
    }

    async function handleSortChange(e: CustomEvent<Mod[]>) {
        items = e.detail
        await saveModsOrder(items)
    }
</script>

<div class="flex flex-col gap-8 max-w-2xl">
    <div class="flex items-center justify-between">
        <Button variant="primary" size="md" on:click={handleNewMod}>
            <Plus class="size-4 mr-2" />
            {$_("pages.mod.tab.modManagement.addModButton")}
        </Button>
    </div>

    <div class="space-y-4">
        {#if items.length === 0}
            <div
                class="p-8 border-2 border-dashed border-stone-800 rounded-xl text-center"
            >
                <p class="text-stone-500">{$_("pages.mod.tab.modManagement.noModsFound")}</p>
                <button
                    on:click={handleNewMod}
                    class="text-blue-500 hover:underline text-sm mt-2"
                    >{$_("pages.mod.tab.modManagement.noModsFoundCreateButton")}</button
                >
            </div>
        {:else}
            <SortableList {items} on:change={handleSortChange} let:item>
                <div class="flex items-center justify-between w-full pr-2">
                    <div class="flex items-center gap-4">
                        <button
                            on:click={() => handleToggle(item.id)}
                            class="p-2 rounded-lg transition-colors {item.enabled
                                ? 'bg-blue-600/20 text-blue-400 hover:bg-blue-600/30'
                                : 'bg-stone-800 text-stone-500 hover:bg-stone-700'}"
                            title={item.enabled ? $_("pages.mod.tab.modManagement.modCards.toggleNoteDisable") : $_("pages.mod.tab.modManagement.modCards.toggleNoteEnable")}
                        >
                            {#if item.enabled}
                                <Power class="size-4" />
                            {:else}
                                <PowerOff class="size-4" />
                            {/if}
                        </button>
                        <div class="flex flex-col gap-0.5">
                            <span
                                class="text-sm font-medium {item.enabled
                                    ? 'text-stone-100'
                                    : 'text-stone-500'}">{item.name}</span
                            >
                        </div>
                    </div>

                    <div class="flex items-center gap-1.5">
                        <Button
                            size="sm"
                            variant="ghost"
                            on:click={() => handleRename(item.id, item.name)}
                            title={$_("pages.mod.tab.modManagement.modCards.renameNote")}
                        >
                            <SquarePen class="size-4 text-stone-400" />
                        </Button>
                        <Button
                            size="sm"
                            variant="ghost"
                            on:click={() => handleOpenFolder(item.name)}
                            title={$_("pages.mod.tab.modManagement.modCards.openFolderNote")}
                        >
                            <Folder class="size-4 text-stone-400" />
                        </Button>
                        <Button
                            size="sm"
                            variant="ghost"
                            on:click={() => handleDelete(item.id, item.name)}
                            title={$_("pages.mod.tab.modManagement.modCards.deleteNote")}
                            class="hover:text-red-400 hover:bg-red-400/10"
                        >
                            <Trash2 class="size-4" />
                        </Button>
                    </div>
                </div>
            </SortableList>
        {/if}
    </div>

    <div class="pt-4 border-t border-stone-900">
        <p class="text-sm text-stone-500 italic">
            {$_("pages.mod.tab.modManagement.modCards.layoutNote")}
        </p>
    </div>
</div>
