<script lang="ts">
    import { onMount } from 'svelte'
    import { openUrl } from '@tauri-apps/plugin-opener'
    import { invoke } from '@tauri-apps/api/core'
    import { load } from '@tauri-apps/plugin-store'
    import {
        Gamepad2,
        Wrench,
        Info,
        ChevronDown,
        DraftingCompass,
    } from '@lucide/svelte'
    import { getCurrentWindow, LogicalSize } from '@tauri-apps/api/window'
    import { _ } from 'svelte-i18n'
    import { deepLinkUrl } from '$lib/stores/deeplink'
    import { info } from '@tauri-apps/plugin-log'

    let firstLaunchValue: boolean | undefined
    let showVariantMenu = false
    let playVariant: 'default' | 'v2' = 'default'

    async function launchBoostrap() {
        deepLinkUrl.set(null)
        localStorage.removeItem('deepLinkUrl') // deep clean

        await invoke('create_or_focus_window', {
            label: 'CrushBoostrap',
            url: 'boostrapWin',
            title: 'Crush',
            width: 500.0,
            height: 350.0,
            minWidth: 500,
            minHeight: 350.0,
        })

        setTimeout(() => {
            // wait before killing to prevent crash
            getCurrentWindow().close()
        }, 100)
    }

    function handlePlayClick() {
        if (playVariant === 'default') {
            launchBoostrap()
        } else {
            // do absolutely nothing
        }
    }

    async function checkLaunch() {
        const store = await load('config.json')

        let firstLaunch = await store.get<boolean>('firstLaunch')

        if (firstLaunch === undefined || firstLaunch === true) {
            await store.set('firstLaunch', false)
            await store.save()
            return true
        }

        return false
    }

    async function openmainwin() {
        if (firstLaunchValue) {
            info('it is first launch')
            await invoke('create_or_focus_window', {
                // temp
                label: 'CrushHello',
                url: 'mainWin/crushHello/welcome',
                title: 'Welcome',
                width: 1000,
                height: 700,
                minWidth: 1000,
                minHeight: 700,
            })
        } else {
            info('it is not first launch')
            await invoke('create_or_focus_window', {
                label: 'CrushMainWindow',
                url: 'mainWin/Ui/integrations',
                title: 'Crush',
                width: 1000,
                height: 600,
                minWidth: 1000,
                minHeight: 600,
            })
        }
        setTimeout(() => {
            // wait before killing to prevent crash
            getCurrentWindow().close()
        }, 100)
    }

    async function openDiscordServer() {
        openUrl('https://discord.gg/ER64xhvQkw')
    }

    onMount(async () => {
        const win = getCurrentWindow()
        if (win.label === 'crushBoostrapChoiceWindow') {
            await win.setSize(new LogicalSize(500, 250))
            await win.center()
        }

        await invoke('set_rpc', {
            details: $_('rpc.general'),
            stateText: 'Loading...',
        })
        firstLaunchValue = await checkLaunch()
    })
</script>

<div
    class="flex flex-col items-center justify-center h-screen flex-1 p-3 gap-5 0 text-white selection:bg-stone-800"
>
    <div>
        <h1 class="text-4xl tracking-tight text-stone-100 font-medium">
            Crush
        </h1>
    </div>

    <div class="flex flex-col gap-2 w-full max-w-sm">
        <div class="relative w-full max-w-sm">
            <div
                class="w-full flex rounded-lg border border-stone-800 hover:border-stone-700 overflow-hidden transition-all"
            >
                <button
                    on:click={handlePlayClick}
                    class="flex-1 bg-stone-900/60 hover:bg-stone-800 active:scale-[0.98] disabled:opacity-50 p-4 flex items-center justify-center gap-3 transition-all text-stone-200"
                >
                    {#if playVariant === 'default'}
                        <Gamepad2 class="size-5" />
                    {:else}
                        <DraftingCompass class="size-5" />
                    {/if}
                    <span class="font-medium">
                        {playVariant === 'default'
                            ? $_('pages.choiceWin.playRoblox')
                            : 'Make Games'}  <!--localize this-->
                    </span>
                </button>

                <div class="w-px bg-stone-800 flex-shrink-0"></div>

                <button
                    on:click={() => (showVariantMenu = !showVariantMenu)}
                    class="bg-stone-900/60 hover:bg-stone-800 active:scale-[0.98] px-3 flex items-center justify-center text-stone-400 hover:text-stone-200 transition-all"
                >
                    <ChevronDown
                        class="size-4 transition-transform duration-200 {showVariantMenu
                            ? 'rotate-180'
                            : ''}"
                    />
                    <!-- animation -->
                </button>
            </div>

            {#if showVariantMenu}
                <div
                    class="absolute top-full right-0 mt-1 z-50 bg-stone-900 border border-stone-800 rounded-lg p-1 flex flex-col gap-0.5 min-w-[140px]"
                >
                    <button
                        on:click={() => {
                            playVariant = 'default'
                            showVariantMenu = false
                        }}
                        class="flex items-center gap-2 px-3 py-1.5 rounded-md hover:bg-stone-800 text-sm text-stone-200 transition-all whitespace-nowrap {playVariant ===
                        'default'
                            ? 'bg-stone-800/50'
                            : ''}"
                    >
                        <Gamepad2 class="size-3.5" />
                        Player
                        {#if playVariant === 'default'}<span
                                class="ml-auto pl-3 text-purple-400 text-xs"
                                >✓</span
                            >{/if}
                    </button>
                    <button
                        on:click={() => {
                            playVariant = 'v2'
                            showVariantMenu = false
                        }}
                        class="flex items-center gap-2 px-3 py-1.5 rounded-md hover:bg-stone-800 text-sm text-stone-200 transition-all whitespace-nowrap {playVariant ===
                        'v2'
                            ? 'bg-stone-800/50'
                            : ''}"
                    >
                        <DraftingCompass class="size-3.5" />
                        Studio
                        {#if playVariant === 'v2'}<span
                                class="ml-auto pl-3 text-purple-400 text-xs"
                                >✓</span
                            >{/if}
                    </button>
                </div>
            {/if}
        </div>

        <div class="flex flex-row gap-2 w-full">
            <button
                on:click={openmainwin}
                class="w-1/2 bg-stone-900/60 hover:bg-stone-800 active:scale-[0.98] disabled:opacity-50 rounded-lg p-4 flex flex-col items-center justify-center gap-2 transition-all border border-stone-800 hover:border-stone-700 text-stone-200 text-sm"
            >
                <Wrench class="size-5" />
                {$_('pages.choiceWin.config')}
            </button>

            <button
                on:click={openDiscordServer}
                class="w-1/2 bg-stone-900/60 hover:bg-stone-800 active:scale-[0.98] disabled:opacity-50 rounded-lg p-4 flex flex-col items-center justify-center gap-2 transition-all border border-stone-800 hover:border-stone-700 text-stone-200 text-sm text-center"
            >
                <Info class="size-5" />
                Discord
            </button>
        </div>
    </div>
</div>
