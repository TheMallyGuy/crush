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
        ArrowDownFromLine,
        ArrowDownToDot,
        ListStart,
        Cuboid,
        Server,
    } from '@lucide/svelte'
    import { marked } from 'marked'
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
    let processPriority: PriorityClass = 'NORMAL_PRIORITY_CLASS'

    let tosDialog: boolean = false
    let tos = `
# Terms of Service

Last updated: 2026-07-07

This document governs use of this API ("the Service"), which accepts reports of running Roblox game server instances (job ID, claimed IP, region) for tracking and approval. By sending requests to the Service, you agree to the following. The Service's source code is not public; these terms cover API usage only.

## 1. What we collect

For each submission we store: the reporting machine's IP (\`serverIp\`), the Roblox server's \`jobId\`, its self-reported \`claimedIp\`, the region string it provides, and the IP address the request actually came from (used only for rate limiting and abuse detection).

## 2. Acceptable use

- Only submit data for server instances you legitimately operate.
- Don't spoof \`serverIp\`, \`claimedIp\`, or \`jobId\` values that don't correspond to a real server instance you control.
- Don't attempt to exceed, bypass, or automate around the published rate limits (currently 60 requests/minute per IP on submissions).
- Don't use the Service to probe, scan, or attack Roblox infrastructure or third parties.
- Don't attempt to access admin-gated endpoints without an authorized key.

Violating any of the above may result in your IP or game being blocked from the Service without notice.

## 3. No guarantees

The Service is provided as-is, run on a best-effort basis by a single maintainer. There is no uptime guarantee, no SLA, and no warranty of correctness. Data may be pruned, migrated, or reset at any time without notice. Access to the Service may be revoked at any time, for any reason.

## 4. Approval is discretionary

Game registration and submission approval are admin-gated actions. Submitting data does not entitle you to approval - the maintainer may accept or reject any game or submission for any reason.

## 5. No license to the Service itself

These terms grant you permission to use the Service's public endpoints; they do not grant any rights to the Service's source code, infrastructure, or internal implementation, which remain private and proprietary.

## 6. Changes

These terms, the API shape, and rate limits may change at any time. Continued use of the Service after a change constitutes acceptance of the updated terms.

## 7. Contact

Questions or abuse reports: contact Mally (m.aay) on Discord.

`

    let crashHandler = false
    let discordRpc = false
    let letJoin = false
    let disableSystemTray = false
    let disableLaunchAtStartUp = false
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

<Dialog
    open={tosDialog}
    onclose={() => {
        tosDialog = !tosDialog
    }}
    title="TERMS OF SERVICE"
>
    <div
        class="tos-content max-h-64 overflow-y-auto border border-stone-800/40 bg-black/20 p-3 text-sm text-stone-300"
    >
        {@html marked(tos)}
    </div>
    {#snippet actions()}
        <Button
            onclick={() => {
                tosDialog = !tosDialog
            }}>I agree</Button
        >
    {/snippet}
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
                        tosDialog = !tosDialog
                    }}>open</Button
                >
            {/snippet}
        </SettingCard>
    </div>
</div>

<style>
    .tos-content :global(h1) {
        font-size: 1.25rem;
        font-weight: 700;
        margin-bottom: 0.5rem;
    }
    .tos-content :global(h2) {
        font-size: 1.05rem;
        font-weight: 600;
        margin-top: 1rem;
        margin-bottom: 0.4rem;
    }
    .tos-content :global(p) {
        margin-bottom: 0.75rem;
    }
    .tos-content :global(ul) {
        list-style: disc;
        padding-left: 1.25rem;
        margin-bottom: 0.75rem;
    }
    .tos-content :global(li) {
        margin-bottom: 0.25rem;
    }
    .tos-content :global(a) {
        color: #f97316;
        text-decoration: underline;
    }
    .tos-content :global(code) {
        background: rgba(255, 255, 255, 0.08);
        padding: 0.1rem 0.3rem;
        border-radius: 0.25rem;
        font-size: 0.85em;
    }
    .tos-content :global(strong) {
        font-weight: 700;
        color: #e7e5e4;
    }
</style>
