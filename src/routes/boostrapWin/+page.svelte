<script lang="ts">
    import { downloadRoblox } from '$lib/downloadRoblox'
    import type {
        ProgressEvent,
        ThemeState,
        BootstrapElement,
        Installation,
        Integrations,
    } from '$lib/types'
    import { launchPlayer, launchStudio, applyMods } from '$lib/launchRoblox'
    import { relaunch } from '@tauri-apps/plugin-process'
    import { getCurrentWindow, LogicalSize } from '@tauri-apps/api/window'
    import { onMount, onDestroy } from 'svelte'
    import { afterNavigate } from '$app/navigation'
    import { themeStore, resolveAsset } from '$lib/theme/themeStore'
    import { invoke } from '@tauri-apps/api/core'
    import { listen } from '@tauri-apps/api/event'
    import { deepLinkUrl } from '$lib/stores/deeplink'
    import { goto } from '$app/navigation'
    import { load } from '@tauri-apps/plugin-store'
    import { _ } from 'svelte-i18n'
    import { info } from '@tauri-apps/plugin-log'
    import { getBestServers } from '$lib/rovalraHelper/rovalra'
    import { parseRobloxDeepLink, rebuildDeeplink } from '$lib/robloxDeepLink'
    import { Window } from '@tauri-apps/api/window'
    import { launchAppType } from '$lib/stores/launchAppType'
    import { loadFlag } from '$lib/fastflag/fastflagManagement'

    let state: ThemeState | null = null
    let windowReady = false

    const unsub = themeStore.subscribe(async (v) => {
        state = v
        // If the theme loads *after* the window is already mounted, resize now
        if (windowReady && v) {
            await setupWindow()
        }
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
        handleProgress({
            type: 'status',
            message: $_('pages.boostrapWin.steps.prep'),
        })

        try {
            const store = await load('config.json')
            const installation = await store.get<Installation>('installation')

            const url = $deepLinkUrl ?? ''
            const appType = url.startsWith('roblox-player:')
                ? 'player'
                : url.startsWith('roblox-studio:')
                  ? 'studio'
                  : $launchAppType === 'studio'
                    ? 'studio'
                    : 'player'

            const version = await downloadRoblox(handleProgress, appType)

            done = true
            handleProgress({
                type: 'status',
                message: $_('pages.boostrapWin.steps.applyMods'),
            })
            await applyMods(version, appType)

            handleProgress({
                type: 'status',
                message: $_('pages.boostrapWin.steps.launch'),
            })

            const integrations = await store.get<Integrations>('integrations')

            await loadFlag(appType, version)

            if (appType === 'studio') {
                await launchStudio(version)
            } else {
                if (integrations?.swifttunnel) {
                    handleProgress({
                        type: 'status',
                        message: 'Connecting to swifttunnel servers...',
                    })

                    try {
                        await invoke('connect', {
                            region: integrations.swifttunnel.perferedRegion,
                            routing: integrations.swifttunnel.enableRouting,
                            dynamicAssets: integrations.swifttunnel.assetsRouting
                        })
                    } catch (e) {
                        // do nothing lmao
                        info(`${e}`)
                    }

                    handleProgress({
                        type: 'status',
                        message: $_('pages.boostrapWin.steps.launch'),
                    })
                }

                await performLaunch(version, url, integrations)
                await sleep(1000)
                await invoke('watch_logs', { isVng: installation?.vng })
                await sleep(3000)
                await invoke('set_process_priority', {
                    priority: integrations?.priority ?? 'NORMAL_PRIORITY_CLASS',
                })
                await sleep(3000)

                if (integrations?.closeCrashHandler) {
                    await invoke('close_crash_handler')
                }
            }

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
        const joinServerForYou =
            integrations?.roValra?.joinServerForYouValue ?? false

        const isSpecialRequest =
            request === 'RequestFollowUser' || request === 'RequestPrivateGame'
        if (isSpecialRequest || !joinServerForYou || parsed.placeId == null) {
            info(`Launching with url: ${url}`)
            return launchPlayer(version, url)
        }

        const result = await getBestServers(parsed.placeId)
        const bestServer = result.servers[0]

        if (!bestServer) {
            return launchPlayer(version, url)
        }

        const finalUrl = rebuildDeeplink(
            parsed,
            parsed.placeId,
            bestServer.server_id
        )
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
        }

        const choiceWin = await Window.getByLabel('crushBoostrapChoiceWindow')
        await choiceWin?.close()

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
        windowReady = true
    })

    afterNavigate(async () => {
        await runBootstrap()
    })

    function sleep(ms: number) {
        return new Promise((resolve) => setTimeout(resolve, ms))
    }

    function buildStyle(el: BootstrapElement, zFallback = 0): string {
        const p: string[] = []
        const h = el.hAlign
        const v = el.vAlign
        let tx = '0', ty = '0', needsTransform = false

        // Horizontal
        if (h === 'Stretch') {
            p.push('left:0', 'right:0', 'width:100%')
        } else if (h === 'Right') {
            p.push('right:0')
            if (el.width != null) p.push(`width:${el.width}px`)
        } else if (h === 'Center') {
            p.push('left:50%')
            tx = '-50%'; needsTransform = true
            if (el.width != null) p.push(`width:${el.width}px`)
        } else {
            p.push('left:0')
            if (el.width != null) p.push(`width:${el.width}px`)
        }

        // Vertical
        if (v === 'Stretch') {
            p.push('top:0', 'bottom:0', 'height:100%')
        } else if (v === 'Bottom') {
            p.push('bottom:0')
            if (el.height != null) p.push(`height:${el.height}px`)
        } else if (v === 'Center') {
            p.push('top:50%')
            ty = '-50%'; needsTransform = true
            if (el.height != null) p.push(`height:${el.height}px`)
        } else {
            p.push('top:0')
            if (el.height != null) p.push(`height:${el.height}px`)
        }

        if (needsTransform) p.push(`transform:translate(${tx},${ty})`)

        // Margin — use !== 0 so negative values are never dropped
        const m = el.margin
        if (m) {
            if (m.left   !== 0) p.push(`margin-left:${m.left}px`)
            if (m.top    !== 0) p.push(`margin-top:${m.top}px`)
            if (m.right  !== 0) p.push(`margin-right:${m.right}px`)
            if (m.bottom !== 0) p.push(`margin-bottom:${m.bottom}px`)
        }

        if (el.opacity != null) p.push(`opacity:${el.opacity}`)
        p.push(`z-index:${el.zIndex ?? zFallback}`)

        return p.join(';') + ';'
    }

    function asset(src?: string) {
        if (!state) return ''
        return resolveAsset(state, src)
    }

    /** Resolve __marker__ binding values to actual runtime colors / strings */
    function resolveBindingProp(value: string | undefined): string {
        if (!value) return ''
        switch (value) {
            case '__fg__':
            case '__icon__': return rootFg
            case '__bg__':   return rootBg
            default:         return value
        }
    }

    /**
     * Map WPF Image Stretch to CSS object-fit.
     * WPF default is Uniform (contain). object-cover (WPF UniformToFill) is NOT
     * the default — it crops, which is what causes the "glitched" look.
     */
    function imgObjectFit(el: BootstrapElement): string {
        switch (el.props.Stretch ?? el.props.stretch) {
            case 'UniformToFill': return 'object-cover'
            case 'Uniform':       return 'object-contain'
            case 'None':          return 'object-none'
            case 'Fill':          return 'object-fill'
            default:
                // WPF default = Uniform. With both dims explicit the image fills exactly;
                // use object-contain so aspect ratio is preserved (no cropping).
                return 'object-contain'
        }
    }

    /** Auto-fit SVG viewBox to the path's actual bounding box after mount */
    function autoViewBox(svgNode: SVGSVGElement) {
        requestAnimationFrame(() => {
            const path = svgNode.querySelector('path')
            if (!path) return
            try {
                const bb = (path as SVGPathElement).getBBox()
                if (bb.width > 0 && bb.height > 0) {
                    svgNode.setAttribute(
                        'viewBox',
                        `${bb.x} ${bb.y} ${bb.width} ${bb.height}`
                    )
                }
            } catch {
                // getBBox can throw if SVG isn't in the DOM yet
            }
        })
    }

    $: cfg = state?.config
    $: isDark = cfg?.theme === 'Dark'
    $: noRound = cfg?.windowCornerPreference === 'DoNotRound'
    $: elements = cfg?.elements ?? []
    // Direct color overrides from root element (BloxstrapCustomBootstrapper)
    $: rootBg  = cfg?.background ?? (isDark ? 'rgba(0,0,0,0.85)' : 'rgba(255,255,255,0.85)')
    $: rootFg  = cfg?.foreground ?? (isDark ? '#ffffff' : '#000000')

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
            class="relative overflow-hidden h-screen w-screen"
            use:mountHtml={state.customHtml || ''}
        ></div>
    {:else if cfg}
        <div
            class="relative overflow-hidden h-screen w-screen"
            style="
                background:{rootBg};
                color:{rootFg};
                border-radius:{noRound ? '0' : '8px'};
            "
        >
            {#each elements as el}
                {#if el.type === 'TitleBar'}
                    <!-- Fake title bar: draggable region + optional OS controls -->
                    <div
                        class="absolute flex items-center justify-between px-2"
                        data-tauri-drag-region
                        style="
                            {buildStyle(el)}
                            {el.height == null ? 'height:30px;' : ''}
                            {el.width  == null ? 'width:100%;left:0;right:0;' : ''}
                            top:0;
                            user-select:none;
                        "
                    >
                        {#if el.props.Title}
                            <span class="text-xs pointer-events-none" style="color:{rootFg};opacity:0.7">
                                {el.props.Title}
                            </span>
                        {/if}
                        <div class="flex items-center gap-1 ml-auto">
                            {#if el.props.ShowMinimize === 'True'}
                                <button
                                    class="w-5 h-5 flex items-center justify-center text-xs hover:bg-white/20 rounded-sm transition-colors"
                                    style="color:{rootFg}"
                                    on:click={() => getCurrentWindow().minimize()}
                                >─</button>
                            {/if}
                            {#if el.props.ShowClose !== 'False'}
                                <button
                                    class="w-5 h-5 flex items-center justify-center text-xs hover:bg-red-500 hover:text-white rounded-sm transition-colors"
                                    style="color:{rootFg}"
                                    on:click={cancel}
                                >✕</button>
                            {/if}
                        </div>
                    </div>

                {:else if el.type === 'Rectangle'}
                    <div
                        class="absolute"
                        style="
                            {buildStyle(el)}
                            background:{el.props.Fill || '#000'};
                            {el.props.RadiusX ? `border-radius:${el.props.RadiusX}px;` : ''}
                        "
                    ></div>

                {:else if el.type === 'Path' && el.props.svgFigures}
                    <svg
                        class="absolute overflow-visible"
                        style={buildStyle(el)}
                        viewBox="0 0 1 1"
                        preserveAspectRatio={el.props.svgStretch === 'Uniform' ? 'xMidYMid meet' : 'none'}
                        use:autoViewBox
                    >
                        <path
                            d={el.props.svgFigures}
                            fill={resolveBindingProp(el.props.svgFill)}
                            fill-rule={el.props.svgFillRule ?? 'evenodd'}
                        />
                    </svg>

                {:else if el.type === 'Image' || el.props.source}
                    <!-- skip __logo__ and other unresolvable binding sources -->
                    {#if el.props.source && !el.props.source.startsWith('__')}
                        <img
                            src={asset(el.props.source || el.props.Source || el.props.ImageSource)}
                            class="absolute {imgObjectFit(el)} {el.props.class || el.props.Class || ''}"
                            style={buildStyle(el)}
                            alt=""
                        />
                    {/if}

                {:else if el.type === 'TextBlock'}
                    <span
                        id={el.name}
                        class="absolute pointer-events-none leading-none {el.props.class || el.props.Class || ''}"
                        style="
                            {buildStyle(el)}
                            color:{resolveBindingProp(el.props.Foreground) || rootFg};
                            {el.props.FontSize   ? `font-size:${el.props.FontSize}px;`     : ''}
                            {el.props.fontWeight ? `font-weight:${el.props.fontWeight};`   : ''}
                            {el.props.FontFamily ? `font-family:'${el.props.FontFamily}';` : ''}
                            {el.props.textAlign  ? `text-align:${el.props.textAlign};`     : 'white-space:nowrap;'}
                        "
                    >{textValues[el.name ?? ''] ?? el.props.Text ?? ''}</span>

                {:else if el.type === 'Border'}
                    <div
                        class="absolute"
                        style="
                            {buildStyle(el)}
                            background:{el.props.Background || 'transparent'};
                            {el.props.BorderBrush
                                ? `border:${el.props.BorderThickness ?? 1}px solid ${el.props.BorderBrush};`
                                : ''}
                            {el.props.CornerRadius ? `border-radius:${el.props.CornerRadius}px;` : ''}
                        "
                    ></div>

                {:else if el.type === 'Button'}
                    <button
                        id={el.name}
                        on:click={el.name === 'CancelButton' || el.props.Content === 'Cancel' ? cancel : undefined}
                        class="absolute cursor-pointer focus:outline-none {el.props.class || el.props.Class || ''}"
                        style="
                            {buildStyle(el, 2)}
                            background:{el.props.Background ?? 'transparent'};
                            color:{resolveBindingProp(el.props.Foreground) || rootFg};
                            {el.props.BorderBrush
                                ? `border:${el.props.BorderThickness ?? 1}px solid ${el.props.BorderBrush};`
                                : 'border:none;'}
                            {el.props.CornerRadius ? `border-radius:${el.props.CornerRadius}px;` : ''}
                            {el.props.FontSize ? `font-size:${el.props.FontSize}px;` : ''}
                            padding:0;
                        "
                    >{el.props.Content || el.props.Label || ''}</button>

                {:else if el.type === 'ProgressBar'}
                    <div
                        class="absolute overflow-hidden {el.props.class || el.props.Class || ''}"
                        style="
                            {buildStyle(el)}
                            background:{el.props.Background ?? 'rgba(255,255,255,0.1)'};
                            {el.props.BorderBrush
                                ? `border:${el.props.BorderThickness ?? 1}px solid ${el.props.BorderBrush};`
                                : ''}
                            border-radius:{el.props.CornerRadius ?? 0}px;
                        "
                    >
                        {#if !done}
                            <div
                                class="h-full w-2/5 animate-indeterminate"
                                style="background:{el.props.Foreground ?? '#919191'};border-radius:inherit;"
                            ></div>
                        {:else}
                            <div
                                class="h-full w-full transition-[width] duration-300 ease-out"
                                style="background:{el.props.Foreground ?? '#919191'};border-radius:inherit;"
                            ></div>
                        {/if}
                    </div>

                {:else if el.type === 'Html'}
                    {#await loadHtml(el.props.Source || el.props.File) then htmlContent}
                        <div
                            id={el.name}
                            class="absolute {el.props.class || el.props.Class || ''}"
                            style={buildStyle(el)}
                            use:mountHtml={htmlContent || el.props.Content || el.props.Html || ''}
                        ></div>
                    {/await}
                {/if}
            {/each}
        </div>
    {/if}
{:else}
    <div
        class="relative h-screen bg-obsidian text-white selection:bg-stone-800"
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
                                class="w-full h-1 bg-stone-800 overflow-hidden"
                            >
                                <div
                                    class="h-full bg-stone-200 transition-all duration-300"
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
                                class="w-full h-1 bg-stone-800 overflow-hidden"
                            >
                                <div
                                    class="h-full bg-stone-200 transition-all duration-300"
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
                        class="flex-1 bg-stone-200 hover:bg-white text-stone-950 active:scale-[0.98] p-4 flex items-center justify-center gap-3 transition-all font-medium"
                    >
                        {$_('pages.boostrapWin.retry')}
                    </button>
                {/if}
                <button
                    on:click={cancel}
                    class="{error
                        ? 'flex-1'
                        : 'w-full'} bg-stone-900 hover:bg-stone-800 active:scale-[0.98] p-4 flex items-center justify-center gap-3 transition-all border border-stone-800 hover:border-stone-700 text-stone-200"
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
