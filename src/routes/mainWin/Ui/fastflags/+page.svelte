<script lang="ts">
    console.log('component module loaded')
    import { onMount, tick } from 'svelte'
    import {
        getLatestVersion,
        getCurrentInstallation,
    } from '$lib/downloadRoblox'
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
    import SettingCard from '$lib/components/molecules/SettingCard.svelte'
    import Dropdown from '$lib/components/molecules/Dropdown.svelte'
    import Switch from '$lib/components/atoms/Switch.svelte'
    import Textbox from '$lib/components/atoms/Textbox.svelte'
    import { RefreshCw, Save, SquarePen } from '@lucide/svelte'
    import ClickableCard from '$lib/components/molecules/ClickableCard.svelte'
    import { load, type Store } from '@tauri-apps/plugin-store'

    let flags: Record<string, string> = $state({})
    let appType: AppType = $state('player')
    let loaded = $state(false)

    let msaaValue: string = $state('0')
    let textureQuality: string = $state('-1')
    let pauseVoxelizer: boolean = $state(false)
    let wavingGrass: string = $state('0')
    let lowMeshQuality: boolean = $state(false)
    let useFastFlag: boolean = $state(true)
    let graySky: boolean = $state(false)
    let config: Store | undefined

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
        config = await load('config')
        useFastFlag = (await config.get('useFlag')) ?? true
        appType = (get(launchAppType) as AppType) || 'player'

        flags = await getFastFlags(appType)

        msaaValue = flags[MSAA_KEY] ?? '0'
        textureQuality = flags[TEXTURE_KEY] ?? '-1'
        pauseVoxelizer = flags[VOXELIZER_KEY] === 'true'
        wavingGrass = flags[GRASS_KEY] ?? '0'
        lowMeshQuality =
            LOW_MESH_KEYS.every((k) => flags[k] === '0') &&
            LOW_MESH_KEYS.some((k) => k in flags)
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

    function handleEditorClick() {
        goto('/mainWin/Ui/fastflags/editor')
    }

    let saveQueue: Promise<void> = Promise.resolve()
    async function save() {
        if (!loaded) return

        saveQueue = saveQueue.then(async () => {
            try {
                await tick()
                console.log('[Preset] Saving state:', {
                    msaaValue,
                    textureQuality,
                    pauseVoxelizer,
                    wavingGrass,
                    lowMeshQuality,
                    graySky,
                })

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
                if (!config) {
                    throw new Error('Config store is not loaded')
                }
                await saveFastFlags(flags, appType)
                await config.set('useFlag', useFastFlag)
                console.log('[Preset] Save successful')
            } catch (e) {
                console.error('[Preset] Save failed:', e)
            }
        })

        await saveQueue
    }
</script>

<div class="flex flex-col gap-8">
    <div class="flex flex-col gap-3">
        <ClickableCard
            title={$_('pages.fastflag.selectScreeen.editCard.title')}
            description={$_(
                'pages.fastflag.selectScreeen.editCard.description'
            )}
            icon={SquarePen}
            onclick={handleEditorClick}
        ></ClickableCard>

        <div>
            <h1 class="text-2xl font-bold tracking-tight text-stone-100">
                {$_('pages.fastflag.preset.preset')}
            </h1>
            <p class="text-stone-400 mt-1">
                {$_('pages.fastflag.preset.description')}
            </p>
        </div>

        <SettingCard
            title={$_('pages.fastflag.preset.disableFastFlag.title')}
            description={$_(
                'pages.fastflag.preset.disableFastFlag.description'
            )}
        >
            {#snippet action()}
                <Switch
                    checked={useFastFlag}
                    onchange={(e) => {
                        useFastFlag = e
                        save()
                    }}
                />
            {/snippet}
        </SettingCard>

        <SettingCard
            title={$_('pages.fastflag.preset.msaaCard.title')}
            description={$_('pages.fastflag.preset.msaaCard.description')}
        >
            {#snippet action()}
                <Dropdown
                    value={msaaValue}
                    options={msaaItems}
                    onchange={(e) => {
                        msaaValue = e
                        save()
                    }}
                />
            {/snippet}
        </SettingCard>

        <SettingCard
            title={$_('pages.fastflag.preset.pauseVoxelizerCard.title')}
            description={$_(
                'pages.fastflag.preset.pauseVoxelizerCard.description'
            )}
        >
            {#snippet action()}
                <Switch
                    checked={pauseVoxelizer}
                    onchange={(e) => {
                        pauseVoxelizer = e
                        save()
                    }}
                />
            {/snippet}
        </SettingCard>

        <SettingCard
            title={$_('pages.fastflag.preset.grassAnimationCard.title')}
            description={$_(
                'pages.fastflag.preset.grassAnimationCard.description'
            )}
        >
            {#snippet action()}
                <div class="w-50">
                    <Textbox
                        bind:value={wavingGrass}
                        onchange={(e) => {
                            wavingGrass = String(e)
                            save()
                        }}
                        onenter={(e) => {
                            wavingGrass = String(e)
                            save()
                        }}
                    />
                </div>
            {/snippet}
        </SettingCard>

        <SettingCard
            title={$_(
                'pages.fastflag.preset.overwriteTextureQualityCard.title'
            )}
            description={$_(
                'pages.fastflag.preset.overwriteTextureQualityCard.description'
            )}
        >
            {#snippet action()}
                <Dropdown
                    value={textureQuality}
                    options={textureQualityItems}
                    onchange={(e) => {
                        textureQuality = e
                        save()
                    }}
                />
            {/snippet}
        </SettingCard>

        <SettingCard
            title={$_('pages.fastflag.preset.lowMeshQualityCard.title')}
            description={$_(
                'pages.fastflag.preset.lowMeshQualityCard.description'
            )}
        >
            {#snippet action()}
                <Switch
                    checked={lowMeshQuality}
                    onchange={(e) => {
                        lowMeshQuality = e
                        save()
                    }}
                />
            {/snippet}
        </SettingCard>

        <SettingCard
            title={$_('pages.fastflag.preset.graySkyCard.title')}
            description={$_('pages.fastflag.preset.graySkyCard.description')}
        >
            {#snippet action()}
                <Switch
                    checked={graySky}
                    onchange={(e) => {
                        graySky = e
                        save()
                    }}
                />
            {/snippet}
        </SettingCard>
    </div>
</div>
