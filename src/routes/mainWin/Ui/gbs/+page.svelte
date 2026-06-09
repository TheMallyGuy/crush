<script lang="ts">
    import { goto } from '$app/navigation'
    import Button from '$lib/components/atoms/Button.svelte'
    import Slider from '$lib/components/atoms/Slider.svelte'
    import Switch from '$lib/components/atoms/Switch.svelte'
    import Textbox from '$lib/components/atoms/Textbox.svelte'
    import SettingCard from '$lib/components/molecules/SettingCard.svelte'
    import { getCurrentInstallation } from '$lib/downloadRoblox'
    import type { Installation } from '$lib/types'
    import { invoke } from '@tauri-apps/api/core'
    import { load } from '@tauri-apps/plugin-store'
    import { onMount } from 'svelte'
    import { _ } from 'svelte-i18n'

    let fpsLimit: number = 67
    let graphicQuaity: number = 0
    let fullscreen: boolean = false
    let reducedMotion: boolean = false
    let currentInstallation: Awaited<ReturnType<typeof getCurrentInstallation>> | undefined

    let rawGBS: string = ''
    let gbs: Record<string, SettingValue> = {}

    type Vector2 = { x: number; y: number }
    type SettingValue = string | number | boolean | Vector2

    function parseGBS(xml: string): Record<string, SettingValue> {
        const parser = new DOMParser()
        const doc = parser.parseFromString(xml, 'text/xml')
        const settings: Record<string, SettingValue> = {}

        doc.querySelectorAll('Properties > *').forEach((el) => {
            const name = el.getAttribute('name')
            if (!name) return

            const tag = el.tagName.toLowerCase()
            const text = el.textContent?.trim() ?? ''

            if (tag === 'bool') {
                settings[name] = text === 'true'
            } else if (tag === 'int' || tag === 'int64' || tag === 'token') {
                settings[name] = parseInt(text)
            } else if (tag === 'float') {
                settings[name] = parseFloat(text)
            } else if (tag === 'vector2') {
                settings[name] = {
                    x: parseFloat(el.querySelector('X')?.textContent ?? '0'),
                    y: parseFloat(el.querySelector('Y')?.textContent ?? '0'),
                }
            } else {
                settings[name] = text
            }
        })

        return settings
    }

    // one trillion helpers

    function updateGBS(name: string, value: string) {
        const parser = new DOMParser()
        const doc = parser.parseFromString(rawGBS, 'text/xml')

        const el = doc.querySelector(`[name="${name}"]`)
        if (el) el.textContent = value

        rawGBS = new XMLSerializer().serializeToString(doc)
    }

    async function saveGBS() {
        const store = await load('config.json')
        const ins = await store.get<Installation>('installation')
        
        await invoke('write_gbs', { content: rawGBS, vng: ins?.vng })
    }

    async function saveSetting(name: string, value: string) {
        updateGBS(name, value)
        await saveGBS()
    }

    let loaded = false

    onMount(async () => {
        const store = await load('config.json')
        const ins = await store.get<Installation>('installation')

        rawGBS = await invoke<string>('get_gbs', { vng: ins?.vng })
        gbs = parseGBS(rawGBS)

        // fuck types

        fpsLimit = gbs.FramerateCap as number
        graphicQuaity = gbs.SavedQualityLevel as number
        fullscreen = gbs.Fullscreen as boolean
        reducedMotion = gbs.ReducedMotion as boolean

        currentInstallation = await getCurrentInstallation('player')
        loaded = true // guard
    })
</script>

<div class="flex flex-col gap-8">
    <div class="flex items-center justify-between">
        <div>
            <h1 class="text-3xl font-bold tracking-tight text-stone-100">
                {$_('pages.gbs.gbs')}
            </h1>
            <p class="text-stone-400 mt-1">
                {$_('pages.gbs.description')}
            </p>
        </div>
    </div>

    {#if currentInstallation?.exists}
        <div class="flex flex-col gap-3">
            <div class="flex flex-row gap-3">
                <SettingCard
                    title={$_('pages.gbs.fpsLimitCard.title')}
                    description={$_('pages.gbs.fpsLimitCard.description')}
                >
                    <Textbox
                        slot="footer"
                        bind:value={fpsLimit}
                        on:change={() =>
                            saveSetting('FramerateCap', String(fpsLimit))}
                    />
                </SettingCard>

                <SettingCard
                    title={$_('pages.gbs.graphicsQualityLevel.title')}
                    description={$_(
                        'pages.gbs.graphicsQualityLevel.description'
                    )}
                >
                    <p slot="action">{graphicQuaity}</p>

                    <Slider
                        bind:value={graphicQuaity}
                        min={0}
                        max={10}
                        slot="footer"
                        onchange={() =>
                            saveSetting(
                                'SavedQualityLevel',
                                String(graphicQuaity)
                            )}
                    />
                </SettingCard>

                <SettingCard
                    title={$_('pages.gbs.reducedMotionCard.title')}
                    description={$_('pages.gbs.reducedMotionCard.description')}
                >
                    <Switch
                        slot="footer"
                        bind:checked={reducedMotion}
                        on:change={() =>
                            saveSetting('ReducedMotion', String(reducedMotion))}
                    />
                </SettingCard>
            </div>

            <div class="flex flex-row gap-3">
                <SettingCard
                    title={$_('pages.gbs.fullscreenCard.title')}
                    description={$_('pages.gbs.fullscreenCard.description')}
                >
                    <Switch
                        slot="footer"
                        bind:checked={fullscreen}
                        on:change={() =>
                            saveSetting('Fullscreen', String(fullscreen))}
                    />
                </SettingCard>

                <SettingCard
                    title={$_('pages.gbs.autoGeneratedGbsCard.title')}
                    description={$_(
                        'pages.gbs.autoGeneratedGbsCard.description'
                    )}
                >
                    <Button
                        slot="footer"
                        variant="secondary"
                        on:click={() => {
                            goto('./gbs/generated')
                        }}>{$_('pages.gbs.autoGeneratedGbsCard.button')}</Button
                    >
                </SettingCard>
            </div>
        </div>
    {:else}
        <p class="max-h-screen text-center">
            {$_('pages.gbs.autoGeneratedGbs.noInstallation')}
        </p>
    {/if}
</div>
