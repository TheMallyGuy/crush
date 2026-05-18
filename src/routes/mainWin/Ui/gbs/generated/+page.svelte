<script lang="ts">
    import { goto } from '$app/navigation'
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

    function getType(value: undefined) {
        if (value === null) return 'null'
        if (value === undefined) return 'undefined'
        if (Array.isArray(value)) return 'array'
        return typeof value
    }

    let loaded = false

    onMount(async () => {
        rawGBS = await invoke<string>('get_gbs')
        gbs = parseGBS(rawGBS)

        loaded = true // guard
    })
</script>

<div class="flex flex-col gap-8">
    <div class="flex items-center justify-between">
        <div>
            <h1 class="text-3xl font-bold tracking-tight text-stone-100">
                Generated Global Basic Settings
            </h1>
            <p class="text-stone-400 mt-1">
                Auto generated based on the GBS file.
            </p>
        </div>
        <div class="flex items-center gap-2">
            <Button variant="secondary" onclick={() => goto('../gbs')}>
                Back
            </Button>
        </div>
    </div>

    <div class="grid grid-cols-3 gap-3">
        {#each Object.entries(gbs) as [key, value]}
            <SettingCard title={key}>
                <p slot="footer">
                    {#if getType(value) === 'number' || getType(value) === 'string'}
                        <Textbox
                            value={String(gbs[key])}
                            on:change={(e) => saveSetting(key, e.target.value)}
                        />
                    {:else if getType(value) === 'boolean'}
                        <Switch
                            checked={gbs[key]}
                            on:change={() => {
                                gbs[key] = !gbs[key]
                                saveSetting(key, String(gbs[key]))
                            }}
                        />
                    {/if}
                </p>
            </SettingCard>
        {/each}
    </div>
</div>
