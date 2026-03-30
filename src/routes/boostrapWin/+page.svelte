<!--
    NOTICE DO NOT REMOVE!!!
    This page is in fact, genarated by Gemini & Claude AI. Although I'm good at making things, but using these tools to make this easier.
    You can call this "AI slop" or whatever, but its a developer tools & I'd to use it.
    Any opinion on this?  
-->

<script lang="ts">
    import { downloadRoblox, type ProgressEvent } from '$lib/downloadRoblox'
    import { launchPlayer, applyMods } from '$lib/launchRoblox'
    import { relaunch } from '@tauri-apps/plugin-process'
    import { getCurrentWindow, LogicalSize } from '@tauri-apps/api/window'
    import { onMount, onDestroy } from 'svelte'
    import { themeStore, resolveAsset } from '$lib/theme/themeStore'
    import type { ThemeState } from '$lib/theme/themeStore'
    import type { BootstrapElement } from '$lib/theme/xmlParser'
    import { invoke } from '@tauri-apps/api/core'
    import { listen } from '@tauri-apps/api/event'
    import { deepLinkUrl } from '$lib/stores/deeplink'
    import { goto } from '$app/navigation'
    import { get } from 'svelte/store'
    import { page } from '$app/stores'

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

    let textValues: Record<string, string> = {}

    function handleProgress(e: ProgressEvent) {
        if (e.type === 'status') {
            status = e.message
            textValues['StatusText'] = e.message
        } else if (e.type === 'download') {
            downloadFile = e.file
            downloadDone = e.done
            downloadTotal = e.total
        } else if (e.type === 'extract') {
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

        if (state.isHtmlTheme) {
            await win.setSize(new LogicalSize(600, 400))
            await win.center()
        } else if (state.config) {
            for (const el of state.config.elements) {
                if (el.name) {
                    textValues[el.name] =
                        el.name === 'StatusText' ? 'Preparing...' : ''
                }
            }
            textValues = { ...textValues }

            const { config } = state
            await win.setSize(new LogicalSize(config.width, config.height))
            await win.center()

            const titleBar = state.config.elements.find(
                (e) => e.type === 'TitleBar'
            )
            if (titleBar?.props?.Title) {
                await win.setTitle(titleBar.props.Title)
            }
        }
        await win.show()
    }

    onMount(async () => {
        await setupWindow()

        let version = await downloadRoblox(handleProgress)
        done = true
        status = 'Applying modification'
        await applyMods(version)
        status = 'Launching'
        textValues['StatusText'] = 'Launching'
        textValues = { ...textValues }
        const url: string = $deepLinkUrl ?? ''
        await launchPlayer(version, url)
        await invoke('watch_logs')

        await invoke('create_or_focus_window', {
            label: 'crushBoostrapChoiceWindow',
            url: 'mainWin/choiceWin',
            title: 'Crush',
            width: 500.0,
            height: 250.0,
            minWidth: 500.0,
            minHeight: 250.0,
        })

        setTimeout(() => {
            // wait before killing to prevent crash
            getCurrentWindow().close()
        }, 100)
    })

    function getPosStyle(h?: string, v?: string) {
        const styles = []
        const transforms = []

        if (h === 'Right') {
            styles.push('right:0')
        } else if (h === 'Center') {
            styles.push('left:50%')
            transforms.push('translateX(-50%)')
        } else {
            styles.push('left:0')
        }

        if (v === 'Bottom') {
            styles.push('bottom:0')
        } else if (v === 'Center') {
            styles.push('top:50%')
            transforms.push('translateY(-50%)')
        } else {
            styles.push('top:0')
        }

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
        const sides = {
            top: 'top',
            right: 'right',
            bottom: 'bottom',
            left: 'left',
        }
        return Object.entries(sides)
            .map(([key, label]) => {
                const val = m[key as keyof typeof m]
                return val ? `margin-${label}:${val}px;` : ''
            })
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
            const newScript = document.createElement('script')
            Array.from(oldScript.attributes).forEach((attr) =>
                newScript.setAttribute(attr.name, attr.value)
            )
            newScript.appendChild(document.createTextNode(oldScript.innerHTML))
            oldScript.parentNode?.replaceChild(newScript, oldScript)
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
            class="relative overflow-hidden h-screen w-screen bg-black"
            use:mountHtml={state.customHtml || ''}
        ></div>
    {:else if cfg}
        <div
            class="relative overflow-hidden h-screen w-screen"
            style="
                background:{isDark ? '#000' : '#fff'};
                color:{isDark ? '#fff' : '#000'};
                border-radius:{noRound ? '0' : '8px'};
            "
        >
            {#each elements as el}
                {#if el.type === 'Image' || el.props.source}
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
        class="relative h-screen bg-stone-950 text-white selection:bg-stone-800"
    >
        <div
            class="absolute inset-0 flex flex-col items-center justify-center text-center p-3 pb-24 gap-6"
        >
            <div>
                <h1 class="text-4xl tracking-tight text-stone-100 font-medium">
                    Crush
                </h1>
                <p class="text-stone-400 mt-2">{status}</p>
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
        <div class="absolute bottom-6 left-0 w-full flex justify-center p-3">
            <div class="w-full max-w-sm">
                <button
                    on:click={cancel}
                    class="w-full bg-stone-900 hover:bg-stone-800 active:scale-[0.98] rounded-lg p-4 flex items-center justify-center gap-3 transition-all border border-stone-800 hover:border-stone-700 text-stone-200"
                >
                    <span class="font-medium">Cancel</span>
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
