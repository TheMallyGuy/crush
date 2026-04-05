<script lang="ts">
    import Button from '$lib/components/atoms/Button.svelte'
    import ExpandableSettingCard from '$lib/components/molecules/ExpandableSettingCard.svelte'
    import SettingCard from '$lib/components/molecules/SettingCard.svelte'
    import Dropdown from '$lib/components/molecules/Dropdown.svelte'
    import type { BuildInfo } from '$lib/types'
    import { Heart, Info, Languages } from '@lucide/svelte'
    import { invoke } from '@tauri-apps/api/core'
    import { openUrl } from '@tauri-apps/plugin-opener'
    import { onMount } from 'svelte'
    import { locale, locales, _, waitLocale } from 'svelte-i18n'
    import { derived } from 'svelte/store'
    import { load } from '@tauri-apps/plugin-store'

    const Arona = '/Arona.png'
    let info: BuildInfo
    let hash: string
    let buildtime: string

    const LOCALE_NAMES: Record<string, string> = {
        af: 'Afrikaans',
        ar: 'العربية',
        ca: 'Català',
        cs: 'Čeština',
        da: 'Dansk',
        de: 'Deutsch',
        el: 'Ελληνικά',
        en: 'English',
        'es-ES': 'Español',
        fi: 'Suomi',
        fr: 'Français',
        he: 'עברית',
        hu: 'Magyar',
        it: 'Italiano',
        ja: '日本語',
        ko: '한국어',
        nl: 'Nederlands',
        no: 'Norsk',
        pl: 'Polski',
        'pt-BR': 'Português (Brasil)',
        vi: 'Tiếng Việt',
    }

    const dropdownOptions = derived(locales, ($locales) =>
        $locales.map((loc) => ({
            label: LOCALE_NAMES[loc] ?? loc,
            value: loc,
        }))
    )

    let currentLocale: string

    onMount(async () => {
        info = await invoke('crush')
        currentLocale = $locale ?? 'en'
        hash = info.hash
        buildtime = info.build_date
    })

    async function handleLanguage() {
        let config = await load('config.json')
        locale.set(currentLocale)
        config.set('language', currentLocale)
        config.save()
        await waitLocale()
    }

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
        <Dropdown
            slot="action"
            bind:value={currentLocale}
            options={$dropdownOptions}
            on:change={handleLanguage}
        />
    </SettingCard>

    <ExpandableSettingCard
        title={$_('pages.settings.aboutCard.title')}
        description={$_('pages.settings.aboutCard.description')}
        icon={Info}
    >
        <div>
            <p class="sm">
                {$_('pages.settings.aboutCard.builtOn', {
                    values: { date: buildtime },
                })}
            </p>
            <p class="sm">
                {$_('pages.settings.aboutCard.commitHash', {
                    values: { hash },
                })}
            </p>
        </div>
    </ExpandableSettingCard>

    <ExpandableSettingCard
        title={$_('pages.settings.donateCard.title')}
        description={$_('pages.settings.donateCard.description')}
        icon={Arona}
    >
        <Button variant="secondary" on:click={handleDonate}
            >{$_('pages.settings.donateCard.button')}</Button
        >
    </ExpandableSettingCard>
</div>
