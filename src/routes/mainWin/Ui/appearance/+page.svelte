<script lang="ts">
    import {
        loadThemeFromDialog,
        saveActiveTheme,
    } from '$lib/theme/themeLoader'
    import { themeStore } from '$lib/theme/themeStore'
    import { onMount } from 'svelte'
    import SettingCard from '$lib/components/SettingCard.svelte'
    import Dropdown from '$lib/components/Dropdown.svelte'

    type State = 'idle' | 'loading' | 'error'

    let state: State = 'idle'
    let error = ''
    let missing: string[] = []
    let activeName = ''

    let themeType = 'default'
    const typeOptions = [
        { value: 'default', label: 'Default' },
        { value: 'custom', label: 'Custom' },
    ]

    let isInitialized = false

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
            state = 'idle'
        } catch (e: any) {
            error = e.message ?? 'Unknown error'
            state = 'error'
            themeType = 'default'
        }
    }

    $: if (isInitialized && themeType) {
        if (themeType === 'default' && activeName) {
            themeStore.set(null)
            saveActiveTheme(null)
        } else if (themeType === 'custom' && !activeName) {
            pick()
        }
    }

    onMount(async () => {
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
        <SettingCard
            title="Bootstrapper Launcher"
            description="Choose diffrent launch screen style! Experimental bloxstrap XML boostrap support."
        >
            <div slot="icon">
                <svg
                    class="w-6 h-6 text-stone-400"
                    fill="none"
                    viewBox="0 0 24 24"
                    stroke="currentColor"
                    stroke-width="1.5"
                >
                    <path
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        d="M9.53 16.122l9.37-9.37m0 0l-1.01-1.01a1.125 1.125 0 00-1.59 0l-1.102 1.101m2.102 1.01l-2.102-1.01m0 0l-9.37 9.37a1.125 1.125 0 000 1.59l1.01 1.01a1.125 1.125 0 001.59 0l9.37-9.37zm0 0l1.101-1.102m-7.034 7.034l2.102 1.01m1.59 1.591l-2.102-1.01m0 0l-1.01-1.01a1.125 1.125 0 00-1.59 0l-1.101 1.101m2.101 1.01l-2.101-1.01m0 0l-9.37 9.37a1.125 1.125 0 000 1.59l1.01 1.01a1.125 1.125 0 001.59 0l9.37-9.37z"
                    />
                </svg>
            </div>

            <div slot="footer" class="flex flex-col w-full gap-4">
                <div class="flex items-center justify-between w-full">
                    <Dropdown bind:value={themeType} options={typeOptions} />

                    {#if themeType === 'custom'}
                        <button
                            on:click={pick}
                            class="text-xs text-stone-500 hover:text-stone-300 transition-colors uppercase tracking-wider font-semibold"
                        >
                            Import New XML
                        </button>
                    {/if}
                </div>

                {#if themeType === 'custom' && activeName}
                    <div
                        class="flex items-center gap-2 px-3 py-2 bg-stone-900/50 border border-stone-800 rounded-lg"
                    >
                        <svg
                            class="w-4 h-4 text-green-500"
                            fill="none"
                            viewBox="0 0 24 24"
                            stroke="currentColor"
                        >
                            <path
                                stroke-linecap="round"
                                stroke-linejoin="round"
                                stroke-width="2"
                                d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"
                            />
                        </svg>
                        <span class="text-sm text-stone-300"
                            >Active: <span class="text-stone-100 font-medium"
                                >{activeName}</span
                            ></span
                        >
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
        </SettingCard>
    </div>
</div>
