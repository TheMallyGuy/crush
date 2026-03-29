<script lang="ts">
    import { createEventDispatcher } from 'svelte'
    import Button from '$lib/components/atoms/Button.svelte'
    import { Search, Plus, Trash2, Ghost } from '@lucide/svelte'
    import {
        detectType,
        validateValue,
        type FlagType,
    } from '$lib/fastflag/flagTypes'

    export let flags: Record<string, string> = {}

    let searchQuery = ''
    let newFlagName = ''
    let newFlagValue = ''
    let addError = ''

    const dispatch = createEventDispatcher<{
        delete: string
        update: { name: string; value: string }
        add: { name: string; value: string }
        search: string
    }>()

    $: entries = Object.entries(flags)
    $: filteredFlags = entries.filter(
        ([name, value]) =>
            name.toLowerCase().includes(searchQuery.toLowerCase()) ||
            value.toLowerCase().includes(searchQuery.toLowerCase())
    )

    const typeBadge: Record<FlagType, string> = {
        bool: 'bg-purple-500/10 text-purple-400 border-purple-500/20',
        int: 'bg-amber-500/10 text-amber-400 border-amber-500/20',
        string: 'bg-emerald-500/10  text-emerald-400  border-emerald-500/20',
    }

    function handleSearch() {
        dispatch('search', searchQuery)
    }

    function handleDelete(name: string) {
        const { [name]: _, ...rest } = flags
        flags = rest
        dispatch('delete', name)
    }

    function handleUpdate(name: string, raw: string) {
        const type = detectType(flags[name])
        if (!validateValue(raw, type)) return // silently reject invalid input
        flags = { ...flags, [name]: raw }
        dispatch('update', { name, value: raw })
    }

    function handleAdd() {
        addError = ''
        if (!newFlagName.trim() || !newFlagValue.trim()) return
        const type = detectType(newFlagValue.trim())
        if (!validateValue(newFlagValue.trim(), type)) {
            addError = 'Value does not match detected type.'
            return
        }
        flags = { ...flags, [newFlagName.trim()]: newFlagValue.trim() }
        dispatch('add', {
            name: newFlagName.trim(),
            value: newFlagValue.trim(),
        })
        newFlagName = ''
        newFlagValue = ''
    }

    function handleKeyDown(e: KeyboardEvent) {
        if (e.key === 'Enter') (e.target as HTMLInputElement).blur()
    }

    $: newValueType = newFlagValue.trim()
        ? detectType(newFlagValue.trim())
        : null
</script>

<div class="flex flex-col gap-6 w-full animate-in fade-in duration-500">
    <!-- Add row -->
    <div
        class="flex items-center gap-4 p-5 bg-anthracite rounded-card border border-white/5 shadow-xl"
    >
        <div class="flex-[2]">
            <input
                type="text"
                bind:value={newFlagName}
                placeholder="Flag name"
                class="w-full bg-obsidian border border-white/5 rounded-item px-4 py-3 text-sm text-stone-200 placeholder-stone-600 focus:ring-2 focus:ring-sapphire/20 focus:border-sapphire/50 outline-none transition-all"
            />
        </div>
        <div class="flex-[1] relative">
            <input
                type="text"
                bind:value={newFlagValue}
                placeholder="Value"
                class="w-full bg-obsidian border border-white/5 rounded-item px-4 py-3 text-sm text-stone-200 placeholder-stone-600 focus:ring-2 focus:ring-sapphire/20 focus:border-sapphire/50 outline-none transition-all pr-16"
            />
            {#if newValueType}
                <span
                    class="absolute right-3 top-1/2 -translate-y-1/2 text-[10px] font-bold uppercase px-2 py-0.5 rounded border {typeBadge[
                        newValueType
                    ]}"
                >
                    {newValueType}
                </span>
            {/if}
        </div>
        <Button
            variant="primary"
            size="md"
            class="rounded-item h-11 px-8 font-bold uppercase text-xs tracking-widest shadow-sapphire"
            on:click={handleAdd}
            disabled={!newFlagName || !newFlagValue}
        >
            <Plus class="h-4 w-4 mr-2" />
            Add
        </Button>
    </div>
    {#if addError}
        <p class="text-red-400 text-xs px-1 font-medium">{addError}</p>
    {/if}

    <div class="flex flex-col gap-4">
        <div class="relative group">
            <div
                class="absolute inset-y-0 left-0 pl-4 flex items-center pointer-events-none"
            >
                <Search
                    class="h-5 w-5 text-stone-600 group-focus-within:text-sapphire transition-colors"
                />
            </div>
            <input
                type="text"
                bind:value={searchQuery}
                on:input={handleSearch}
                placeholder="Search all fastflags..."
                class="block w-full pl-12 pr-4 py-4 border border-white/5 rounded-card bg-anthracite text-stone-200 placeholder-stone-600 focus:ring-2 focus:ring-sapphire/10 focus:border-white/10 transition-all outline-none text-sm shadow-inner"
            />
        </div>

        <div
            class="flex flex-col rounded-card border border-white/5 bg-anthracite overflow-hidden shadow-2xl"
        >
            <div
                class="flex items-center px-6 py-4 text-[11px] font-bold uppercase tracking-[0.2em] text-stone-500 border-b border-white/5 bg-white/2"
            >
                <div class="flex-[2]">Flag Name</div>
                <div class="w-24">Type</div>
                <div class="flex-[1]">Value</div>
                <div class="w-12"></div>
            </div>

            <div class="flex flex-col divide-y divide-white/5">
                {#if filteredFlags.length === 0}
                    <div class="flex flex-col items-center justify-center p-16 text-center gap-4 animate-in fade-in slide-in-from-bottom-4 duration-500">
                        <div class="w-16 h-16 rounded-full bg-white/5 flex items-center justify-center text-stone-600">
                             <Ghost size={32} />
                        </div>
                        <div class="flex flex-col gap-1">
                             <p class="text-stone-300 font-bold">No flags found</p>
                             <p class="text-stone-500 text-sm italic">
                                Your search returned no results. Try another query or add a new flag.
                             </p>
                        </div>
                    </div>
                {:else}
                    {#each filteredFlags as [name, value], i (name)}
                        {@const type = detectType(value)}
                        <div
                            class="group flex items-center px-6 py-3.5 hover:bg-white/5 transition-colors duration-200 animate-in fade-in slide-in-from-left-2"
                            style="animation-delay: {i * 20}ms"
                        >
                            <div
                                class="flex-[2] font-mono text-[13px] text-stone-300 truncate pr-6 select-all group-hover:text-sapphire transition-colors"
                                title={name}
                            >
                                {name}
                            </div>
                            <div class="w-24">
                                <span
                                    class="text-[10px] font-bold uppercase px-2 py-0.5 rounded border {typeBadge[
                                        type
                                    ]}"
                                >
                                    {type}
                                </span>
                            </div>
                            <div class="flex-[1] flex items-center">
                                {#if type === 'bool'}
                                    <select
                                        {value}
                                        on:change={(e) =>
                                            handleUpdate(
                                                name,
                                                e.currentTarget.value
                                            )}
                                        class="w-full bg-obsidian border border-white/5 hover:border-sapphire/30 focus:border-sapphire/50 rounded-item px-3 py-2 font-mono text-[13px] text-purple-400 outline-none transition-all cursor-pointer"
                                    >
                                        <option value="true">true</option>
                                        <option value="false">false</option>
                                    </select>
                                {:else}
                                    <input
                                        type={type === 'int'
                                            ? 'number'
                                            : 'text'}
                                        {value}
                                        on:change={(e) =>
                                            handleUpdate(
                                                name,
                                                e.currentTarget.value
                                            )}
                                        on:keydown={handleKeyDown}
                                        class="w-full bg-obsidian border border-white/5 hover:border-sapphire/30 focus:border-sapphire/50 rounded-item px-3 py-2 font-mono text-[13px] {type ===
                                        'int'
                                            ? 'text-amber-400'
                                            : 'text-emerald-400'} outline-none transition-all"
                                    />
                                {/if}
                            </div>
                            <div class="w-12 flex justify-end">
                                <button
                                    class="p-2 text-stone-600 hover:text-red-400 hover:bg-red-400/10 rounded-item transition-all opacity-0 group-hover:opacity-100 focus:opacity-100"
                                    on:click={() => handleDelete(name)}
                                    title="Delete Flag"
                                >
                                    <Trash2 class="h-4.5 w-4.5" />
                                </button>
                            </div>
                        </div>
                    {/each}
                {/if}
            </div>
        </div>
    </div>
</div>

<style>
    @keyframes in {
        from {
            opacity: 0;
            transform: translateY(10px);
        }
        to {
            opacity: 1;
            transform: translateY(0);
        }
    }

    @keyframes fade-in {
        from { opacity: 0; }
        to { opacity: 1; }
    }

    @keyframes slide-in-from-left {
        from { transform: translateX(-10px); }
        to { transform: translateX(0); }
    }

    .animate-in {
        animation-duration: 400ms;
        animation-timing-function: cubic-bezier(0.16, 1, 0.3, 1);
        animation-fill-mode: forwards;
    }

    .fade-in { animation-name: fade-in; }
    .slide-in-from-bottom-4 { animation-name: in; }
    .slide-in-from-left-2 { animation-name: slide-in-from-left; }
</style>
