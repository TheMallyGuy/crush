<script lang="ts">
    import { invoke } from '@tauri-apps/api/core'
    import SettingCard from '$lib/components/molecules/SettingCard.svelte'
    import Textbox from '$lib/components/atoms/Textbox.svelte'
    import {
        CircleFadingArrowUp,
        Folders,
        HardDriveDownload,
        Rocket,
    } from '@lucide/svelte'
    import { onMount } from 'svelte'
    import { load } from '@tauri-apps/plugin-store'
    import type { Installation } from '$lib/types'
    import { _ } from 'svelte-i18n'
    import Switch from '$lib/components/atoms/Switch.svelte'
    import Button from '$lib/components/atoms/Button.svelte'
    import { goto } from '$app/navigation'
    import Dialog from '$lib/components/molecules/Dialog.svelte'
    import { operating_system } from '$lib/stores/operating_system.svelte'

    const vngLogo = '/VNG.png'

    let warningVng = $state(false)

    let version: string = $state('')
    let forceReinstall: boolean = $state(false)
    let dontUpdate: boolean = $state(false)
    let vng: boolean = $state(false)
    let parallel: number = $state(5)

    let resolveWarning: ((value: boolean | null) => void) | null = null

    async function handleWarning() {
        if (!vng) {
            handleChanges()
            return
        }
        warningVng = true

        const ifContinue = await new Promise<boolean | null>((resolve) => {
            resolveWarning = resolve
        })

        warningVng = false

        if (ifContinue) {
            handleChanges()
        } else {
            vng = false
        }
    }

    async function loadConfig() {
        const store = await load('config.json')
        const savedInstallation = await store.get<Installation>('installation')

        if (savedInstallation) {
            version = savedInstallation.version ?? 'latest'
            forceReinstall = savedInstallation.forceReinstall ?? false
            dontUpdate = savedInstallation.dontUpdate ?? false
            vng = savedInstallation.vng ?? false
            parallel = savedInstallation.parallel ?? 4
        }
    }

    let disableWin: string | undefined = $state()
    let disableWinUn: string | undefined = $state()

    onMount(async () => {
        console.log('test')
        if ($operating_system == 'macos') {
            disableWin = $_('crossplatform.notAvailableWindowsOnly')
            disableWinUn = $_('crossplatform.notAvailableWindowsPlanned')
        }

        invoke('set_rpc', {
            details: $_('rpc.general'),
            stateText: $_('rpc.installation'),
        })

        await loadConfig()
        console.log('loaded')
    })

    async function handleChanges() {
        const store = await load('config.json')

        const newInstallation: Installation = {
            version,
            vng,
            forceReinstall,
            dontUpdate,
            parallel,
        }

        await store.set('installation', newInstallation)

        await store.save()
    }
</script>

<Dialog
    bind:open={warningVng}
    onclose={() => resolveWarning?.(false)}
    title={$_('pages.installations.dialogs.vngWarning.title')}
    description={$_('pages.installations.dialogs.vngWarning.description')}
>
    {#snippet actions()}
        <div>
            <Button
                variant="secondary"
                size="sm"
                onclick={() => resolveWarning?.(false)}
            >
                {$_('pages.installations.dialogs.vngWarning.cancel')}
            </Button>
            <Button
                variant="danger"
                size="sm"
                onclick={() => resolveWarning?.(true)}
            >
                {$_('pages.installations.dialogs.vngWarning.confirm')}
            </Button>
        </div>
    {/snippet}
</Dialog>

<div class="flex flex-col gap-8">
    <div class="flex items-center justify-between">
        <div>
            <h1 class="text-3xl font-bold tracking-tight text-stone-100">
                {$_('pages.installations.installations')}
            </h1>
            <p class="text-stone-400 mt-1">
                {$_('pages.installations.description')}
            </p>
        </div>
    </div>

    <div class="flex flex-col gap-3">
        <SettingCard
            title={$_('pages.installations.versionManagerCard.title')}
            description={$_(
                'pages.installations.versionManagerCard.description'
            )}
            disabled={disableWin}
            icon={Rocket}
        >
            {#snippet action()}
                <Button
                    variant="secondary"
                    onclick={() => {
                        goto('./installation/versionManager')
                    }}
                >
                    {$_('pages.installations.versionManagerCard.button')}
                </Button>
            {/snippet}
        </SettingCard>

        <SettingCard
            title={$_('pages.installations.parallelDownloadingCard.title')}
            description={$_(
                'pages.installations.parallelDownloadingCard.description'
            )}
            disabled={disableWin}
            icon={Folders}
        >
            {#snippet action()}
                <Textbox
                    class="w-30 h-8 text-sm"
                    bind:value={parallel}
                    onchange={handleChanges}
                />
            {/snippet}
        </SettingCard>

        <SettingCard
            title={$_('pages.installations.forceReinstallCard.title')}
            description={$_(
                'pages.installations.forceReinstallCard.description'
            )}
            icon={HardDriveDownload}
        >
            {#snippet action()}
                <Switch
                    bind:checked={forceReinstall}
                    onchange={handleChanges}
                />
            {/snippet}
        </SettingCard>

        <SettingCard
            title={$_('pages.installations.useVNGCard.title')}
            description={$_('pages.installations.useVNGCard.description')}
            disabled={disableWin}
            icon={vngLogo}
        >
            {#snippet action()}
                <Switch bind:checked={vng} onchange={handleWarning} />
            {/snippet}
        </SettingCard>

        <SettingCard
            title={$_('pages.installations.dontUpdateCard.title')}
            description={$_('pages.installations.dontUpdateCard.description')}
            icon={CircleFadingArrowUp}
        >
            {#snippet action()}
                <Switch bind:checked={dontUpdate} onchange={handleChanges} />
            {/snippet}
        </SettingCard>
    </div>
</div>
