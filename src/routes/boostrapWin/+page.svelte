<script lang="ts">
    import { downloadRoblox, type ProgressEvent } from '$lib/downloadRoblox'
    import { launchPlayer } from '$lib/launchRoblox'
    import { relaunch } from '@tauri-apps/plugin-process'
    import { getCurrentWindow, LogicalSize } from '@tauri-apps/api/window'
    import { onMount, onDestroy } from 'svelte'
    import { themeStore, resolveAsset } from '$lib/theme/themeStore'
    import type { ThemeState } from '$lib/theme/themeStore'
    import type { BootstrapElement } from '$lib/theme/xmlParser'

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
    }

    async function cancel() {
        await relaunch()
    }

    onMount(async () => {
        const win = getCurrentWindow()
        if (state) {
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
        } else {
            await win.setSize(new LogicalSize(630, 363))
            await win.center()
        }
        await win.show()

        let version = await downloadRoblox(handleProgress)
        done = true
        status = 'Launching'
        textValues['StatusText'] = 'Launching'
        textValues = { ...textValues }
        await launchPlayer(version)
    })

    function hPos(h?: string) {
        if (h === 'Right') return 'right:0;'
        if (h === 'Center') return 'left:50%;transform:translateX(-50%);'
        return 'left:0;'
    }
    function vPos(v?: string) {
        if (v === 'Bottom') return 'bottom:0;'
        if (v === 'Center') return 'top:50%;transform:translateY(-50%);'
        return 'top:0;'
    }
    function opStyle(op?: number) {
        return op !== undefined && op !== 0 ? `opacity:${op};` : ''
    }
    function marginStyle(m?: {
        top: number
        right: number
        bottom: number
        left: number
    }) {
        if (!m) return ''
        return [
            m.top ? `margin-top:${m.top}px;` : '',
            m.right ? `margin-right:${m.right}px;` : '',
            m.bottom ? `margin-bottom:${m.bottom}px;` : '',
            m.left ? `margin-left:${m.left}px;` : '',
        ]
            .filter(Boolean)
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
</script>

{#if state && cfg}
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
                    class="absolute object-cover"
                    style="
                        {hPos(el.hAlign)} {vPos(el.vAlign)}
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
                    class="absolute whitespace-nowrap pointer-events-none leading-none"
                    style="
                        {hPos(el.hAlign)} {vPos(el.vAlign)}
                        {opStyle(el.opacity)}
                        {marginStyle(el.margin)}
                        {el.props.Foreground
                        ? `color:${el.props.Foreground};`
                        : ''}
                        {el.props.FontSize
                        ? `font-size:${el.props.FontSize}px;`
                        : ''}
                        z-index:{el.zIndex ?? 0};
                    ">{textValues[el.name ?? ''] ?? el.props.Text ?? ''}</span
                >
            {:else if el.type === 'Button'}
                <button
                    id={el.name}
                    on:click={cancel}
                    class="absolute bg-transparent border-0 cursor-pointer focus:outline-none focus:ring-0"
                    style="
                        {hPos(el.hAlign)} {vPos(el.vAlign)}
                        {el.width ? `width:${el.width}px;` : ''}
                        {el.height ? `height:${el.height}px;` : ''}
                        {opStyle(el.opacity)}
                        {marginStyle(el.margin)}
                        z-index:{el.zIndex ?? 2};
                    ">{el.props.Content || el.props.Label || ''}</button
                >
            {:else if el.type === 'ProgressBar'}
                <div
                    class="absolute overflow-hidden bg-white/10"
                    style="
                        {hPos(el.hAlign)} {vPos(el.vAlign)}
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
            {/if}
        {/each}
    </div>
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
                                />
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
                                />
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
