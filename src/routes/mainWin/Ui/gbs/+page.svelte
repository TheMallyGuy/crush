<script lang="ts">
    import Button from '$lib/components/atoms/Button.svelte'
    import Slider from '$lib/components/atoms/Slider.svelte'
    import Switch from '$lib/components/atoms/Switch.svelte'
    import Textbox from '$lib/components/atoms/Textbox.svelte'
    import SettingCard from '$lib/components/molecules/SettingCard.svelte'
    import { invoke } from '@tauri-apps/api/core'
    import { info } from '@tauri-apps/plugin-log'
    import { onMount } from 'svelte'

    let fpsLimit: number = 67
    let graphicQuaity: number = 0
    let fullscreen: boolean = false
    let reducedMotion: boolean = false

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
        await invoke('write_gbs', { content: rawGBS })
    }

    async function saveSetting(name: string, value: string) {
        updateGBS(name, value)
        await saveGBS()
    }

    let loaded = false

    onMount(async () => {
        rawGBS = await invoke<string>('get_gbs')
        gbs = parseGBS(rawGBS)

        // fuck types

        fpsLimit = gbs.FramerateCap
        graphicQuaity = gbs.SavedQualityLevel
        fullscreen = gbs.Fullscreen
        reducedMotion = gbs.ReducedMotion

        loaded = true // guard
    })
</script>

<div class="flex flex-col gap-8">
    <div class="flex items-center justify-between">
        <div>
            <h1 class="text-3xl font-bold tracking-tight text-stone-100">
                Global Basic Settings
            </h1>
            <p class="text-stone-400 mt-1">
                Configure Roblox's default settings.
            </p>
        </div>
    </div>

    <div class="flex flex-col gap-3">
        <div class="flex flex-row gap-3">
            <SettingCard
                title="FPS limit"
                description="Configure roblox's fps limit. Notice that Roblox only built for 240 fps max, going higher might break Roblox."
            >
                <Textbox
                    slot="footer"
                    bind:value={fpsLimit}
                    on:change={() =>
                        saveSetting('FramerateCap', String(fpsLimit))}
                />
            </SettingCard>

            <SettingCard
                title="Graphics Quality Level"
                description="description i guess"
            >
                <p slot="action">{graphicQuaity}</p>

                <Slider
                    bind:value={graphicQuaity}
                    min={0}
                    max={10}
                    slot="footer"
                    onchange={() =>
                        saveSetting('SavedQualityLevel', String(graphicQuaity))}
                />
            </SettingCard>

            <SettingCard
                title="Reduced Motion"
                description="Reduce the ui's motion"
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
            <SettingCard title="Fullscreen" description="roblox fullscreen">
                <Switch
                    slot="footer"
                    bind:checked={fullscreen}
                    on:change={() =>
                        saveSetting('Fullscreen', String(fullscreen))}
                />
            </SettingCard>

            <SettingCard
                title="Auto generated settings"
                description="Generated from roblox's GBS file. Process with caution."
            >
                <Button slot="footer" variant="secondary">Open</Button>
            </SettingCard>
        </div>
    </div>
</div>
