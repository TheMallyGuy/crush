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
    import Dialog from '$lib/components/molecules/Dialog.svelte'

    let processPriorityItems = [
        { value: 'BELOW_NORMAL_PRIORITY_CLASS', label: 'BELOW_NORMAL' },
        { value: 'NORMAL_PRIORITY_CLASS', label: 'NORMAL' },
        { value: 'ABOVE_NORMAL_PRIORITY_CLASS', label: 'ABOVE_NORMAL' },
        { value: 'HIGH_PRIORITY_CLASS', label: 'HIGH' },
        { value: 'REALTIME_PRIORITY_CLASS', label: 'REALTIME' },
    ]
    let processPriority: PriorityClass = 'NORMAL_PRIORITY_CLASS'

    let warningDialog = false

    let crashHandler = false
    let discordRpc = false
    let letJoin = false
    let displayAccount = false
    let serverLocationNotifier = false
    let optimizer: boolean
    let activityWatching = true
    let fullscreenOpts: boolean = false
    let sleepSchedule: boolean = true
    let isLateNightGamer: boolean = false

    type shi = {
        version: string
        installPath: string
        exists: boolean
    } | null

    let exe: shi
    let exePath: string

    const roValaraLogo = '/Rovalra.png'
    const swifty = '/swifty.png'

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
            roValra: current?.roValra ?? { joinServerForYouValue: false },
            activityWatching: activityWatching,
            optimizer: optimizer,
        }

        await invoke('set_fullscreen_prop', {
            disable: fullscreenOpts,
            rblxExe: exePath,
        })

        await store.set('integrations', newIntegrations)
        await store.save()
    }
</script>

<Dialog
    bind:open={warningDialog}
    on:close={() => {
        warningDialog = false
    }}
    title={$_('pages.integrations.dialogs.title')}
    description={$_('pages.integrations.dialogs.description')}
>
    <div slot="actions">
        <Button
            variant="secondary"
            size="sm"
            on:click={() => {
                warningDialog = false
                goto('integrations/swift')
            }}
        >
            {$_('pages.integrations.dialogs.confirm')}
        </Button>
    </div>
</Dialog>

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
            <Switch
                slot="action"
                bind:checked={activityWatching}
                on:change={handleChanges}
            />
        </SettingCard>

        {#if isLateNightGamer}
            <SettingCard
                title={$_('pages.integrations.sleepScheduleCard.title')}
                description={$_(
                    'pages.integrations.sleepScheduleCard.description'
                )}
                icon={BedSingle}
            >
                <Switch
                    slot="action"
                    disabled={!activityWatching}
                    bind:checked={sleepSchedule}
                    on:change={handleChanges}
                />
            </SettingCard>
        {/if}

        <SettingCard
            title={$_('pages.integrations.serverNotifierCard.title')}
            description={$_(
                'pages.integrations.serverNotifierCard.description'
            )}
            icon={Bell}
        >
            <Switch
                slot="action"
                disabled={!activityWatching}
                bind:checked={serverLocationNotifier}
                on:change={handleChanges}
            />
        </SettingCard>

        <SettingCard
            title={$_('pages.integrations.processPriorityCard.title')}
            description={$_(
                'pages.integrations.processPriorityCard.description'
            )}
            icon={Cpu}
        >
            <Dropdown
                slot="action"
                options={processPriorityItems}
                bind:value={processPriority}
                on:change={handleChanges}
            />
        </SettingCard>

        <SettingCard
            title={$_('pages.integrations.optimizerCard.title')}
            description={$_('pages.integrations.optimizerCard.description')}
            icon={Sparkles}
        >
            <Switch
                slot="action"
                on:change={handleChanges}
                bind:checked={optimizer}
            />
        </SettingCard>

        <SettingCard
            title="SwiftTunnel"
            description="Crush have swifttunell.ddl meaning crush have ablities to replicate swifttunnel!"
            icon={swifty}
            doTheGrayThing={true}
        >
            <Button
                slot="action"
                on:click={() => (warningDialog = true)}
                variant="secondary"
            >
                Open</Button
            >
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
            <Switch
                slot="action"
                bind:checked={fullscreenOpts}
                on:change={handleChanges}
            />
        </SettingCard>

        <SettingCard
            title={$_('pages.integrations.closeCrashHandlerCard.title')}
            description={$_(
                'pages.integrations.closeCrashHandlerCard.description'
            )}
            icon={Bomb}
        >
            <Switch
                slot="action"
                bind:checked={crashHandler}
                on:change={handleChanges}
            />
        </SettingCard>

        <ExpandableSettingCard
            title={$_('pages.integrations.rpcCard.title')}
            description={$_('pages.integrations.rpcCard.description')}
            icon={Plug}
        >
            <Switch
                slot="action"
                disabled={!activityWatching}
                bind:checked={discordRpc}
                on:change={handleChanges}
            />

            <div class="flex gap-3">
                <p>{$_('pages.integrations.rpcCard.option1')}</p>
                <Switch
                    bind:checked={letJoin}
                    on:change={handleChanges}
                    disabled={!activityWatching}
                />
            </div>

            <div class="flex gap-3">
                <p>{$_('pages.integrations.rpcCard.option2')}</p>
                <Switch
                    bind:checked={displayAccount}
                    on:change={handleChanges}
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
            <Button
                slot="action"
                variant="secondary"
                disabled={!activityWatching}
                on:click={() => goto('integrations/interactiveSettings')}
            >
                {$_('pages.integrations.windowManipulationCard.button')}
            </Button>
        </SettingCard>

        <SettingCard
            title={$_('pages.integrations.gameHistoryCard.title')}
            description={$_('pages.integrations.gameHistoryCard.description')}
            icon={History}
        >
            <Button
                slot="action"
                disabled={!activityWatching}
                variant="secondary"
                on:click={() => goto('integrations/gameHistory')}
            >
                {$_('pages.integrations.gameHistoryCard.button')}
            </Button>
        </SettingCard>

        <SettingCard
            title={$_('pages.integrations.roValraCard.title')}
            description={$_('pages.integrations.roValraCard.description')}
            icon={roValaraLogo}
            doTheGrayThing={true}
        >
            <Button
                slot="action"
                variant="secondary"
                on:click={() => goto('integrations/roValra')}
            >
                {$_('pages.integrations.roValraCard.button')}
            </Button>
        </SettingCard>
    </div>
</div>
