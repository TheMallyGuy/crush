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
    let loaded = false

    let msaaValue: string = '0'
    let textureQuality: string = '-1'
    let pauseVoxelizer: boolean = false
    let wavingGrass: string = '0'
    let lowMeshQuality: boolean = false

    const MSAA_KEY = 'FIntDebugForceMSAASamples'
    const TEXTURE_KEY = 'DFIntTextureQualityOverride'
    const VOXELIZER_KEY = 'DFFlagDebugPauseVoxelizer'
    const GRASS_KEY = 'FIntGrassMovementReducedMotionFactor'
    const LOW_MESH_KEYS = [
        'DFIntCSGLevelOfDetailSwitchingDistance',
        'DFIntCSGLevelOfDetailSwitchingDistanceL12',
        'DFIntCSGLevelOfDetailSwitchingDistanceL23',
        'DFIntCSGLevelOfDetailSwitchingDistanceL34',
    ]

    const msaaItems = [
        { value: '0', label: 'x0' },
        { value: '1', label: 'x1' },
        { value: '2', label: 'x2' },
        { value: '4', label: 'x4' },
        { value: '8', label: 'x8' },
    ]

    const textureQualityItems = [
        {
            value: '-1',
            label: $_(
                'pages.fastflag.preset.overwriteTextureQualityCard.dropdownDefault'
            ),
        },
        { value: '0', label: '0' },
        { value: '1', label: '1' },
        { value: '2', label: '2' },
        { value: '3', label: '3' },
        { value: '4', label: '4' },
    ]

    onMount(async () => {
        await invoke('set_rpc', {
            details: $_('rpc.general'),
            stateText: $_('rpc.fastflag'),
        })
        version = await getLatestVersion()
        flags = await getFastFlags(version)

        msaaValue = flags[MSAA_KEY] ?? '0'
        textureQuality = flags[TEXTURE_KEY] ?? '-1'
        pauseVoxelizer = flags[VOXELIZER_KEY] === 'true'
        wavingGrass = flags[GRASS_KEY] ?? '0'
        lowMeshQuality = LOW_MESH_KEYS.every((k) => flags[k] === '0')

        loaded = true
    })

    async function setFlag(key: string, value: string | null) {
        if (value === null) {
            const { [key]: _, ...rest } = flags
            flags = rest
        } else {
            flags = { ...flags, [key]: value }
        }
        await saveFastFlags(version, flags)
    }

    $: if (loaded) setFlag(MSAA_KEY, msaaValue === '0' ? null : msaaValue)
    $: if (loaded)
        setFlag(TEXTURE_KEY, textureQuality === '-1' ? null : textureQuality)
    $: if (loaded) setFlag(VOXELIZER_KEY, pauseVoxelizer ? 'true' : null)
    $: if (loaded) setFlag(GRASS_KEY, wavingGrass === '0' ? null : wavingGrass)
    $: if (loaded) {
        for (const key of LOW_MESH_KEYS) {
            setFlag(key, lowMeshQuality ? '0' : null)
        }
    }
</script>

<div class="flex flex-col gap-8">
    <div class="flex items-center justify-between">
        <div>
            <h1 class="text-3xl font-bold tracking-tight text-stone-100">
                {$_('pages.fastflag.preset.preset')}
            </h1>
            <p class="text-stone-400 mt-1">
                {$_('pages.fastflag.preset.description')}
            </p>
        </div>
        <div class="flex items-center gap-2">
            <Button variant="secondary" onclick={() => goto('../fastflags')}>
                {$_('pages.fastflag.generalBack')}
            </Button>
        </div>
    </div>

    <div class="flex flex-col gap-3">
        <SettingCard
            title={$_('pages.fastflag.preset.msaaCard.title')}
            description={$_('pages.fastflag.preset.msaaCard.description')}
        >
            <Dropdown
                slot="action"
                bind:value={msaaValue}
                options={msaaItems}
            />
        </SettingCard>

        <SettingCard
            title={$_('pages.fastflag.preset.pauseVoxelizerCard.title')}
            description={$_(
                'pages.fastflag.preset.pauseVoxelizerCard.description'
            )}
        >
            <Switch slot="action" bind:checked={pauseVoxelizer} />
        </SettingCard>

        <SettingCard
            title={$_('pages.fastflag.preset.grassAnimationCard.title')}
            description={$_(
                'pages.fastflag.preset.grassAnimationCard.description'
            )}
        >
            <div slot="action" class="w-50">
                <Textbox bind:value={wavingGrass} />
            </div>
        </SettingCard>

        <SettingCard
            title={$_(
                'pages.fastflag.preset.overwriteTextureQualityCard.title'
            )}
            description={$_(
                'pages.fastflag.preset.overwriteTextureQualityCard.description'
            )}
        >
            <Dropdown
                slot="action"
                bind:value={textureQuality}
                options={textureQualityItems}
            />
        </SettingCard>

        <SettingCard
            title={$_('pages.fastflag.preset.lowMeshQualityCard.title')}
            description={$_(
                'pages.fastflag.preset.lowMeshQualityCard.description'
            )}
        >
            <Switch slot="action" bind:checked={lowMeshQuality} />
        </SettingCard>
    </div>
</div>
