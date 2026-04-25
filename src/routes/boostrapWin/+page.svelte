<script lang="ts">
    import { downloadRoblox } from '$lib/downloadRoblox'
    import type {
        ProgressEvent,
        ThemeState,
        BootstrapElement,
        Installation,
        Integrations,
    } from '$lib/types'
    import { launchPlayer, applyMods } from '$lib/launchRoblox'
    import { relaunch } from '@tauri-apps/plugin-process'
    import { getCurrentWindow, LogicalSize } from '@tauri-apps/api/window'
    import { onMount, onDestroy } from 'svelte'
    import { afterNavigate } from '$app/navigation'
    import { themeStore, resolveAsset } from '$lib/theme/themeStore'
    import { invoke } from '@tauri-apps/api/core'
    import { listen } from '@tauri-apps/api/event'
    import { deepLinkUrl } from '$lib/stores/deeplink'
    import { goto } from '$app/navigation'
    import { get } from 'svelte/store'
    import { page } from '$app/stores'
    import { load } from '@tauri-apps/plugin-store'
    import { _ } from 'svelte-i18n'
    import { info } from '@tauri-apps/plugin-log'
    import { getBestServers } from '$lib/rovalraHelper/rovalra'
    import { parseRobloxDeepLink, rebuildDeeplink } from '$lib/robloxDeepLink'
    import { Window } from '@tauri-apps/api/window';

    let state: ThemeState | null = null
    const unsub = themeStore.subscribe((v) => {
        state = v
    })
    onDestroy(unsub)

    let status = 'Preparing...'
    let downloadFile = '',
        downloadDone = 0,
        downloadTotal = 0
    let extractFile = '',
        extractDone = 0,
        extractTotal = 0
    let done = false
    let error = false
    let errorMessage = ''

    let textValues: Record<string, string> = {}

    function handleProgress(e: ProgressEvent) {
        if (e.type === 'status') {
            status = e.message
            textValues['StatusText'] = e.message
            downloadDone = 0
            downloadTotal = 0
            extractDone = 0
            extractTotal = 0
        } else if (e.type === 'download') {
            extractDone = 0
            extractTotal = 0
            downloadFile = e.file
            downloadDone = e.done
            downloadTotal = e.total
        } else if (e.type === 'extract') {
            downloadFile = ''
            downloadDone = 0
            downloadTotal = 0
            extractFile = e.file
            extractDone = e.done
            extractTotal = e.total
        }
        textValues = { ...textValues }
        window.dispatchEvent(new CustomEvent('crush:progress', { detail: e }))
    }

    async function cancel() {
        await relaunch()
    }

    async function setupWindow() {
        const win = getCurrentWindow()
        if (!state) {
            await win.setSize(new LogicalSize(630, 363))
            await win.center()
            await win.show()
            return
        }

        if (state.config) {
            const width = state.config.width || 600
            const height = state.config.height || 400

            await win.setSize(new LogicalSize(width, height))
            await win.center()

            for (const el of state.config.elements) {
                if (el.name) {
                    textValues[el.name] =
                        el.name === 'StatusText' ? 'Preparing...' : ''
                }
            }
            textValues = { ...textValues }

            const titleBar = state.config.elements.find(
                (e) => e.type === 'TitleBar'
            )
            if (titleBar?.props?.Title) {
                await win.setTitle(titleBar.props.Title)
            }
        }
        await win.show()
    }

    async function runBootstrap() {
        error = false
        errorMessage = ''
        done = false
        handleProgress({ type: 'status', message: 'Preparing...' })

        try {
            const store = await load('config.json')
            const installation = await store.get<Installation>('installation')

            const version = await downloadRoblox(
                handleProgress,
                installation?.version === 'latest'
                    ? undefined
                    : installation?.version
            )

            done = true
            handleProgress({ type: 'status', message: 'Applying modification' })
            await applyMods(version)

            handleProgress({ type: 'status', message: 'Launching' })

            const integrations = await store.get<Integrations>('integrations')
            const url = $deepLinkUrl ?? ''
            await performLaunch(version, url, integrations)

            await sleep(1000)
            await invoke('watch_logs')
            await finalizeBootstrap()
        } catch (e: any) {
            handleBootstrapError(e)
        }
    }

    async function performLaunch(
        version: string,
        url: string,
        integrations: Integrations | null | undefined
    ) {
        const parsed = parseRobloxDeepLink(url)

        if (!parsed.placelauncherurl) {
            return launchPlayer(version, url)
        }

        const launchUrl = new URL(parsed.placelauncherurl)
        const request = launchUrl.searchParams.get('request')
        const joinServerForYou = integrations?.roValra?.joinServerForYouValue ?? false

        const isSpecialRequest = request === 'RequestFollowUser' || request === 'RequestPrivateGame'
        if (isSpecialRequest || !joinServerForYou || parsed.placeId == null) {
            info(`Launching with url: ${url}`)
            return launchPlayer(version, url)
        }

        const result = await getBestServers(parsed.placeId)
        const bestServer = result.servers[0]

        if (!bestServer) {
            return launchPlayer(version, url)
        }

        const finalUrl = rebuildDeeplink(parsed, parsed.placeId, bestServer.server_id)
        return launchPlayer(version, finalUrl)
    }

    async function finalizeBootstrap() {
        await invoke('create_or_focus_window', {
            label: 'crushBoostrapChoiceWindow',
            url: 'mainWin/choiceWin',
            title: 'Crush',
            width: 500.0,
            height: 250.0,
            minWidth: 500.0,
            minHeight: 250.0,
        })

        const win = getCurrentWindow()
        if (win.label === 'crushBoostrapChoiceWindow') {
            await goto('/mainWin/choiceWin')
            return
        }


        const choiceWin = await Window.getByLabel('crushBoostrapChoiceWindow');
        await choiceWin?.close();

        setTimeout(() => {
            win.close()
        }, 100)
    }

    function handleBootstrapError(e: any) {
        error = true
        errorMessage = e.message || String(e)
        handleProgress({
            type: 'status',
            message: `Error: ${errorMessage}`,
        })
        console.error('Bootstrap failed:', e)
    }

    onMount(async () => {
        await setupWindow()
    })

    afterNavigate(async () => {
        await runBootstrap()
    })

    function sleep(ms: number) {
        return new Promise(resolve => setTimeout(resolve, ms));
    }

    function getPosStyle(h?: string, v?: string) {
        const styles: string[] = []
        const transforms: string[] = []

        const hPos =
            h === 'Right' ? 'right:0' : h === 'Center' ? 'left:50%' : 'left:0'
        styles.push(hPos)
        if (h === 'Center') transforms.push('translateX(-50%)')

        const vPos =
            v === 'Bottom' ? 'bottom:0' : v === 'Center' ? 'top:50%' : 'top:0'
        styles.push(vPos)
        if (v === 'Center') transforms.push('translateY(-50%)')

        if (transforms.length > 0) {
            styles.push(`transform:${transforms.join(' ')}`)
        }

        return styles.map((s) => `${s};`).join('')
    }

    function opStyle(op?: number) {
        return op ? `opacity:${op};` : ''
    }

    function marginStyle(m?: {
        top: number
        right: number
        bottom: number
        left: number
    }) {
        if (!m) return ''
        return Object.entries(m)
            .map(([side, val]) => (val ? `margin-${side}:${val}px;` : ''))
            .join('')
    }
    function asset(src?: string) {
        if (!state) return ''
        return resolveAsset(state, src)
    }

    $: cfg = state?.config
    $: isDark = cfg?.theme === 'Dark'
    $: noRound = cfg?.windowCornerPreference === 'DoNotRound'
    $: elements = cfg?.elements ?? []

    function mountHtml(node: HTMLElement, content: string) {
        if (!content) return
        node.innerHTML = content
        const scripts = node.querySelectorAll('script')
        scripts.forEach((oldScript) => {
            const documentScript = document.createElement('script')
            Array.from(oldScript.attributes).forEach((attr) =>
                documentScript.setAttribute(attr.name, attr.value)
            )
            documentScript.appendChild(
                document.createTextNode(oldScript.innerHTML)
            )
            oldScript.parentNode?.replaceChild(documentScript, oldScript)
        })
    }

    async function loadHtml(src?: string) {
        if (!src) return ''
        if (!state) return ''
        const url = resolveAsset(state, src)
        try {
            const res = await fetch(url)
            return await res.text()
        } catch (e) {
            console.error('Failed to load html:', e)
            return ''
        }
    }
</script>

{#if state}
    {#if state.isHtmlTheme}
        <div
            class="relative overflow-hidden h-screen w-screen bg-transparent"
            use:mountHtml={state.customHtml || ''}
        ></div>
    {:else if cfg}
        <div
            class="relative overflow-hidden h-screen w-screen"
            style="
                background:{isDark ? 'rgba(0,0,0,0.8)' : 'rgba(255,255,255,0.8)'};
                color:{isDark ? '#fff' : '#000'};
                border-radius:{noRound ? '0' : '8px'};
            "
        >
            {#each elements as el}
                {#if el.type === 'Rectangle'}
                    <div
                        class="absolute"
                        style="
                            {getPosStyle(el.hAlign, el.vAlign)}
                            width:{el.width}px;
                            height:{el.height}px;
                            background:{el.props.Fill || '#000'};
                            {el.props.RadiusX
                            ? `border-radius:${el.props.RadiusX}px;`
                            : ''}
                            {opStyle(el.opacity)}
                            {marginStyle(el.margin)}
                            z-index:{el.zIndex ?? 0};
                        "
                    ></div>
                {:else if el.type === 'Image' || el.props.source}
                    <img
                        src={asset(
                            el.props.source ||
                                el.props.Source ||
                                el.props.ImageSource
                        )}
                        class="absolute object-cover {el.props.class ||
                            el.props.Class ||
                            ''}"
                        style="
                            {getPosStyle(el.hAlign, el.vAlign)}
                            {el.width ? `width:${el.width}px;` : ''}
                            {el.height ? `height:${el.height}px;` : ''}
                            {opStyle(el.opacity)}
                            {marginStyle(el.margin)}
                            z-index:{el.zIndex ?? 0};
                        "
                        alt=""
                    />
                {:else if el.type === 'TextBlock'}
                    <span
                        id={el.name}
                        class="absolute whitespace-nowrap pointer-events-none leading-none {el
                            .props.class ||
                            el.props.Class ||
                            ''}"
                        style="
                            {getPosStyle(el.hAlign, el.vAlign)}
                            {opStyle(el.opacity)}
                            {marginStyle(el.margin)}
                            {el.props.Foreground
                            ? `color:${el.props.Foreground};`
                            : ''}
                            {el.props.FontSize
                            ? `font-size:${el.props.FontSize}px;`
                            : ''}
                            z-index:{el.zIndex ?? 0};
                        "
                        >{textValues[el.name ?? ''] ??
                            el.props.Text ??
                            ''}</span
                    >
                {:else if el.type === 'Button'}
                    <button
                        id={el.name}
                        on:click={cancel}
                        class="absolute bg-transparent border-0 cursor-pointer focus:outline-none focus:ring-0 {el
                            .props.class ||
                            el.props.Class ||
                            ''}"
                        style="
                            {getPosStyle(el.hAlign, el.vAlign)}
                            {el.width ? `width:${el.width}px;` : ''}
                            {el.height ? `height:${el.height}px;` : ''}
                            {opStyle(el.opacity)}
                            {marginStyle(el.margin)}
                            z-index:{el.zIndex ?? 2};
                        ">{el.props.Content || el.props.Label || ''}</button
                    >
                {:else if el.type === 'ProgressBar'}
                    <div
                        class="absolute overflow-hidden bg-white/10 {el.props
                            .class ||
                            el.props.Class ||
                            ''}"
                        style="
                            {getPosStyle(el.hAlign, el.vAlign)}
                            {el.width ? `width:${el.width}px;` : ''}
                            {el.height ? `height:${el.height}px;` : ''}
                            {opStyle(el.opacity)}
                            border-radius:{el.props.CornerRadius ?? 0}px;
                            z-index:{el.zIndex ?? 0};
                        "
                    >
                        {#if !done}
                            <div
                                class="h-full w-2/5 animate-indeterminate"
                                style="background:{el.props.Foreground ??
                                    '#919191'};border-radius:inherit;"
                            ></div>
                        {:else}
                            <div
                                class="h-full w-full transition-[width] duration-300 ease-out"
                                style="background:{el.props.Foreground ??
                                    '#919191'};border-radius:inherit;"
                            ></div>
                        {/if}
                    </div>
                {:else if el.type === 'Html'}
                    {#await loadHtml(el.props.Source || el.props.File) then htmlContent}
                        <div
                            id={el.name}
                            class="absolute {el.props.class ||
                                el.props.Class ||
                                ''}"
                            style="
                            {getPosStyle(el.hAlign, el.vAlign)}
                            {el.width ? `width:${el.width}px;` : ''}
                            {el.height ? `height:${el.height}px;` : ''}
                            {opStyle(el.opacity)}
                            {marginStyle(el.margin)}
                            z-index:{el.zIndex ?? 0};
                        "
                            use:mountHtml={htmlContent ||
                                el.props.Content ||
                                el.props.Html ||
                                ''}
                        ></div>
                    {/await}
                {/if}
            {/each}
        </div>
    {/if}
{:else}
    <div
        class="relative h-screen bg-transparent text-white selection:bg-stone-800"
    >
        <div
            class="absolute inset-0 flex flex-col items-center justify-center text-center p-3 pb-24 gap-6"
        >
            <div>
                <h1 class="text-4xl tracking-tight text-stone-100 font-medium">
                    Crush
                </h1>
                <p class={error ? 'text-red-400 mt-2' : 'text-stone-400 mt-2'}>
                    {status}
                </p>
            </div>
            {#if !done}
                <div class="w-full max-w-sm flex flex-col gap-3">
                    {#if downloadTotal > 0}
                        <div class="flex flex-col gap-1.5">
                            <div
                                class="flex justify-between text-xs text-stone-400"
                            >
                                <span class="truncate max-w-[70%]"
                                    >{downloadFile}</span
                                >
                                <span>{downloadDone}/{downloadTotal}</span>
                            </div>
                            <div
                                class="w-full h-1 bg-stone-800 rounded-full overflow-hidden"
                            >
                                <div
                                    class="h-full bg-stone-200 rounded-full transition-all duration-300"
                                    style="width: {(downloadDone /
                                        downloadTotal) *
                                        100}%"
                                ></div>
                            </div>
                        </div>
                    {/if}
                    {#if extractTotal > 0}
                        <div class="flex flex-col gap-1.5">
                            <div
                                class="flex justify-between text-xs text-stone-400"
                            >
                                <span class="truncate max-w-[70%]"
                                    >{extractFile}</span
                                >
                                <span>{extractDone}/{extractTotal}</span>
                            </div>
                            <div
                                class="w-full h-1 bg-stone-800 rounded-full overflow-hidden"
                            >
                                <div
                                    class="h-full bg-stone-200 rounded-full transition-all duration-300"
                                    style="width: {(extractDone /
                                        extractTotal) *
                                        100}%"
                                ></div>
                            </div>
                        </div>
                    {/if}
                </div>
            {/if}
        </div>
        <div
            class="absolute bottom-6 left-0 w-full flex flex-col items-center gap-3 p-3"
        >
            <div class="w-full max-w-sm flex gap-3">
                {#if error}
                    <button
                        on:click={runBootstrap}
                        class="flex-1 bg-stone-200 hover:bg-white text-stone-950 active:scale-[0.98] rounded-lg p-4 flex items-center justify-center gap-3 transition-all font-medium"
                    >
                        {$_('pages.boostrapWin.retry')}
                    </button>
                {/if}
                <button
                    on:click={cancel}
                    class="{error
                        ? 'flex-1'
                        : 'w-full'} bg-stone-900 hover:bg-stone-800 active:scale-[0.98] rounded-lg p-4 flex items-center justify-center gap-3 transition-all border border-stone-800 hover:border-stone-700 text-stone-200"
                >
                    <span class="font-medium"
                        >{$_('pages.boostrapWin.cancel')}</span
                    >
                </button>
            </div>
        </div>
    </div>
{/if}

<style>
    @keyframes indeterminate {
        0% {
            transform: translateX(-150%) scaleX(1);
        }
        50% {
            transform: translateX(80%) scaleX(1.6);
        }
        100% {
            transform: translateX(300%) scaleX(1);
        }
    }
    .animate-indeterminate {
        animation: indeterminate 1.5s cubic-bezier(0.4, 0, 0.6, 1) infinite;
        transform-origin: left center;
    }
</style>
