<script lang="ts">
    import { createEventDispatcher } from 'svelte'
    import Button from '$lib/components/atoms/Button.svelte'
    import { Search, Plus, Trash2, Upload } from '@lucide/svelte'
    import { _ } from 'svelte-i18n';
    import { detectType, validateValue } from '$lib/fastflag/flagTypes'
    import type { FlagType } from '$lib/types'

    export let flags: Record<string, string> = {}

    let searchQuery = ''
    let newFlagName = ''
    let newFlagValue = ''
    let addError = ''
    let jsonFileInput: HTMLInputElement

    const dispatch = createEventDispatcher<{
        delete: string
        update: { name: string; value: string }
        add: { name: string; value: string }
        search: string
        import: Record<string, string>
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
        if (!validateValue(raw, type)) return
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

    function handleImportClick() {
        jsonFileInput.click()
    }

    function handleFileChange(e: Event) {
        addError = ''
        const file = (e.target as HTMLInputElement).files?.[0]
        if (!file) return
        const reader = new FileReader()
        reader.onload = (ev) => {
            try {
                const parsed = JSON.parse(ev.target?.result as string)
                if (typeof parsed !== 'object' || Array.isArray(parsed)) {
                    addError = 'JSON must be a flat key-value object.'
                    return
                }
                const imported: Record<string, string> = {}
                for (const [k, v] of Object.entries(parsed)) {
                    imported[k] = String(v)
                }
                flags = { ...flags, ...imported }
                dispatch('import', imported)
            } catch {
                addError = 'Invalid JSON file.'
            }
        }
        reader.readAsText(file)
        ;(e.target as HTMLInputElement).value = ''
    }

    $: newValueType = newFlagValue.trim()
        ? detectType(newFlagValue.trim())
        : null
</script>

<div class="flex flex-col gap-4 w-full">
    <!-- Compact add row + import -->
    <div class="flex items-center gap-2">
        <input
            type="text"
            bind:value={newFlagName}
            placeholder={$_('pages.fastflag.editor.flagTable.flagCol.name')}
            on:keydown={(e) => e.key === 'Enter' && handleAdd()}
            class="flex-[2] min-w-0 bg-stone-900/50 border border-stone-800/40 rounded-xl px-3 py-2 text-sm text-stone-200 placeholder-stone-600 focus:ring-2 focus:ring-sapphire/20 focus:border-sapphire/40 outline-none transition-all duration-150"
        />
        <div class="flex-[1] min-w-0 relative">
            <input
                type="text"
                bind:value={newFlagValue}
                placeholder={$_('pages.fastflag.editor.flagTable.flagCol.value')}
                on:keydown={(e) => e.key === 'Enter' && handleAdd()}
                class="w-full bg-stone-900/50 border border-stone-800/40 rounded-xl px-3 py-2 text-sm text-stone-200 placeholder-stone-600 focus:ring-2 focus:ring-sapphire/20 focus:border-sapphire/40 outline-none transition-all duration-150 {newValueType ? 'pr-14' : ''}"
            />
            {#if newValueType}
                <span
                    class="absolute right-2.5 top-1/2 -translate-y-1/2 text-[10px] font-bold uppercase px-1.5 py-0.5 rounded border {typeBadge[newValueType]}"
                >
                    {newValueType}
                </span>
            {/if}
        </div>
        <Button
            variant="primary"
            size="sm"
            class="rounded-xl shrink-0"
            on:click={handleAdd}
            disabled={!newFlagName || !newFlagValue}
        >
            <Plus class="h-4 w-4 mr-1" />
            <span class="font-semibold text-sm">{$_('pages.fastflag.editor.flagTable.buttonAdd')}</span>
        </Button>

        <!-- Divider -->
        <div class="w-px h-6 bg-stone-800/60 shrink-0"></div>

        <!-- Import from JSON -->
        <input
            type="file"
            accept=".json,application/json"
            class="hidden"
            bind:this={jsonFileInput}
            on:change={handleFileChange}
        />
        <button
            on:click={handleImportClick}
            class="flex items-center gap-1.5 px-3 py-2 rounded-xl text-sm font-semibold text-stone-400 hover:text-stone-200 bg-stone-900/0 hover:bg-stone-800/60 border border-stone-800/40 hover:border-stone-700/60 transition-all duration-150 shrink-0"
            title="Import flags from a JSON file"
        >
            <Upload class="h-4 w-4" />
            <span>Import JSON</span>
        </button>
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
                placeholder={$_('pages.fastflag.editor.flagTable.search')}
                class="block w-full pl-12 pr-4 py-3 border border-stone-800/20 rounded-2xl bg-anthracite/40 backdrop-blur-sm text-stone-200 placeholder-stone-600 focus:ring-2 focus:ring-sapphire/10 focus:border-stone-700/60 transition-all duration-150 outline-none text-sm"
            />
        </div>

        <div
            class="flex flex-col rounded-2xl border border-stone-800/20 bg-anthracite/40 backdrop-blur-sm overflow-hidden"
        >
            <div
                class="flex items-center px-6 py-3 text-[11px] font-bold uppercase tracking-widest text-stone-500 border-b border-stone-800/20 bg-stone-900/60"
            >
                <div class="flex-[2]">{$_('pages.fastflag.editor.flagTable.flagCol.name')}</div>
                <div class="w-20">{$_('pages.fastflag.editor.flagTable.flagCol.type')}</div>
                <div class="flex-[1]">{$_('pages.fastflag.editor.flagTable.flagCol.value')}</div>
                <div class="w-12"></div>
            </div>

            <div class="flex flex-col divide-y divide-stone-800/10">
                {#if filteredFlags.length === 0}
                    <div class="p-12 text-center text-stone-500 text-sm italic">
                        {$_('pages.fastflag.editor.flagTable.searchNotFound')}
                    </div>
                {:else}
                    {#each filteredFlags as [name, value] (name)}
                        {@const type = detectType(value)}
                        <div
                            class="group flex items-center px-6 py-2.5 hover:bg-stone-800/40 transition-colors duration-150"
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
                                    title={$_('pages.fastflag.editor.flagTable.flagCol.deleteNote')}
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