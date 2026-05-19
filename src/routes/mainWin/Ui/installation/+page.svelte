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

    let version: string
    let forceReinstall: boolean
    let dontUpdate: boolean
    let parallel: number

    async function loadConfig() {
        const store = await load('config.json')
        const savedInstallation = await store.get<Installation>('installation')

        if (savedInstallation) {
            version = savedInstallation.version ?? 'latest'
            forceReinstall = savedInstallation.forceReinstall ?? false
            dontUpdate = savedInstallation.dontUpdate ?? false
            parallel = savedInstallation.parallel ?? 4
        }
    }

    onMount(async () => {
        console.log('test')

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
            forceReinstall,
            dontUpdate,
            parallel,
        }

        await store.set('installation', newInstallation)

        await store.save()
    }
</script>

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
            title="Versions Manager"
            description="Manage multiple roblox versions with speed"
            icon={Rocket}
        >
            <Button slot="action" variant="secondary" on:click={() => {goto("./installation/versionManager")}}>
                Open
            </Button>
        </SettingCard>

        <SettingCard
            title={$_("pages.installations.parallelDownloadingCard.title")}
            description={$_("pages.installations.parallelDownloadingCard.description")}
            icon={Folders}
        >
            <Textbox
                slot="action"
                class="w-30 h-8 text-sm"
                bind:value={parallel}
                on:change={handleChanges}
            />
        </SettingCard>

        <SettingCard
            title={$_('pages.installations.forceReinstallCard.title')}
            description={$_(
                'pages.installations.forceReinstallCard.description'
            )}
            icon={HardDriveDownload}
        >
            <Switch
                slot="action"
                bind:checked={forceReinstall}
                on:change={handleChanges}
            />
        </SettingCard>

        <SettingCard
            title={$_('pages.installations.dontUpdateCard.title')}
            description={$_('pages.installations.dontUpdateCard.description')}
            icon={CircleFadingArrowUp}
        >
            <Switch
                slot="action"
                bind:checked={dontUpdate}
                on:change={handleChanges}
            />
        </SettingCard>
    </div>
</div>
