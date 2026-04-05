<script lang="ts">
    import { createEventDispatcher } from 'svelte'
    import Button from '$lib/components/atoms/Button.svelte'
    import { Search, Plus, Trash2 } from '@lucide/svelte'
    import { _ } from 'svelte-i18n';
    import { detectType, validateValue } from '$lib/fastflag/flagTypes'
    import type { FlagType } from '$lib/types'

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
        int: 'bg-yellow-500/10 text-yellow-400 border-yellow-500/20',
        string: 'bg-emerald-500/10 text-emerald-400 border-emerald-500/20',
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

<div class="flex flex-col gap-6 w-full">
    <!-- Add row -->
    <div
        class="flex items-center gap-3 p-4 bg-anthracite rounded-2xl border border-stone-800/20 shadow-sm"
    >
        <div class="flex-[2]">
            <input
                type="text"
                bind:value={newFlagName}
                placeholder={$_('pages.fastflag.flagTable.flagCol.name')}
                class="w-full bg-stone-900/50 border border-stone-800/40 rounded-xl px-4 py-2.5 text-sm text-stone-200 placeholder-stone-600 focus:ring-2 focus:ring-sapphire/20 focus:border-sapphire/40 outline-none transition-all duration-150"
            />
        </div>
        <div class="flex-[1] relative">
            <input
                type="text"
                bind:value={newFlagValue}
                placeholder={$_('pages.fastflag.flagTable.flagCol.value')}
                class="w-full bg-stone-900/50 border border-stone-800/40 rounded-xl px-4 py-2.5 text-sm text-stone-200 placeholder-stone-600 focus:ring-2 focus:ring-sapphire/20 focus:border-sapphire/40 outline-none transition-all duration-150 pr-16"
            />
            {#if newValueType}
                <span
                    class="absolute right-3 top-1/2 -translate-y-1/2 text-[10px] font-bold uppercase px-1.5 py-0.5 rounded border {typeBadge[
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
            class="rounded-xl h-11 px-6 shadow-glow-sapphire transition-all duration-150"
            on:click={handleAdd}
            disabled={!newFlagName || !newFlagValue}
        >
            <Plus class="h-5 w-5 mr-2" />
            <span class="font-bold">{$_('pages.fastflag.flagTable.buttonAdd')}</span>
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
                    class="h-5 w-5 text-stone-600 group-focus-within:text-sapphire transition-colors duration-150"
                />
            </div>
            <input
                type="text"
                bind:value={searchQuery}
                on:input={handleSearch}
                placeholder={$_('pages.fastflag.flagTable.search')}
                class="block w-full pl-12 pr-4 py-3 border border-stone-800/20 rounded-2xl bg-anthracite text-stone-200 placeholder-stone-600 focus:ring-2 focus:ring-sapphire/10 focus:border-stone-700/60 transition-all duration-150 outline-none text-sm"
            />
        </div>

        <div
            class="flex flex-col rounded-2xl border border-stone-800/20 bg-anthracite overflow-hidden"
        >
            <div
                class="flex items-center px-6 py-3 text-[11px] font-bold uppercase tracking-widest text-stone-500 border-b border-stone-800/20 bg-stone-900/40"
            >
                <div class="flex-[2]">{$_('pages.fastflag.flagTable.flagCol.name')}</div>
                <div class="w-20">{$_('pages.fastflag.flagTable.flagCol.type')}</div>
                <div class="flex-[1]">{$_('pages.fastflag.flagTable.flagCol.value')}</div>
                <div class="w-12"></div>
            </div>

            <div class="flex flex-col divide-y divide-stone-800/10">
                {#if filteredFlags.length === 0}
                    <div class="p-12 text-center text-stone-500 text-sm italic">
                        {$_('pages.fastflag.flagTable.searchNotFound')}
                    </div>
                {:else}
                    {#each filteredFlags as [name, value] (name)}
                        {@const type = detectType(value)}
                        <div
                            class="group flex items-center px-6 py-2.5 hover:bg-stone-800/30 transition-colors duration-150"
                        >
                            <div
                                class="flex-[2] font-mono text-[13px] text-stone-300 truncate pr-6 select-all"
                                title={name}
                            >
                                {name}
                            </div>
                            <div class="w-20">
                                <span
                                    class="text-[10px] font-bold uppercase px-1.5 py-0.5 rounded border {typeBadge[
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
                                        class="w-full bg-stone-900/0 hover:bg-stone-900/40 border border-transparent hover:border-stone-800/50 focus:bg-stone-900/60 focus:border-sapphire/40 rounded-lg px-3 py-1.5 font-mono text-[13px] text-purple-400 outline-none transition-all duration-150"
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
                                        class="w-full bg-stone-900/0 hover:bg-stone-900/40 border border-transparent hover:border-stone-800/50 focus:bg-stone-900/60 focus:border-sapphire/40 rounded-lg px-3 py-1.5 font-mono text-[13px] {type ===
                                        'int'
                                            ? 'text-yellow-400'
                                            : 'text-emerald-400'} outline-none transition-all duration-150"
                                    />
                                {/if}
                            </div>
                            <div class="w-12 flex justify-end">
                                <button
                                    class="p-2 text-stone-600 hover:text-red-400 hover:bg-red-400/10 rounded-xl transition-all duration-150 opacity-0 group-hover:opacity-100 focus:opacity-100"
                                    on:click={() => handleDelete(name)}
                                    title={$_('pages.fastflag.flagTable.flagCol.deleteNote')}
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
