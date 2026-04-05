<script lang="ts">
    import Button from '$lib/components/atoms/Button.svelte'
    import ExpandableSettingCard from '$lib/components/molecules/ExpandableSettingCard.svelte'
    import SettingCard from '$lib/components/molecules/SettingCard.svelte'
    import Dropdown from '$lib/components/molecules/Dropdown.svelte'
    import type { BuildInfo } from '$lib/types'
    import { Heart } from '@lucide/svelte'
    import { Info } from '@lucide/svelte'
    import { invoke } from '@tauri-apps/api/core'
    import { openUrl } from '@tauri-apps/plugin-opener'
    import { onMount } from 'svelte'
    import {Languages} from '@lucide/svelte'
    import { locale, locales } from 'svelte-i18n';
    import { get } from 'svelte/store'
    import { _ } from 'svelte-i18n'

    const Arona = '/Arona.png'
    let info: BuildInfo
    let hash: string
    let buildtime: string

    const localeList = get(locales);

    const dropdownOptions = localeList.map((loc) => ({
        label: loc,
        value: loc
    }));

    let currentLocale

    onMount(async () => {
        info = await invoke('crush')
        currentLocale = get(locale)

        hash = info.hash
        buildtime = info.build_date
    })

    async function handleDonate() {
        openUrl('https://mally.qzz.io/donate')
    }
</script>

<div class="flex flex-col gap-4">
    <div class="flex items-center justify-between">
        <div>
            <h1 class="text-3xl font-bold tracking-tight text-stone-100">
                {$_('pages.settings.settings')}
            </h1>
        </div>
    </div>

    <SettingCard
        title={$_('pages.settings.languageCard.title')}
        description={$_('pages.settings.languageCard.description')}
        icon={Languages}
    >
        <Dropdown slot="action"
            bind:value={currentLocale}
            options={dropdownOptions}
            on:select={(e) => locale.set(e.detail)}
        />
    </SettingCard>

    <ExpandableSettingCard
        title={$_('pages.settings.aboutCard.title')}
        description={$_('pages.settings.aboutCard.description')}
        icon={Info}
    >
        <div>
            <p class="sm">{$_('pages.settings.aboutCard.builtOn', { values: { date: buildtime } })}</p>
            <p class="sm">{$_('pages.settings.aboutCard.commitHash', { values: { hash } })}</p>
        </div>
    </ExpandableSettingCard>

    <ExpandableSettingCard
        title={$_('pages.settings.donateCard.title')}
        description={$_('pages.settings.donateCard.description')}
        icon={Arona}
    >
        <Button variant="secondary" on:click={handleDonate}>{$_('pages.settings.donateCard.button')}</Button>
    </ExpandableSettingCard>
</div>
