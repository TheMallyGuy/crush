<script lang="ts">
    import { createEventDispatcher } from 'svelte';
    import Button from '$lib/components/Button.svelte';
    import { Search, Plus, Trash2 } from '@lucide/svelte';
    import { detectType, validateValue, type FlagType } from '$lib/fastflag/flagTypes';

    export let flags: Record<string, string> = {};

    let searchQuery = '';
    let newFlagName = '';
    let newFlagValue = '';
    let addError = '';

    const dispatch = createEventDispatcher<{
        delete: string;
        update: { name: string; value: string };
        add: { name: string; value: string };
        search: string;
    }>();

    $: entries = Object.entries(flags);
    $: filteredFlags = entries.filter(([name, value]) =>
        name.toLowerCase().includes(searchQuery.toLowerCase()) ||
        value.toLowerCase().includes(searchQuery.toLowerCase())
    );

    const typeBadge: Record<FlagType, string> = {
        bool:   'bg-purple-500/15 text-purple-400 border-purple-500/30',
        int:    'bg-yellow-500/15 text-yellow-400 border-yellow-500/30',
        string: 'bg-green-500/15  text-green-400  border-green-500/30',
    };

    function handleSearch() { dispatch('search', searchQuery); }

    function handleDelete(name: string) {
        const { [name]: _, ...rest } = flags;
        flags = rest;
        dispatch('delete', name);
    }

    function handleUpdate(name: string, raw: string) {
        const type = detectType(flags[name]);
        if (!validateValue(raw, type)) return; // silently reject invalid input
        flags = { ...flags, [name]: raw };
        dispatch('update', { name, value: raw });
    }

    function handleAdd() {
        addError = '';
        if (!newFlagName.trim() || !newFlagValue.trim()) return;
        const type = detectType(newFlagValue.trim());
        if (!validateValue(newFlagValue.trim(), type)) {
            addError = 'Value does not match detected type.';
            return;
        }
        flags = { ...flags, [newFlagName.trim()]: newFlagValue.trim() };
        dispatch('add', { name: newFlagName.trim(), value: newFlagValue.trim() });
        newFlagName = '';
        newFlagValue = '';
    }

    function handleKeyDown(e: KeyboardEvent) {
        if (e.key === 'Enter') (e.target as HTMLInputElement).blur();
    }

    $: newValueType = newFlagValue.trim() ? detectType(newFlagValue.trim()) : null;
</script>

<div class="flex flex-col gap-6 w-full">
    <!-- Add row -->
    <div class="flex items-center gap-3 p-4 bg-[#0f0f0f] rounded-2xl border border-stone-800/40 shadow-sm">
        <div class="flex-[2]">
            <input
                type="text"
                bind:value={newFlagName}
                placeholder="Flag name"
                class="w-full bg-stone-900/50 border border-stone-800/50 rounded-xl px-4 py-2.5 text-sm text-stone-200 placeholder-stone-600 focus:ring-2 focus:ring-blue-500/20 focus:border-blue-500/50 outline-none transition-all"
            />
        </div>
        <div class="flex-[1] relative">
            <input
                type="text"
                bind:value={newFlagValue}
                placeholder="Value"
                class="w-full bg-stone-900/50 border border-stone-800/50 rounded-xl px-4 py-2.5 text-sm text-stone-200 placeholder-stone-600 focus:ring-2 focus:ring-blue-500/20 focus:border-blue-500/50 outline-none transition-all pr-16"
            />
            {#if newValueType}
                <span class="absolute right-3 top-1/2 -translate-y-1/2 text-[10px] font-bold uppercase px-1.5 py-0.5 rounded border {typeBadge[newValueType]}">
                    {newValueType}
                </span>
            {/if}
        </div>
        <Button
            variant="primary"
            size="md"
            class="rounded-xl h-11 px-6 shadow-lg shadow-blue-500/10 active:scale-95 transition-all"
            on:click={handleAdd}
            disabled={!newFlagName || !newFlagValue}
        >
            <Plus class="h-5 w-5 mr-2"/>
            <span class="font-semibold">Add</span>
        </Button>
    </div>
    {#if addError}
        <p class="text-red-400 text-xs px-1">{addError}</p>
    {/if}

    <div class="flex flex-col gap-4">
        <div class="relative group">
            <div class="absolute inset-y-0 left-0 pl-4 flex items-center pointer-events-none">
                <Search class="h-5 w-5 text-stone-600 group-focus-within:text-blue-500 transition-colors"/>
            </div>
            <input
                type="text"
                bind:value={searchQuery}
                on:input={handleSearch}
                placeholder="Search all fastflags..."
                class="block w-full pl-12 pr-4 py-3 border border-stone-800/40 rounded-2xl bg-[#0f0f0f] text-stone-200 placeholder-stone-600 focus:ring-2 focus:ring-blue-500/10 focus:border-stone-700 transition-all outline-none text-sm"
            />
        </div>

        <div class="flex flex-col rounded-2xl border border-stone-800/40 bg-[#0f0f0f] overflow-hidden">
            <div class="flex items-center px-6 py-3 text-[11px] font-bold uppercase tracking-[0.1em] text-stone-500 border-b border-stone-800/40 bg-stone-900/20">
                <div class="flex-[2]">Flag Name</div>
                <div class="w-20">Type</div>
                <div class="flex-[1]">Value</div>
                <div class="w-12"></div>
            </div>

            <div class="flex flex-col divide-y divide-stone-800/30">
                {#if filteredFlags.length === 0}
                    <div class="p-12 text-center text-stone-500 text-sm italic">
                        No flags found matching your search.
                    </div>
                {:else}
                    {#each filteredFlags as [name, value] (name)}
                        {@const type = detectType(value)}
                        <div class="group flex items-center px-6 py-2.5 hover:bg-stone-800/10 transition-colors duration-150">
                            <div class="flex-[2] font-mono text-[13px] text-stone-300 truncate pr-6 select-all" title={name}>
                                {name}
                            </div>
                            <div class="w-20">
                                <span class="text-[10px] font-bold uppercase px-1.5 py-0.5 rounded border {typeBadge[type]}">
                                    {type}
                                </span>
                            </div>
                            <div class="flex-[1] flex items-center">
                                {#if type === 'bool'}
                                    <select // temp for now
                                        value={value}
                                        on:change={(e) => handleUpdate(name, e.currentTarget.value)}
                                        class="w-full bg-stone-900/0 hover:bg-stone-900/40 border border-transparent hover:border-stone-800/50 focus:bg-stone-900/60 focus:border-blue-500/40 rounded-lg px-3 py-1.5 font-mono text-[13px] text-purple-400 outline-none transition-all"
                                    >
                                        <option value="true">true</option>
                                        <option value="false">false</option>
                                    </select>
                                {:else}
                                    <input
                                        type={type === 'int' ? 'number' : 'text'}
                                        value={value}
                                        on:change={(e) => handleUpdate(name, e.currentTarget.value)}
                                        on:keydown={handleKeyDown}
                                        class="w-full bg-stone-900/0 hover:bg-stone-900/40 border border-transparent hover:border-stone-800/50 focus:bg-stone-900/60 focus:border-blue-500/40 rounded-lg px-3 py-1.5 font-mono text-[13px] {type === 'int' ? 'text-yellow-400' : 'text-green-400'} outline-none transition-all"
                                    />
                                {/if}
                            </div>
                            <div class="w-12 flex justify-end">
                                <button
                                    class="p-2 text-stone-600 hover:text-red-400 hover:bg-red-400/10 rounded-xl transition-all opacity-0 group-hover:opacity-100 focus:opacity-100"
                                    on:click={() => handleDelete(name)}
                                    title="Delete Flag"
                                >
                                    <Trash2 class="h-4.5 w-4.5"/>
                                </button>
                            </div>
                        </div>
                    {/each}
                {/if}
            </div>
        </div>
    </div>
</div>