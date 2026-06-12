<script lang="ts">
    import {
        loadThemeFromDialog,
        saveActiveTheme,
        listThemes,
        loadThemeFromAppData,
        removeTheme,
    } from '$lib/theme/themeLoader'
    import { _ } from 'svelte-i18n'
    import { themeStore } from '$lib/theme/themeStore'
    import { onMount } from 'svelte'
    import {
        Brush,
        Trash2,
        Plus,
        Check,
        RotateCcw,
        Image as ImageIcon,
    } from '@lucide/svelte'
    import ExpandableSettingCard from '$lib/components/molecules/ExpandableSettingCard.svelte'
    import Dropdown from '$lib/components/molecules/Dropdown.svelte'
    import Button from '$lib/components/atoms/Button.svelte'
    import Textbox from '$lib/components/atoms/Textbox.svelte'
    import { invoke } from '@tauri-apps/api/core'
    import { load } from '@tauri-apps/plugin-store'
    import {
        background,
        type BackgroundType,
    } from '$lib/stores/background.svelte'

    type State = 'idle' | 'loading' | 'error'

    let state: State = 'idle'
    let error = ''
    let missing: string[] = []
    let activeName = ''
    let themes: string[] = []

    let themeType = 'default'
    const typeOptions = [
        {
            value: 'default',
            label: $_('pages.appearance.bootstrapThemeCard.dropdown.default'),
        },
        {
            value: 'custom',
            label: $_('pages.appearance.bootstrapThemeCard.dropdown.custom'),
        },
    ]

    let isInitialized = false

    const backgroundTypeOptions = [
        {
            value: 'default',
            label: $_('pages.appearance.backgroundCard.dropdown.default'),
        },
        {
            value: 'color',
            label: $_('pages.appearance.backgroundCard.dropdown.color'),
        },
        {
            value: 'image',
            label: $_('pages.appearance.backgroundCard.dropdown.image'),
        },
    ]

    let bgType: BackgroundType = 'default'
    let bgColor = '#0a0a0a'
    let bgInitialized = false

    async function pickBackgroundImage() {
        await background.pickImage()
        bgType = background.type
    }

    let colorTimer: ReturnType<typeof setTimeout> | undefined
    function onColorInput(value: string) {
        bgColor = value
        bgType = 'color'
        clearTimeout(colorTimer)
        colorTimer = setTimeout(() => background.setColor(value), 150)
    }

    // Persist when the dropdown type changes (after initial sync)
    $: if (bgInitialized && bgType !== background.type) {
        background.setType(bgType)
    }

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
            details: $_('rpc.general'),
            stateText: $_('rpc.appearance'),
        })
        ;(async () => {
            if (!background.loaded) await background.init()
            bgType = background.type
            bgColor = background.color
            bgInitialized = true
        })()

        let unsub: (() => void) | undefined
        let isMounted = true

        load('config.json').then(async (store) => {
            if (!isMounted) return

            refreshThemes()

            unsub = themeStore.subscribe((v) => {
                activeName = v?.themeName || ''
                const newType = v ? 'custom' : 'default'
                if (!isInitialized) {
                    themeType = newType
                    isInitialized = true
                } else if (themeType !== newType) {
                    themeType = newType
                }
            })
        })

        return () => {
            isMounted = false
            if (unsub) unsub()
        }
    })
</script>

<div class="flex flex-col gap-8">
    <div>
        <h1 class="text-3xl font-bold tracking-tight text-stone-100">
            {$_('pages.appearance.appearance')}
        </h1>
        <p class="text-stone-400 mt-1">
            {$_('pages.appearance.description')}
        </p>
    </div>

    <div class="grid gap-6">
        <ExpandableSettingCard
            title={$_('pages.appearance.bootstrapThemeCard.title')}
            description={$_('pages.appearance.bootstrapThemeCard.description')}
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
                            e.stopPropagation()
                            themeType = 'default'
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
                            {$_(
                                'pages.appearance.bootstrapThemeCard.customDropDownContents.importNew'
                            )}
                        </button>
                    {/if}
                </div>

                {#if themeType === 'custom'}
                    <div class="flex flex-col gap-2">
                        {#if themes.length === 0}
                            <p class="text-xs text-stone-500 italic py-2">
                                {$_(
                                    'pages.appearance.bootstrapThemeCard.customDropDownContents.noThemes'
                                )}
                            </p>
                        {:else}
                            <div class="grid gap-2">
                                {#each themes as theme}
                                    <div
                                        class="group flex items-center justify-between px-3 py-2 bg-stone-900/50 border {activeName ===
                                        theme
                                            ? 'border-stone-700 bg-stone-800/50'
                                            : 'border-stone-800'}  transition-all"
                                    >
                                        <button
                                            on:click={() => selectTheme(theme)}
                                            class="flex items-center gap-3 flex-grow text-left"
                                        >
                                            <div
                                                class="w-2 h-2 {activeName ===
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
                                                class="p-1.5 text-stone-600 hover:text-red-400 hover:bg-red-400/10 transition-all opacity-0 group-hover:opacity-100"
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
                        class="bg-yellow-950/30 border border-yellow-900/50 px-4 py-3 text-yellow-500 text-sm"
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
                            <p class="font-semibold">
                                {$_(
                                    'pages.appearance.bootstrapThemeCard.customDropDownContents.missingAssets'
                                )}
                            </p>
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
                        class="bg-red-950/30 border border-red-900/50 px-4 py-3 text-red-400 text-sm flex items-center gap-2"
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

        <ExpandableSettingCard
            title={$_('pages.appearance.backgroundCard.title')}
            description={$_('pages.appearance.backgroundCard.description')}
            isOpen={bgType !== 'default'}
        >
            <div slot="icon">
                <ImageIcon />
            </div>

            <div slot="action">
                {#if bgType !== 'default'}
                    <Button
                        variant="ghost"
                        size="sm"
                        class="h-8 w-8 !p-0"
                        on:click={(e) => {
                            e.stopPropagation()
                            bgType = 'default'
                        }}
                    >
                        <RotateCcw size={16} />
                    </Button>
                {/if}
            </div>

            <div class="flex flex-col gap-4">
                <Dropdown bind:value={bgType} options={backgroundTypeOptions} />

                {#if bgType === 'color'}
                    <div class="flex items-center gap-3">
                        <input
                            type="color"
                            value={bgColor}
                            on:input={(e) =>
                                onColorInput(e.currentTarget.value)}
                            class="h-10 w-16 cursor-pointer bg-transparent border border-stone-800 p-1"
                            aria-label={$_(
                                'pages.appearance.backgroundCard.colorLabel'
                            )}
                        />
                        <Textbox
                            value={bgColor}
                            fullWidth={false}
                            containerClass="w-32"
                            class="font-mono"
                            on:input={(e) => onColorInput(e.detail)}
                        />
                    </div>
                {/if}

                {#if bgType === 'image'}
                    <div class="flex flex-col gap-3">
                        <button
                            on:click={pickBackgroundImage}
                            class="flex items-center gap-1.5 text-xs text-stone-500 hover:text-stone-300 transition-colors uppercase tracking-wider font-semibold whitespace-nowrap"
                        >
                            <Plus size={14} />
                            {$_('pages.appearance.backgroundCard.chooseImage')}
                        </button>
                    </div>
                {/if}
            </div>
        </ExpandableSettingCard>
    </div>
</div>
