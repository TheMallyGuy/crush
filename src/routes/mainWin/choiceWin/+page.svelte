<script lang="ts">
    import { onMount } from 'svelte'
    import { openUrl } from '@tauri-apps/plugin-opener'
    import { invoke } from '@tauri-apps/api/core'
    import { load } from '@tauri-apps/plugin-store'
    import {
        Gamepad2,
        Wrench,
        ChevronDown,
        DraftingCompass,
        MessageCircleHeart,
    } from '@lucide/svelte'
    import { getCurrentWindow, LogicalSize } from '@tauri-apps/api/window'
    import { _ } from 'svelte-i18n'
    import { deepLinkUrl } from '$lib/stores/deeplink'
    import { info } from '@tauri-apps/plugin-log'
    import { launchAppType } from '$lib/stores/launchAppType'
    import BlurFade from '$lib/components/magic/blur-fade/blur-fade.svelte'
    import Text from '$lib/components/molecules/Text.svelte'

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
            launchAppType.set('player')
        } else {
            launchAppType.set('studio')
        }

        launchBoostrap()
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
                width: 1200,
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

        invoke('set_rpc', {
            details: $_('rpc.general'),
            stateText: 'Loading...',
        })
        firstLaunchValue = await checkLaunch()
    })
</script>

<div
    class="flex flex-col items-center justify-between h-screen w-screen flex-1 p-3 py-8 gap-5 text-white"
>
    <div>
        <h1
            class="text-4xl tracking-tight text-foreground font-medium text-center"
        >
            <BlurFade delay={0}>Crush</BlurFade>
            <BlurFade delay={0.1}><Text /></BlurFade>
        </h1>
    </div>

    <div class="flex gap-3 w-full max-w-md">
        <div class="relative w-1/2 h-24">
            <BlurFade delay={0.2} class="h-full flex">
                <div
                    class="w-full h-full flex border border-stone-800 hover:border-stone-700 transition-all"
                >
                    <button
                        onclick={handlePlayClick}
                        class="flex-1 h-full bg-stone-900/60 hover:bg-stone-800 active:scale-[0.98] disabled:opacity-50 p-4 flex flex-col items-center justify-center gap-2 transition-all text-card-foreground overflow-hidden"
                    >
                        {#if playVariant === 'default'}
                            <Gamepad2 class="size-6" />
                        {:else}
                            <DraftingCompass class="size-6" />
                        {/if}
                        <span
                            class="font-medium text-sm text-center line-clamp-1"
                        >
                            {playVariant === 'default'
                                ? $_('pages.choiceWin.playRoblox')
                                : $_('pages.choiceWin.makeGames')}
                        </span>
                    </button>

                    <div class="w-px h-full bg-stone-800 shrink-0"></div>

                    <button
                        onclick={() => (showVariantMenu = !showVariantMenu)}
                        class="h-full px-2.5 bg-stone-900/60 hover:bg-stone-800 active:scale-[0.98] flex items-center justify-center text-muted-foreground hover:text-accent-foreground transition-all overflow-hidden"
                    >
                        <ChevronDown
                            class="size-4 transition-transform duration-200 {showVariantMenu
                                ? 'rotate-180'
                                : ''}"
                        />
                    </button>
                </div>
            </BlurFade>

            {#if showVariantMenu}
                <div class="absolute bottom-full right-0 mb-2 z-50">
                    <BlurFade duration={0.15}>
                        <div
                            class="absolute top-full right-0 mt-1 z-50 bg-stone-900 border border-stone-800 p-1 flex flex-col gap-0.5 min-w-35"
                        >
                            <button
                                onclick={() => {
                                    playVariant = 'default'
                                    showVariantMenu = false
                                }}
                                class="flex items-center gap-2 px-3 py-1.5 hover:bg-accent text-sm text-popover-foreground transition-all whitespace-nowrap {playVariant ===
                                'default'
                                    ? 'bg-stone-800/50'
                                    : ''}"
                            >
                                <Gamepad2 class="size-3.5" />
                                Player
                                {#if playVariant === 'default'}
                                    <span
                                        class="ml-auto pl-3 text-primary text-xs"
                                        >✓</span
                                    >
                                {/if}
                            </button>
                            <button
                                onclick={() => {
                                    playVariant = 'v2'
                                    showVariantMenu = false
                                }}
                                class="flex items-center gap-2 px-3 py-1.5 hover:bg-accent text-sm text-popover-foreground transition-all whitespace-nowrap {playVariant ===
                                'v2'
                                    ? 'bg-stone-800/50'
                                    : ''}"
                            >
                                <DraftingCompass class="size-3.5" />
                                Studio
                                {#if playVariant === 'v2'}
                                    <span
                                        class="ml-auto pl-3 text-primary text-xs"
                                        >✓</span
                                    >
                                {/if}
                            </button>
                        </div>
                    </BlurFade>
                </div>
            {/if}
        </div>

        <div class="flex flex-col gap-2 w-1/2 h-24">
            <BlurFade delay={0.4} class="flex-1 flex">
                <button
                    onclick={openmainwin}
                    class="w-full h-full bg-stone-900/60 hover:bg-stone-800 active:scale-[0.98] disabled:opacity-50 px-4 flex items-center justify-center gap-3 transition-all hover:border-stone-700 text-stone-200 text-sm overflow-hidden shadow-sm"
                >
                    <Wrench class="size-4 shrink-0" />
                    <span class="font-medium truncate"
                        >{$_('pages.choiceWin.config')}</span
                    >
                </button>
            </BlurFade>

            <BlurFade delay={0.6} class="flex-1 flex">
                <button
                    onclick={openDiscordServer}
                    class="w-full h-full bg-stone-900/60 hover:bg-stone-800 active:scale-[0.98] disabled:opacity-50 px-4 flex items-center justify-center gap-3 transition-all hover:border-stone-700 text-stone-200 text-sm overflow-hidden shadow-sm"
                >
                    <MessageCircleHeart class="size-4 shrink-0" />
                    <span class="font-medium truncate"
                        >{$_('pages.choiceWin.discord')}</span
                    >
                </button>
            </BlurFade>
        </div>
    </div>
</div>
