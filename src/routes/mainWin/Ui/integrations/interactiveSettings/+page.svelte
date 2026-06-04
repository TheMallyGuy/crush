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

    async function loadConfig() {
        const store = await load('config.json')
        let savedIntegrations =
            (await store.get<Integrations>('integrations')) ??
            (await store.get<Integrations>('intergrations'))

        const interactive = savedIntegrations?.interactive
        const scopes = interactive?.scopes

        interactAPIValue = interactive?.enable ?? false
        mimimizeValue = scopes?.minimize ?? true
        maximizeValue = scopes?.maximize ?? true
        focusValue = scopes?.focus ?? true
        moveWindowValue = scopes?.moveWindow ?? true
        restoreValue = scopes?.restore ?? true
        windowTitleValue = scopes?.setTitle ?? true
        borderlessValue = scopes?.setBorderless ?? true
    }

    async function handleChanges() {
        const store = await load('config.json')
        const savedIntegrations =
            (await store.get<Integrations>('integrations')) ??
            (await store.get<Integrations>('intergrations'))

        const savedInteractive = savedIntegrations?.interactive
        const savedScopes = savedInteractive?.scopes

        const newIntegrations: Integrations = {
            ...savedIntegrations,
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
                    transparencyScopes: savedIntegrations?.interactive?.scopes?.transparencyScopes ?? {
                        enabled: false,
                        maxTransparency: 0,
                        minTransparency: 0,
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
                {$_("pages.integrations.windowManipulation.title")}
            </h1>
            <p class="text-stone-400 mt-1">
                {$_("pages.integrations.windowManipulation.description")}
            </p>
        </div>
        <div class="flex items-center gap-2">
            <Button variant="secondary" onclick={() => goto('../integrations')}>
                {$_('pages.integrations.gameHistory.backToIntegrations')}
            </Button>
        </div>
    </div>

    <SettingCard
        title={$_("pages.integrations.windowManipulation.enableCard.title")}
        description={$_("pages.integrations.windowManipulation.enableCard.description")}
    >
        <Switch slot="action" bind:checked={interactAPIValue} on:change={handleChanges} />
    </SettingCard>

    <div class="flex flex-col gap-3">
        <div>
            <h1 class="text-3xl font-bold tracking-tight text-stone-100">
                {$_("pages.integrations.windowManipulation.advanced.title")}
            </h1>
            <p class="text-stone-400 mt-1">{$_("pages.integrations.windowManipulation.advanced.description")}</p>
        </div>

        <div class="flex flex-col gap-2">
            <h1 class="text-2xl font-bold tracking-tight text-stone-100">
                {$_("pages.integrations.windowManipulation.advanced.scopes.title")}
            </h1>

            <SettingCard
                title={$_("pages.integrations.windowManipulation.advanced.scopes.minimizeCard.title")}
                description={$_("pages.integrations.windowManipulation.advanced.scopes.minimizeCard.description")}
            >
                <Switch slot="action" bind:checked={mimimizeValue} disabled={!interactAPIValue} on:change={handleChanges}/>
            </SettingCard>

            <SettingCard
                title={$_("pages.integrations.windowManipulation.advanced.scopes.maximizeCard.title")}
                description={$_("pages.integrations.windowManipulation.advanced.scopes.maximizeCard.description")}
            >
                <Switch slot="action" bind:checked={maximizeValue} disabled={!interactAPIValue} on:change={handleChanges}/>
            </SettingCard>

            <SettingCard
                title={$_("pages.integrations.windowManipulation.advanced.scopes.focusCard.title")}
                description={$_("pages.integrations.windowManipulation.advanced.scopes.focusCard.description")}
            >
                <Switch slot="action" bind:checked={focusValue} disabled={!interactAPIValue} on:change={handleChanges}/>
            </SettingCard>

            <SettingCard
                title={$_("pages.integrations.windowManipulation.advanced.scopes.moveWindowCard.title")}
                description={$_("pages.integrations.windowManipulation.advanced.scopes.moveWindowCard.description")}
            >
                <Switch slot="action" bind:checked={moveWindowValue} disabled={!interactAPIValue} on:change={handleChanges}/>
            </SettingCard>


            <SettingCard
                title={$_("pages.integrations.windowManipulation.advanced.scopes.setWindowTitleCard.title")}
                description={$_("pages.integrations.windowManipulation.advanced.scopes.setWindowTitleCard.description")}
            >
                <Switch slot="action" bind:checked={windowTitleValue} disabled={!interactAPIValue} on:change={handleChanges}/>
            </SettingCard>

            <SettingCard
                title={$_("pages.integrations.windowManipulation.advanced.scopes.setBorderless.title")}
                description={$_("pages.integrations.windowManipulation.advanced.scopes.setBorderless.description")}
            >
                <Switch slot="action" bind:checked={borderlessValue} disabled={!interactAPIValue} on:change={handleChanges}/>
            </SettingCard>
        </div>

    </div>
</div>
