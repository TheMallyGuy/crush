<script lang="ts">
    import { _ } from 'svelte-i18n'
    import Button from '$lib/components/atoms/Button.svelte'
    import { goto } from '$app/navigation'
    import SettingCard from '$lib/components/molecules/SettingCard.svelte'
    import Switch from '$lib/components/atoms/Switch.svelte'
    import { load } from '@tauri-apps/plugin-store'
    import { onMount } from 'svelte'
    import type { Integrations, DiscordRpc } from '$lib/types'
    import Textbox from '$lib/components/atoms/Textbox.svelte'
    import { resolve } from '$app/paths'

    let interactAPIValue: boolean

    let mimimizeValue: boolean
    let maximizeValue: boolean
    let focusValue: boolean
    let moveWindowValue: boolean
    let restoreValue: boolean
    let windowTitleValue: boolean
    let borderlessValue: boolean

    let transparencyValue: boolean
    let minimumTransValue: number
    let maximizeTransValue: number

    async function loadConfig() {
        const store = await load('config.json')
        let savedIntegrations =
            (await store.get<Integrations>('integrations')) ??
            (await store.get<Integrations>('intergrations'))

        const interactive = savedIntegrations?.interactive
        const scopes = interactive?.scopes
        const transparencyScope = scopes?.transparencyScopes

        interactAPIValue = interactive?.enable ?? false
        mimimizeValue = scopes?.minimize ?? true
        maximizeValue = scopes?.maximize ?? true
        focusValue = scopes?.focus ?? true
        moveWindowValue = scopes?.moveWindow ?? true
        restoreValue = scopes?.restore ?? true
        windowTitleValue = scopes?.setTitle ?? true
        borderlessValue = scopes?.setBorderless ?? true
        transparencyValue = transparencyScope?.enabled ?? true
        minimumTransValue = transparencyScope?.minTransparency ?? 0
        maximizeTransValue = transparencyScope?.maxTransparency ?? 250
    }

    async function handleChanges() {
        const store = await load('config.json')
        const savedIntegrations =
            (await store.get<Integrations>('integrations')) ??
            (await store.get<Integrations>('intergrations'))

        const savedInteractive = savedIntegrations?.interactive
        const savedScopes = savedInteractive?.scopes
        const savedTransparency = savedScopes?.transparencyScopes

        const newIntegrations: Integrations = {
            discordRpc: savedIntegrations?.discordRpc ?? {
                enable: false,
                displayAccount: false,
                letJoin: false,
            },
            serverLocationNotifier:
                savedIntegrations?.serverLocationNotifier ?? false,
            roValra: savedIntegrations?.roValra ?? {
                joinServerForYouValue: false,
            },
            gameCache: savedIntegrations?.gameCache,
            crushRpc: savedIntegrations?.crushRpc,
            interactive: {
                enable: interactAPIValue,
                scopes: {
                    minimize: mimimizeValue,
                    maximize: maximizeValue,
                    focus: focusValue,
                    moveWindow: moveWindowValue,
                    restore: restoreValue,
                    setTitle: windowTitleValue,
                    setBorderless: borderlessValue,
                    transparencyScopes: {
                        ...savedTransparency,
                        enabled: transparencyValue,
                        minTransparency: minimumTransValue,
                        maxTransparency: maximizeTransValue,
                    },
                },
            },
        }

        await store.set('integrations', newIntegrations)
        await store.save()
    }

    onMount(async () => {
        loadConfig()
    })
</script>

<div class="flex flex-col gap-8">
    <div class="flex items-center justify-between">
        <div>
            <h1 class="text-3xl font-bold tracking-tight text-stone-100">
                Interactive API settings
            </h1>
            <p class="text-stone-400 mt-1">
                Configure how Interactive API should act.
            </p>
        </div>
        <div class="flex items-center gap-2">
            <Button variant="secondary" onclick={() => goto('../integrations')}>
                {$_('pages.integrations.gameHistory.backToIntegrations')}
            </Button>
        </div>
    </div>

    <SettingCard
        title="Enable"
        description="Choose whether or not to enable Interactive API."
    >
        <Switch slot="action" bind:checked={interactAPIValue} on:change={handleChanges} />
    </SettingCard>

    <div class="flex flex-col gap-3">
        <div>
            <h1 class="text-3xl font-bold tracking-tight text-stone-100">
                Advanced
            </h1>
            <p class="text-stone-400 mt-1">Deeply configure Interactive API.</p>
        </div>

        <div class="flex flex-col gap-2">
            <h1 class="text-2xl font-bold tracking-tight text-stone-100">
                Scopes
            </h1>

            <SettingCard
                title="Minimize"
                description="Whenther or not enable mimimize control via Interactive API."
            >
                <Switch slot="action" bind:checked={mimimizeValue} disabled={!interactAPIValue} on:change={handleChanges}/>
            </SettingCard>

            <SettingCard
                title="Maximize"
                description="Whenther or not enable maximize control via Interactive API."
            >
                <Switch slot="action" bind:checked={maximizeValue} disabled={!interactAPIValue} on:change={handleChanges}/>
            </SettingCard>

            <SettingCard
                title="Focus"
                description="Whenther or not enable focus control via Interactive API."
            >
                <Switch slot="action" bind:checked={focusValue} disabled={!interactAPIValue} on:change={handleChanges}/>
            </SettingCard>

            <SettingCard
                title="Move Window"
                description="Whenther or not enable move window control via Interactive API."
            >
                <Switch slot="action" bind:checked={moveWindowValue} disabled={!interactAPIValue} on:change={handleChanges}/>
            </SettingCard>

            <SettingCard
                title="Restore"
                description="Whenther or not enable restore control via Interactive API."
            >
                <Switch slot="action" bind:checked={restoreValue} disabled={!interactAPIValue} on:change={handleChanges}/>
            </SettingCard>

            <SettingCard
                title="Set window title"
                description="Whenther or not enable set window title control via Interactive API."
            >
                <Switch slot="action" bind:checked={windowTitleValue} disabled={!interactAPIValue} on:change={handleChanges}/>
            </SettingCard>

            <SettingCard
                title="Set borderless"
                description="Whenther or not enable borderless of the window control via Interactive API."
            >
                <Switch slot="action" bind:checked={borderlessValue} disabled={!interactAPIValue} on:change={handleChanges}/>
            </SettingCard>
        </div>

        <div class="flex flex-col gap-2">
            <h1 class="text-2xl font-bold tracking-tight text-stone-100">
                Transparency Settings
            </h1>

            <SettingCard
                title="Enable"
                description="Whenther or not transparency control via Interactive API."
            >
                <Switch slot="action" bind:checked={transparencyValue} disabled={!interactAPIValue} on:change={handleChanges}/>
            </SettingCard>

            <SettingCard
                title="Min transparency"
                description="The minimum of how transparency of your window would get."
            >
                <Textbox slot="action" bind:value={minimumTransValue} disabled={!interactAPIValue} on:change={handleChanges}/>
            </SettingCard>

            <SettingCard
                title="Max transparency"
                description="The maximum of how transparency of your window would get."
            >
                <Textbox slot="action" bind:value={maximizeTransValue} disabled={!interactAPIValue} on:change={handleChanges}/>
            </SettingCard>
        </div>
    </div>
</div>
