<script lang="ts">
    import SettingCard from '$lib/components/molecules/SettingCard.svelte'
    import Button from '$lib/components/atoms/Button.svelte'
    import Switch from '$lib/components/atoms/Switch.svelte'
    import { Bell, Plug, History, CodeXml, View, Cpu } from '@lucide/svelte'
    import { invoke } from '@tauri-apps/api/core'
    import { onMount } from 'svelte'
    import { load } from '@tauri-apps/plugin-store'
    import type { Integrations, DiscordRpc, PriorityClass } from '$lib/types'
    import { _ } from 'svelte-i18n'
    import { goto } from '$app/navigation'
    import ExpandableSettingCard from '$lib/components/molecules/ExpandableSettingCard.svelte'
    import Dropdown from '$lib/components/molecules/Dropdown.svelte'

    let processPriorityItems = [
        {
            value: 'BELOW_NORMAL_PRIORITY_CLASS',
            label: 'BELOW_NORMAL',
        },
        {
            value: 'NORMAL_PRIORITY_CLASS',
            label: 'NORMAL',
        },
        {
            value: 'ABOVE_NORMAL_PRIORITY_CLASS',
            label: 'ABOVE_NORMAL',
        },
        {
            value: 'HIGH_PRIORITY_CLASS',
            label: 'HIGH',
        },
        {
            value: 'REALTIME_PRIORITY_CLASS',
            label: 'REALTIME',
        },
    ]
    let processPriority: PriorityClass = 'NORMAL_PRIORITY_CLASS'

    let discordRpc = false
    let letJoin = false
    let displayAccount = false
    let serverLocationNotifier = false
    let activityWatching = true
    const roValaraLogoColored = '/RovalraColored.svg'
    const roValaraLogo = '/Rovalra.svg'

    async function loadConfig() {
        const store = await load('config.json')
        let savedIntegrations = await store.get<Integrations>('integrations')
        let savedRpc = <DiscordRpc>savedIntegrations?.discordRpc

        if (!savedIntegrations) {
            savedIntegrations = await store.get<Integrations>('intergrations')
        }

        if (savedIntegrations) {
            processPriority = savedIntegrations.priority
            if (savedRpc) {
                discordRpc = savedRpc.enable
                letJoin = savedRpc.letJoin
                displayAccount = savedRpc.displayAccount
            }
            serverLocationNotifier = savedIntegrations.serverLocationNotifier
            activityWatching = savedIntegrations.activityWatching
        }
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
            priority: processPriority ?? 'NORMAL_PRIORITY_CLASS',
            discordRpc: { enable: discordRpc, letJoin, displayAccount },
            serverLocationNotifier,
            roValra: current?.roValra ?? { joinServerForYouValue: false },
            activityWatching: activityWatching,
        }

        await store.set('integrations', newIntegrations)

        await store.save()
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
            <Switch
                slot="action"
                bind:checked={activityWatching}
                on:change={handleChanges}
            />
        </SettingCard>

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
            title="Process Priority"
            description="EXTREMELY EXPERIMENTAL. Change roblox process priority. May boost performance."
            icon={Cpu}
        >
            <Dropdown
                slot="action"
                options={processPriorityItems}
                bind:value={processPriority}
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
                <!-- option 1 -->
                <p>
                    {$_('pages.integrations.rpcCard.option1')}
                </p>
                <Switch
                    bind:checked={letJoin}
                    on:change={handleChanges}
                    disabled={!activityWatching}
                />
            </div>

            <div class="flex gap-3">
                <!-- option 2 -->
                <p>
                    {$_('pages.integrations.rpcCard.option2')}
                </p>
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
                on:click={() => {
                    goto('integrations/interactiveSettings')
                }}
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
                on:click={() => {
                    goto('integrations/gameHistory')
                }}
            >
                {$_('pages.integrations.gameHistoryCard.button')}
            </Button>
        </SettingCard>

        <SettingCard
            title={$_('pages.integrations.roValraCard.title')}
            description={$_('pages.integrations.roValraCard.description')}
            icon={roValaraLogo}
            iconHover={roValaraLogoColored}
        >
            <Button
                slot="action"
                variant="secondary"
                on:click={() => {
                    goto('integrations/roValra')
                }}
            >
                {$_('pages.integrations.roValraCard.button')}
            </Button>
        </SettingCard>
    </div>
</div>
