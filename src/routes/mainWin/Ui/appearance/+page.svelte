<script lang="ts">
    import {
        loadThemeFromDialog,
        saveActiveTheme,
        listThemes,
        loadThemeFromAppData,
        removeTheme,
    } from '$lib/theme/themeLoader'
    import { themeStore } from '$lib/theme/themeStore'
    import { onMount } from 'svelte'
    import { Brush, Trash2, Plus, Check, RotateCcw } from '@lucide/svelte'
    import ExpandableSettingCard from '$lib/components/molecules/ExpandableSettingCard.svelte'
    import Dropdown from '$lib/components/molecules/Dropdown.svelte'
    import Button from '$lib/components/atoms/Button.svelte'
    import { invoke } from '@tauri-apps/api/core'

    type State = 'idle' | 'loading' | 'error'

    let state: State = 'idle'
    let error = ''
    let missing: string[] = []
    let activeName = ''
    let themes: string[] = []

    let themeType = 'default'
    const typeOptions = [
        { value: 'default', label: 'Default' },
        { value: 'custom', label: 'Custom' },
    ]

    let isInitialized = false

    async function refreshThemes() {
        themes = await listThemes()
    }

    async function pick() {
        state = 'loading'
        error = ''
        missing = []

        try {
            const result = await loadThemeFromDialog()
            if (!result) {
                if (!activeName) themeType = 'default'
                state = 'idle'
                return
            }

            await saveActiveTheme(result.themeName)
            activeName = result.themeName
            missing = result.missing
            themeType = 'custom'
            await refreshThemes()
            state = 'idle'
        } catch (e: any) {
            error = e.message ?? 'Unknown error'
            state = 'error'
            themeType = 'default'
        }
    }

    async function selectTheme(name: string) {
        state = 'loading'
        try {
            const result = await loadThemeFromAppData(name)
            if (result) {
                await saveActiveTheme(name)
                activeName = name
                missing = result.missing
            }
        } catch (e: any) {
            error = e.message ?? 'Unknown error'
        } finally {
            state = 'idle'
        }
    }

    async function delTheme(name: string) {
        if (activeName === name) {
            themeStore.set(null)
            await saveActiveTheme(null)
            activeName = ''
            themeType = 'default'
        }
        await removeTheme(name)
        await refreshThemes()
    }

    $: if (isInitialized && themeType) {
        if (themeType === 'default' && activeName) {
            themeStore.set(null)
            saveActiveTheme(null)
            activeName = ''
        }
    }

    onMount(() => {
        invoke('set_rpc', {
            details: 'A roblox boostrapper written from scratch',
            stateText: 'In Appearance Route',
        })

        refreshThemes()
        const unsub = themeStore.subscribe((v) => {
            activeName = v?.themeName || ''
            const newType = v ? 'custom' : 'default'
            if (!isInitialized) {
                themeType = newType
                isInitialized = true
            } else {
                if (themeType !== newType) themeType = newType
            }
        })
        return unsub
    })
</script>

<div class="flex flex-col gap-8 max-w-2xl">
    <div>
        <h1 class="text-3xl font-bold tracking-tight text-stone-100">
            Appearance
        </h1>
        <p class="text-stone-400 mt-1">
            Customize the look and feel of the Roblox bootstrapper.
        </p>
    </div>

    <div class="grid gap-6">
        <ExpandableSettingCard
            title="Bootstrapper Theme"
            description="Choose between the default Crush theme or a custom XML theme."
            isOpen={themeType === 'custom'}
        >
            <div slot="icon">
                <Brush />
            </div>

            <div slot="action">
                {#if themeType === 'custom'}
                    <Button 
                        variant="ghost" 
                        size="sm" 
                        class="h-8 w-8 !p-0"
                        on:click={(e) => {
                            e.stopPropagation();
                            themeType = 'default';
                        }}
                    >
                        <RotateCcw size={16} />
                    </Button>
                {/if}
            </div>

            <div class="flex flex-col gap-4">
                <div class="flex items-center justify-between gap-3">
                    <Dropdown bind:value={themeType} options={typeOptions} />

                    {#if themeType === 'custom'}
                        <button
                            on:click={pick}
                            class="flex items-center gap-1.5 text-xs text-stone-500 hover:text-stone-300 transition-colors uppercase tracking-wider font-semibold whitespace-nowrap"
                        >
                            <Plus size={14} />
                            Import New
                        </button>
                    {/if}
                </div>

                {#if themeType === 'custom'}
                    <div class="flex flex-col gap-2">
                        {#if themes.length === 0}
                            <p class="text-xs text-stone-500 italic py-2">
                                No custom themes imported yet.
                            </p>
                        {:else}
                            <div class="grid gap-2">
                                {#each themes as theme}
                                    <div
                                        class="group flex items-center justify-between px-3 py-2 bg-stone-900/50 border {activeName ===
                                        theme
                                            ? 'border-stone-700 bg-stone-800/50'
                                            : 'border-stone-800'} rounded-lg transition-all"
                                    >
                                        <button
                                            on:click={() => selectTheme(theme)}
                                            class="flex items-center gap-3 flex-grow text-left"
                                        >
                                            <div
                                                class="w-2 h-2 rounded-full {activeName ===
                                                theme
                                                    ? 'bg-green-500'
                                                    : 'bg-stone-700'}"
                                            ></div>
                                            <span
                                                class="text-sm {activeName ===
                                                theme
                                                    ? 'text-stone-100 font-medium'
                                                    : 'text-stone-400'}"
                                                >{theme}</span
                                            >
                                        </button>

                                        <div class="flex items-center gap-1">
                                            {#if activeName === theme}
                                                <div
                                                    class="p-1.5 text-green-500"
                                                >
                                                    <Check size={14} />
                                                </div>
                                            {/if}
                                            <button
                                                on:click={() => delTheme(theme)}
                                                class="p-1.5 text-stone-600 hover:text-red-400 hover:bg-red-400/10 rounded-md transition-all opacity-0 group-hover:opacity-100"
                                            >
                                                <Trash2 size={14} />
                                            </button>
                                        </div>
                                    </div>
                                {/each}
                            </div>
                        {/if}
                    </div>
                {/if}

                {#if missing.length > 0}
                    <div
                        class="bg-yellow-950/30 border border-yellow-900/50 rounded-xl px-4 py-3 text-yellow-500 text-sm"
                    >
                        <div class="flex items-center gap-2 mb-1">
                            <svg
                                class="w-4 h-4"
                                fill="none"
                                viewBox="0 0 24 24"
                                stroke="currentColor"
                            >
                                <path
                                    stroke-linecap="round"
                                    stroke-linejoin="round"
                                    stroke-width="2"
                                    d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 14c-.77 1.333.192 3 1.732 3z"
                                />
                            </svg>
                            <p class="font-semibold">Missing Theme Assets</p>
                        </div>
                        <ul
                            class="list-disc list-inside text-xs opacity-80 space-y-0.5 ml-1"
                        >
                            {#each missing as f}
                                <li class="font-mono">{f}</li>
                            {/each}
                        </ul>
                    </div>
                {/if}

                {#if error}
                    <div
                        class="bg-red-950/30 border border-red-900/50 rounded-xl px-4 py-3 text-red-400 text-sm flex items-center gap-2"
                    >
                        <svg
                            class="w-4 h-4 flex-shrink-0"
                            fill="none"
                            viewBox="0 0 24 24"
                            stroke="currentColor"
                        >
                            <path
                                stroke-linecap="round"
                                stroke-linejoin="round"
                                stroke-width="2"
                                d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
                            />
                        </svg>
                        <span>{error}</span>
                    </div>
                {/if}
            </div>
        </ExpandableSettingCard>
    </div>
</div>
