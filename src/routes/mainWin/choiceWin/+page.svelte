<script lang="ts">
    import { onMount } from 'svelte'
    import { openUrl } from '@tauri-apps/plugin-opener'
    import { invoke } from '@tauri-apps/api/core'
    import { load } from '@tauri-apps/plugin-store'
    import { Gamepad2, Wrench, Info } from '@lucide/svelte'
    import { getCurrentWindow, LogicalSize } from '@tauri-apps/api/window'
    import { _ } from 'svelte-i18n'
    import { deepLinkUrl } from '$lib/stores/deeplink'
    import { info } from '@tauri-apps/plugin-log'

    let firstLaunchValue: boolean | undefined

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
            info("it is first launch")
            await invoke('create_or_focus_window', {
                // temp
                label: 'CrushHello',
                url: 'mainWin/crushHello/welcome',
                title: 'Welcome',
                width: 1000,
                height: 600,
                minWidth: 1000,
                minHeight: 600,
            })
        } else {
            info("it is not first launch")
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
    class="flex flex-col items-center justify-center h-screen flex-1 p-3 gap-5 bg-stone-950 text-white selection:bg-stone-800"
>
    <div>
        <h1 class="text-4xl tracking-tight text-stone-100 font-medium">
            Crush
        </h1>
    </div>

    <div class="flex flex-col gap-2 w-full max-w-sm">
        <button
            on:click={launchBoostrap}
            class="w-full bg-stone-900 hover:bg-stone-800 active:scale-[0.98] disabled:opacity-50 rounded-lg p-4 flex items-center justify-center gap-3 transition-all border border-stone-800 hover:border-stone-700 text-stone-200"
        >
            <Gamepad2 class="size-5" />
            <span class="font-medium">{$_('pages.choiceWin.playRoblox')}</span>
        </button>

        <div class="flex flex-row gap-2 w-full">
            <button
                on:click={openmainwin}
                class="w-1/2 bg-stone-900 hover:bg-stone-800 active:scale-[0.98] disabled:opacity-50 rounded-lg p-4 flex flex-col items-center justify-center gap-2 transition-all border border-stone-800 hover:border-stone-700 text-stone-200 text-sm"
            >
                <Wrench class="size-5" />
                {$_('pages.choiceWin.config')}
            </button>

            <button
                on:click={openDiscordServer}
                class="w-1/2 bg-stone-900 hover:bg-stone-800 active:scale-[0.98] disabled:opacity-50 rounded-lg p-4 flex flex-col items-center justify-center gap-2 transition-all border border-stone-800 hover:border-stone-700 text-stone-200 text-sm text-center"
            >
                <Info class="size-5" />
                Discord
            </button>
        </div>
    </div>
</div>
