<script lang="ts">
console.log('component module loaded')
    import { onMount, tick } from 'svelte'
    import { getLatestVersion, getCurrentInstallation } from '$lib/downloadRoblox'
    import {
        getFastFlags,
        saveFastFlags,
    } from '$lib/fastflag/fastflagManagement'
    import { invoke } from '@tauri-apps/api/core'
    import { _ } from 'svelte-i18n'
    import { goto } from '$app/navigation'
    import { get } from 'svelte/store'
    import { launchAppType } from '$lib/stores/launchAppType'
    import type { AppType } from '$lib/types'
    import Button from '$lib/components/atoms/Button.svelte'
    import SettingCard from '$lib/components/molecules/SettingCard.svelte'
    import Dropdown from '$lib/components/molecules/Dropdown.svelte'
    import Switch from '$lib/components/atoms/Switch.svelte'
    import Textbox from '$lib/components/atoms/Textbox.svelte'
    import { RefreshCw, Save } from '@lucide/svelte'

    let flags: Record<string, string> = {}
    let version = ''
    let appType: AppType = 'player'
    let loaded = false

    let msaaValue: string = '0'
    let textureQuality: string = '-1'
    let pauseVoxelizer: boolean = false
    let wavingGrass: string = '0'
    let lowMeshQuality: boolean = false
    let graySky: boolean = false
    const MSAA_KEY = 'FIntDebugForceMSAASamples'
    const TEXTURE_KEY = 'DFIntTextureQualityOverride'
    const VOXELIZER_KEY = 'DFFlagDebugPauseVoxelizer'
    const GRASS_KEY = 'FIntGrassMovementReducedMotionFactor'
    const GRAY_SKY = 'FFlagDebugSkyGray'
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

    async function loadState() {
        loaded = false
        appType = (get(launchAppType) as AppType) || 'player'

        flags = await getFastFlags(appType)

        msaaValue = flags[MSAA_KEY] ?? '0'
        textureQuality = flags[TEXTURE_KEY] ?? '-1'
        pauseVoxelizer = flags[VOXELIZER_KEY] === 'true'
        wavingGrass = flags[GRASS_KEY] ?? '0'
        lowMeshQuality = LOW_MESH_KEYS.every((k) => flags[k] === '0') && LOW_MESH_KEYS.some(k => k in flags)
        graySky = flags[GRAY_SKY] === 'true'

        loaded = true
    }

    onMount(async () => {
        await loadState()
        invoke('set_rpc', {
            details: $_('rpc.general'),
            stateText: $_('rpc.fastflag'),
        })
    })

    let saveQueue: Promise<void> = Promise.resolve()
    async function save() {
        if (!loaded) return
        
        saveQueue = saveQueue.then(async () => {
            try {
                await tick()
                console.log('[Preset] Saving state:', { msaaValue, textureQuality, pauseVoxelizer, wavingGrass, lowMeshQuality, graySky })
                
                const latestFlags = await getFastFlags(appType)
                const newFlags = { ...latestFlags }

                newFlags[MSAA_KEY] = msaaValue
                newFlags[TEXTURE_KEY] = textureQuality
                newFlags[VOXELIZER_KEY] = pauseVoxelizer ? 'true' : 'false'
                newFlags[GRASS_KEY] = wavingGrass
                newFlags[GRAY_SKY] = graySky ? 'true' : 'false'
                
                for (const key of LOW_MESH_KEYS) {
                    newFlags[key] = lowMeshQuality ? '0' : '1000' // 1000 is a safe "normal" distance
                }

                flags = newFlags
                await saveFastFlags(flags, appType)
                console.log('[Preset] Save successful')
            } catch (e) {
                console.error('[Preset] Save failed:', e)
            }
        })
        
        await saveQueue
    }
</script>

<div class="flex flex-col gap-8">
    <div class="flex items-center justify-between">
        <div>
            <h1 class="text-3xl font-bold tracking-tight text-foreground">
                {$_('pages.fastflag.preset.preset')}
            </h1>
            <p class="text-muted-foreground mt-1">
                {$_('pages.fastflag.preset.description')}
            </p>
        </div>
        <div class="flex items-center gap-2">
            <Button variant="secondary" on:click={loadState}>
                <RefreshCw class="h-4 w-4 mr-2" />
                <span>Refresh</span>
            </Button>
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
                value={msaaValue}
                options={msaaItems}
                on:change={(e) => { msaaValue = e.detail; save(); }}
            />
        </SettingCard>

        <SettingCard
            title={$_('pages.fastflag.preset.pauseVoxelizerCard.title')}
            description={$_(
                'pages.fastflag.preset.pauseVoxelizerCard.description'
            )}
        >
            <Switch slot="action" checked={pauseVoxelizer} on:change={(e) => { pauseVoxelizer = e.detail; save(); }}/>
        </SettingCard>

        <SettingCard
            title={$_('pages.fastflag.preset.grassAnimationCard.title')}
            description={$_(
                'pages.fastflag.preset.grassAnimationCard.description'
            )}
        >
            <div slot="action" class="w-50">
                <Textbox value={wavingGrass} on:change={(e) => { wavingGrass = e.detail; save(); }} on:enter={(e) => { wavingGrass = e.detail; save(); }} />
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
                value={textureQuality}
                options={textureQualityItems}
                on:change={(e) => { textureQuality = e.detail; save(); }}
            />
        </SettingCard>

        <SettingCard
            title={$_('pages.fastflag.preset.lowMeshQualityCard.title')}
            description={$_(
                'pages.fastflag.preset.lowMeshQualityCard.description'
            )}
        >
            <Switch slot="action" checked={lowMeshQuality} on:change={(e) => { lowMeshQuality = e.detail; save(); }} />
        </SettingCard>

        <SettingCard
            title={$_('pages.fastflag.preset.graySkyCard.title')}
            description={$_('pages.fastflag.preset.graySkyCard.description')}
        >
            <Switch slot="action" checked={graySky} on:change={(e) => { graySky = e.detail; save(); }} />
        </SettingCard>
    </div>
</div>
