<script lang="ts">
    import SettingCard from '$lib/components/molecules/SettingCard.svelte'
    import Button from '$lib/components/atoms/Button.svelte'
    import Switch from '$lib/components/atoms/Switch.svelte'
    import {
        Bell,
        Plug,
        History,
        CodeXml,
        View,
        Cpu,
        Bomb,
        Expand,
        BedSingle,
        Sparkles,
        ArrowDownToDot,
        ListStart,
        Server,
    } from '@lucide/svelte'
    import { invoke } from '@tauri-apps/api/core'
    import { onMount } from 'svelte'
    import { load } from '@tauri-apps/plugin-store'
    import type { Integrations, DiscordRpc, PriorityClass } from '$lib/types'
    import { _ } from 'svelte-i18n'
    import { goto } from '$app/navigation'
    import ExpandableSettingCard from '$lib/components/molecules/ExpandableSettingCard.svelte'
    import Dropdown from '$lib/components/molecules/Dropdown.svelte'
    import { getCurrentInstallation } from '$lib/downloadRoblox'
    import {
        setLaunchAtStartup,
        setMinimizeToTray,
    } from '$lib/localAppJsonHelper'

    let processPriorityItems = [
        { value: 'BELOW_NORMAL_PRIORITY_CLASS', label: 'BELOW_NORMAL' },
        { value: 'NORMAL_PRIORITY_CLASS', label: 'NORMAL' },
        { value: 'ABOVE_NORMAL_PRIORITY_CLASS', label: 'ABOVE_NORMAL' },
        { value: 'HIGH_PRIORITY_CLASS', label: 'HIGH' },
        { value: 'REALTIME_PRIORITY_CLASS', label: 'REALTIME' },
    ]
    let processPriority: PriorityClass = $state('NORMAL_PRIORITY_CLASS')

    let crashHandler = $state(false)
    let discordRpc = $state(false)
    let letJoin = $state(false)
    let disableSystemTray = $state(false)
    let disableLaunchAtStartUp = $state(false)
    let displayAccount = $state(false)
    let serverLocationNotifier = $state(false)
    let optimizer: boolean = $state(false)
    let activityWatching = $state(true)
    let fullscreenOpts: boolean = $state(false)
    let sleepSchedule: boolean = $state(true)
    let isLateNightGamer: boolean = $state(false)

    type shi = {
        version: string
        installPath: string
        exists: boolean
    } | null

    let exe: shi
    let exePath: string

    async function loadConfig() {
        const store = await load('config.json')
        let savedIntegrations = await store.get<Integrations>('integrations')

        // Fix: Fallback for the old typo before reading inner properties like discordRpc
        if (!savedIntegrations) {
            savedIntegrations = await store.get<Integrations>('intergrations')
        }

        if (savedIntegrations) {
            const savedRpc = savedIntegrations.discordRpc as DiscordRpc

            processPriority =
                savedIntegrations.priority ?? 'NORMAL_PRIORITY_CLASS'
            isLateNightGamer = savedIntegrations.sleepSchedule?.visible ?? false
            sleepSchedule = savedIntegrations.sleepSchedule?.enabled ?? true
            disableSystemTray = savedIntegrations.disableSystemTray ?? false
            disableLaunchAtStartUp =
                savedIntegrations.disableLaunchAtStartUp ?? false

            if (savedRpc) {
                discordRpc = savedRpc.enable
                letJoin = savedRpc.letJoin
                displayAccount = savedRpc.displayAccount
            }
            crashHandler = savedIntegrations.closeCrashHandler ?? false
            serverLocationNotifier =
                savedIntegrations.serverLocationNotifier ?? false
            activityWatching = savedIntegrations.activityWatching ?? false
            optimizer = savedIntegrations.optimizer ?? false
        }

        exe = await getCurrentInstallation('player')

        exePath =
            exe?.installPath.replace(/\//g, '\\') + '\\RobloxPlayerBeta.exe'

        fullscreenOpts = await invoke('read_fullscreen_prop', {
            rblxExe: exePath,
        })

        console.log(fullscreenOpts)
    }

    onMount(async () => {
        await loadConfig()

        invoke('set_rpc', {
            details: $_('rpc.general'),
            stateText: $_('rpc.integrations'),
        })
    })

    async function handleChanges() {
        const store = await load('config.json')
        const current = await store.get<Integrations>('integrations')

        const newIntegrations: Integrations = {
            ...current,
            sleepSchedule: {
                enabled: sleepSchedule ?? true,
                visible: current?.sleepSchedule?.visible ?? isLateNightGamer,
            },
            closeCrashHandler: crashHandler ?? false,
            priority: processPriority ?? 'NORMAL_PRIORITY_CLASS',
            discordRpc: { enable: discordRpc, letJoin, displayAccount },
            serverLocationNotifier,
            disableSystemTray: disableSystemTray ?? false,
            disableLaunchAtStartUp: disableLaunchAtStartUp ?? false,
            activityWatching: activityWatching,
            optimizer: optimizer,
        }

        await store.set('integrations', newIntegrations)
        await store.save()

        if (exe?.exists) {
            try {
                setMinimizeToTray(!disableSystemTray)
                setLaunchAtStartup(!disableLaunchAtStartUp)
                await invoke('set_fullscreen_prop', {
                    disable: fullscreenOpts,
                    rblxExe: exePath,
                })
            } catch (e) {
                console.error('Failed to set fullscreen prop', e)
            }
        }
    }
</script>

<div class="flex flex-col gap-8">
    <div class="flex items-center justify-between">
        <div>
            <h1 class="text-3xl font-bold tracking-tight text-stone-100">
                {$_('pages.integrations.integrations')}
            </h1>
            <p class="text-stone-400 mt-1">
                {$_('pages.integrations.description')}
            </p>
        </div>
    </div>

    <div class="flex flex-col gap-3">
        <SettingCard
            title={$_('pages.integrations.activityWatcherCard.title')}
            description={$_(
                'pages.integrations.activityWatcherCard.description'
            )}
            icon={View}
        >
            {#snippet action()}
                <Switch
                    bind:checked={activityWatching}
                    onchange={handleChanges}
                />
            {/snippet}
        </SettingCard>

        {#if isLateNightGamer}
            <SettingCard
                title={$_('pages.integrations.sleepScheduleCard.title')}
                description={$_(
                    'pages.integrations.sleepScheduleCard.description'
                )}
                icon={BedSingle}
            >
                {#snippet action()}
                    <Switch
                        disabled={!activityWatching}
                        bind:checked={sleepSchedule}
                        onchange={handleChanges}
                    />
                {/snippet}
            </SettingCard>
        {/if}

        <SettingCard
            title={$_('pages.integrations.serverNotifierCard.title')}
            description={$_(
                'pages.integrations.serverNotifierCard.description'
            )}
            icon={Bell}
        >
            {#snippet action()}
                <Switch
                    disabled={!activityWatching}
                    bind:checked={serverLocationNotifier}
                    onchange={handleChanges}
                />
            {/snippet}
        </SettingCard>

        <SettingCard
            title={$_('pages.integrations.processPriorityCard.title')}
            description={$_(
                'pages.integrations.processPriorityCard.description'
            )}
            icon={Cpu}
        >
            {#snippet action()}
                <Dropdown
                    options={processPriorityItems}
                    bind:value={processPriority}
                    onchange={handleChanges}
                />
            {/snippet}
        </SettingCard>

        <SettingCard
            title="Disable system tray"
            description="Roblox is watching."
            icon={ArrowDownToDot}
        >
            {#snippet action()}
                <Switch
                    bind:checked={disableSystemTray}
                    onchange={handleChanges}
                />
            {/snippet}
        </SettingCard>

        <SettingCard
            title="Disable launch Roblox at startup"
            description="Don't launch Roblox when you turn on your computer."
            icon={ListStart}
        >
            {#snippet action()}
                <Switch
                    bind:checked={disableLaunchAtStartUp}
                    onchange={handleChanges}
                />
            {/snippet}
        </SettingCard>

        <SettingCard
            title={$_('pages.integrations.optimizerCard.title')}
            description={$_('pages.integrations.optimizerCard.description')}
            icon={Sparkles}
        >
            {#snippet action()}
                <Switch onchange={handleChanges} bind:checked={optimizer} />
            {/snippet}
        </SettingCard>

        <SettingCard
            title={$_(
                'pages.integrations.disableFullscreenoptimizationCard.title'
            )}
            description={$_(
                'pages.integrations.disableFullscreenoptimizationCard.description'
            )}
            icon={Expand}
        >
            {#snippet action()}
                <Switch
                    bind:checked={fullscreenOpts}
                    onchange={handleChanges}
                />
            {/snippet}
        </SettingCard>

        <SettingCard
            title={$_('pages.integrations.closeCrashHandlerCard.title')}
            description={$_(
                'pages.integrations.closeCrashHandlerCard.description'
            )}
            icon={Bomb}
        >
            {#snippet action()}
                <Switch bind:checked={crashHandler} onchange={handleChanges} />
            {/snippet}
        </SettingCard>

        <ExpandableSettingCard
            title={$_('pages.integrations.rpcCard.title')}
            description={$_('pages.integrations.rpcCard.description')}
            icon={Plug}
        >
            {#snippet action()}
                <Switch
                    disabled={!activityWatching}
                    bind:checked={discordRpc}
                    onchange={handleChanges}
                />
            {/snippet}

            <div class="flex gap-3">
                <p>{$_('pages.integrations.rpcCard.option1')}</p>
                <Switch
                    bind:checked={letJoin}
                    onchange={handleChanges}
                    disabled={!activityWatching}
                />
            </div>

            <div class="flex gap-3">
                <p>{$_('pages.integrations.rpcCard.option2')}</p>
                <Switch
                    bind:checked={displayAccount}
                    onchange={handleChanges}
                    disabled={!activityWatching}
                />
            </div>
        </ExpandableSettingCard>

        <SettingCard
            title={$_('pages.integrations.windowManipulationCard.title')}
            description={$_(
                'pages.integrations.windowManipulationCard.description'
            )}
            icon={CodeXml}
        >
            {#snippet action()}
                <Button
                    variant="secondary"
                    disabled={!activityWatching}
                    onclick={() => goto('integrations/interactiveSettings')}
                >
                    {$_('pages.integrations.windowManipulationCard.button')}
                </Button>
            {/snippet}
        </SettingCard>

        <SettingCard
            title={$_('pages.integrations.gameHistoryCard.title')}
            description={$_('pages.integrations.gameHistoryCard.description')}
            icon={History}
        >
            {#snippet action()}
                <Button
                    disabled={!activityWatching}
                    variant="secondary"
                    onclick={() => goto('integrations/gameHistory')}
                >
                    {$_('pages.integrations.gameHistoryCard.button')}
                </Button>
            {/snippet}
        </SettingCard>

        <SettingCard
            title="Server Management"
            description="Select region, Submission Servers"
            icon={Server}
        >
            {#snippet action()}
                <Button
                    variant="secondary"
                    onclick={() => {
                        goto('./integrations/serverManagement')
                    }}>open</Button
                >
            {/snippet}
        </SettingCard>
    </div>
</div>
