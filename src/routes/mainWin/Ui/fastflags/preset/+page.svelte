<script lang="ts">
    import { onMount } from 'svelte'
    import { getLatestVersion } from '$lib/downloadRoblox'
    import {
        getFastFlags,
        saveFastFlags,
    } from '$lib/fastflag/fastflagManagement'
    import { invoke } from '@tauri-apps/api/core'
    import { _ } from 'svelte-i18n'
    import { goto } from '$app/navigation'
    import Button from '$lib/components/atoms/Button.svelte'
    import SettingCard from '$lib/components/molecules/SettingCard.svelte'
    import Dropdown from '$lib/components/molecules/Dropdown.svelte'
    import Switch from '$lib/components/atoms/Switch.svelte'
    import Textbox from '$lib/components/atoms/Textbox.svelte'

    let flags: Record<string, string> = {}
    let version = ''

    let msaaropdownItems = [
        { value: '0', label: 'x0' },
        { value: '1', label: 'x1' },
        { value: '2', label: 'x2' },
        { value: '4', label: 'x4' },
        { value: '8', label: 'x8' },
    ]
    let msaaDropdownDefault: string = '0'

    let textureQualityItems = [
        { value: '-1', label: 'default' },
        { value: '0', label: '0' },
        { value: '1', label: '1' },
        { value: '2', label: '2' },
        { value: '3', label: '3' },
        { value: '4', label: '4' },
    ]
    let textureQualityDefault: string = '-1'

    onMount(async () => {
        await invoke('set_rpc', {
            details: $_('rpc.general'),
            stateText: $_('rpc.fastflag'),
        })
        version = await getLatestVersion()
        flags = await getFastFlags(version)
    })

    async function handleDelete(event: CustomEvent<string>) {
        const { [event.detail]: _, ...rest } = flags
        flags = rest
        await saveFastFlags(version, flags)
    }

    async function handleAdd(
        event: CustomEvent<{ name: string; value: string }>
    ) {
        const { name, value } = event.detail
        if (name in flags) return
        flags = { ...flags, [name]: value }
        await saveFastFlags(version, flags)
    }
</script>

<div class="flex flex-col gap-8">
    <div class="flex items-center justify-between">
        <div>
            <h1 class="text-3xl font-bold tracking-tight text-stone-100">
                Presets
            </h1>
            <p class="text-stone-400 mt-1">
                Use our built-in fast-flag preset
            </p>
        </div>
        <div class="flex items-center gap-2">
            <Button variant="secondary" onclick={() => goto('../fastflags')}>
                {$_('pages.integrations.gameHistory.backToIntegrations')}
            </Button>
        </div>
    </div>

    <div class="flex flex-col gap-3">
        <SettingCard
            title="Anti-Aliasing Quality (MSAA)"
            description="Smooths jagged edges. Higher values look better but may reduce performance."
        >
            <Dropdown
                slot="action"
                bind:value={msaaDropdownDefault}
                options={msaaropdownItems}
            />
        </SettingCard>

        <SettingCard
            title="Pause Voxelier"
            description="Stops voxelization to improve performance"
        >
            <Switch slot="action" />
        </SettingCard>

        <SettingCard
            title="Grass waving animation"
            description="Adjusts the intensity of grass movement.  (0 is no reduction, 999 or any higher number will freezes the grass, 0 & 1 slightly reduced motion and value that greater than 1 is strongly reduced motion, approaching stillness)"
        >
            <div slot="action" class="w-50">
                <Textbox />
            </div>
        </SettingCard>

        <SettingCard
            title="Overwrite texture quality level"
            description="Forces a custom texture quality level."
        >
            <Dropdown
                slot="action"
                bind:value={textureQualityDefault}
                options={textureQualityItems}
            />
        </SettingCard>

        <SettingCard
            title="Enable Low Quality Meshes"
            description="May improves performance."
        >
            <Switch slot="action" />
        </SettingCard>
    </div>
</div>
