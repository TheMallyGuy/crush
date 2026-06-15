<script lang="ts">
    import Button from '$lib/components/atoms/Button.svelte'
    import ExpandableSettingCard from '$lib/components/molecules/ExpandableSettingCard.svelte'
    import SettingCard from '$lib/components/molecules/SettingCard.svelte'
    import Dropdown from '$lib/components/molecules/Dropdown.svelte'
    import type { BuildInfo } from '$lib/types'
    import { relaunch } from '@tauri-apps/plugin-process'
    import {
        Heart,
        Info,
        Languages,
        BookHeart,
        AudioWaveform,
        Crosshair,
        Cuboid,
        ScrollText,
        Shield,
        Cloud,
    } from '@lucide/svelte'
    import { invoke } from '@tauri-apps/api/core'
    import { openUrl } from '@tauri-apps/plugin-opener'
    import { onMount } from 'svelte'
    import { settings } from '$lib/stores/settings.svelte'
    import { locale, locales, _ } from 'svelte-i18n'
    import { derived } from 'svelte/store'
    import { load } from '@tauri-apps/plugin-store'
    import Switch from '$lib/components/atoms/Switch.svelte'
    import { Backlight } from '$lib/components/magic/backlight'
    import { SmoothCursor } from '$lib/components/magic/smooth-cursor'
    import Dialog from '$lib/components/molecules/Dialog.svelte'
    import Checkbox from '$lib/components/atoms/Checkbox.svelte'
    import { notify } from '$lib/notify'
    import { goto } from '$app/navigation'

    const Arona = '/Arona.png'

    let wasLockedIn = settings.lockedInMode
    let wasRedRings = settings.redRings

    let privacyAndDataDialog = $state(false)
    let creditDialog = $state(false)

    let info: BuildInfo | undefined = $state()
    let hash: string = $state('Unknown hash')
    let buildtime: string = $state('Unknown Build time')
    let version: string = $state('Unknown Version')

    let devMode = $state(false)

    const DEV = 'dev'

    let buffer = ''

    function handleGlobalKey(event: KeyboardEvent): void {
        const target = event.target as HTMLElement
        if (
            target.tagName === 'INPUT' ||
            target.tagName === 'TEXTAREA' ||
            target.isContentEditable
        ) {
            return
        }

        buffer += event.key.toLowerCase()

        buffer = buffer.slice(-DEV.length)

        if (buffer === DEV) {
            console.log('devloper mode enabled')
            devMode = true
            notify.send({
                title: 'Developer Mode enabled',
                variant: 'success',
            })
        }
    }

    const LOCALE_NAMES: Record<string, string> = {
        'af-ZA': 'Afrikaans',
        'ar-SA': 'العربية',
        'ca-ES': 'Català',
        'cs-CZ': 'Čeština',
        'da-DK': 'Dansk',
        'de-DE': 'Deutsch',
        'el-GR': 'Ελληνικά',
        'en-US': 'English',
        'es-ES': 'Español',
        'fi-FI': 'Suomi',
        'fr-FR': 'Français',
        'he-IL': 'עברית',
        'hu-HU': 'Magyar',
        'it-IT': 'Italiano',
        'ja-JP': '日本語',
        'ko-KR': '한국어',
        'nl-NL': 'Nederlands',
        'no-NO': 'Norsk',
        'pl-PL': 'Polski',
        'pt-BR': 'Português (Brasil)',
        'pt-PT': 'Português (Portugal)',
        'ro-RO': 'Română',
        'ru-RU': 'Русский',
        'sr-SP': 'Српски',
        'sv-SE': 'Svenska',
        'tr-TR': 'Türkçe',
        'uk-UA': 'Українська',
        'vi-VN': 'Tiếng Việt',
        'vls-BE': 'Vlaams',
        'zh-CN': '中文 (简体)',
        'zh-TW': '中文 (繁體)',
        'ni-ha': 'NIHAHAHA!',
    }

    const dropdownOptions = derived(locales, ($locales) =>
        $locales.map((loc) => ({
            label: LOCALE_NAMES[loc] ?? loc,
            value: loc,
        }))
    )

    async function handleResetCrushOnboarding() {
        const store = await load('config.json')
        await store.set('firstLaunch', true)
        await relaunch()
    }

    async function handleDonate() {
        openUrl('https://mally.qzz.io/donate')
    }

    $effect(() => {
        if (!settings.loaded) return

        const lang = settings.currentLocale
        const locked = settings.lockedInMode
        const redRinged = settings.redRings

        void settings.discordRpcEnabled
        void settings.robloxWarpped

        locale.set(lang)
        ;(async () => {
            await settings.save()

            if (wasLockedIn && !locked) {
                window.location.reload()
            }

            if (wasRedRings && !redRinged) {
                window.location.reload()
            }

            wasLockedIn = locked
            wasRedRings = redRinged
        })()
    })

    onMount(async () => {
        if (!settings.loaded) {
            await settings.init()
        }

        info = await invoke('crush')
        if (info) {
            hash = info.hash
            buildtime = info.build_date
            version = info.version
        }
    })
</script>

<svelte:window onkeydown={handleGlobalKey} />

{#if !settings.lockedInMode}
    <SmoothCursor />
{/if}

<Dialog
    open={creditDialog}
    onclose={() => {
        creditDialog = false
    }}
    title={$_('pages.settings.creditsCard.dialog.title')}
>
    <div class="flex flex-col gap-4 overflow-y-auto max-h-80 scrollbar-none">
        <div>
            <p class="text-lx1 mb-1">
                {$_('pages.settings.creditsCard.dialog.development')}
            </p>
            <p class="text-stone-300">Mally - Lead Developer</p>
            <p class="text-stone-300">
                Damon - Original Roblox optimizer's logic
            </p>
        </div>
        <div>
            <p class="text-lx1 mb-1">
                {$_('pages.settings.creditsCard.dialog.inspiration')}
            </p>
            <p class="text-stone-300">
                Bloxstrap, Frostrap, Funkstrap, AppleBlox, Voidstrap
            </p>
        </div>
        <div>
            <p class="text-lx1 mb-1">
                {$_('pages.settings.creditsCard.dialog.localization')}
            </p>
            <p class="text-stone-300">polover - Vietnamese</p>
        </div>
        <div>
            <p class="text-lx1 mb-1">
                {$_('pages.settings.creditsCard.dialog.specialThanks')}
            </p>
            <div class="flex flex-col gap-1 text-stone-300">
                <p>@miawuawua</p>
                <p>@polover1682</p>
                <p>@headlessangelwings</p>
                <p>@someonehelpme_12</p>
            </div>
        </div>
    </div>
    <div class="text-sm">Scroll down, there is more^^</div>
</Dialog>

<Dialog
    open={privacyAndDataDialog}
    onclose={() => {
        privacyAndDataDialog = false
    }}
    title={$_('pages.settings.pirvacyCard.title')}
>
    <div class="flex flex-col gap-4 overflow-y-auto max-h-80 scrollbar-none">
        <Checkbox bind:checked={settings.robloxWarpped}
            >{$_('pages.settings.pirvacyCard.dialog.warpped')}</Checkbox
        >
    </div>
</Dialog>

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
        {#snippet action()}
            <Dropdown
                bind:value={settings.currentLocale}
                options={$dropdownOptions}
            />
        {/snippet}
    </SettingCard>

    <SettingCard
        title="Cloud config"
        description="Sync your crush config to the cloud."
        icon={Cloud}
    >
        {#snippet action()}
            <Button
                variant="secondary"
                onclick={() => {
                    goto('./settings/cloud')
                }}
            >
                Open
            </Button>
        {/snippet}
    </SettingCard>

    <SettingCard
        title={$_('pages.settings.onBoardCard.title')}
        description={$_('pages.settings.onBoardCard.description')}
        icon={BookHeart}
    >
        {#snippet action()}
            <Button variant="danger" onclick={handleResetCrushOnboarding}>
                {$_('pages.settings.onBoardCard.button')}
            </Button>
        {/snippet}
    </SettingCard>

    <SettingCard
        title={$_('pages.settings.enableCrushRpcCard.title')}
        description={$_('pages.settings.enableCrushRpcCard.description')}
        icon={AudioWaveform}
    >
        {#snippet action()}
            <Switch bind:checked={settings.discordRpcEnabled} />
        {/snippet}
    </SettingCard>

    <SettingCard
        title={$_('pages.settings.lockedInCard.title')}
        description={$_('pages.settings.lockedInCard.description')}
        icon={Crosshair}
    >
        {#snippet action()}
            <Switch bind:checked={settings.lockedInMode} />
        {/snippet}
    </SettingCard>

    <SettingCard
        title={$_('pages.settings.redRingsCard.title')}
        description={$_('pages.settings.redRingsCard.description')}
        icon={Cuboid}
    >
        {#snippet action()}
            <Switch bind:checked={settings.redRings} />
        {/snippet}
    </SettingCard>

    <SettingCard
        title={$_('pages.settings.pirvacyCard.title')}
        description={$_('pages.settings.pirvacyCard.description')}
        icon={Shield}
    >
        {#snippet action()}
            <Button
                variant="secondary"
                onclick={() => {
                    privacyAndDataDialog = true
                }}>{$_('pages.settings.creditsCard.button')}</Button
            >
        {/snippet}
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
            <p class="sm">
                {$_('pages.settings.aboutCard.version', {
                    values: { version },
                })}
            </p>
            <p class="sm text-gray-600">
                {$_('pages.settings.aboutCard.note')}
            </p>
        </div>
    </ExpandableSettingCard>

    <SettingCard
        title={$_('pages.settings.creditsCard.title')}
        icon={ScrollText}
    >
        {#snippet action()}
            <Button
                variant="secondary"
                onclick={() => {
                    creditDialog = true
                }}>{$_('pages.settings.creditsCard.button')}</Button
            >
        {/snippet}
    </SettingCard>

    <ExpandableSettingCard
        title={$_('pages.settings.donateCard.title')}
        description={$_('pages.settings.donateCard.description')}
        icon={Arona}
    >
        <Button variant="secondary" onclick={handleDonate}
            >{$_('pages.settings.donateCard.button')}</Button
        >
    </ExpandableSettingCard>

    {#if devMode}
        <SettingCard title="Test page" icon={ScrollText} description="dev only">
            {#snippet action()}
                <Button
                    variant="secondary"
                    onclick={() => {
                        goto('./settings/dev')
                    }}>Open</Button
                >
            {/snippet}
        </SettingCard>
    {/if}
</div>

<div class="justify-center w-full flex flex-row gap-3">
    <Backlight blur={3}>
        <svg
            class="size-10"
            viewBox="0 0 256 308"
            preserveAspectRatio="xMidYMid"
        >
            <path
                d="M239.682 40.707C211.113-.182 154.69-12.301 113.895 13.69L42.247 59.356a82.198 82.198 0 0 0-37.135 55.056 86.566 86.566 0 0 0 8.536 55.576 82.425 82.425 0 0 0-12.296 30.719 87.596 87.596 0 0 0 14.964 66.244c28.574 40.893 84.997 53.007 125.787 27.016l71.648-45.664a82.182 82.182 0 0 0 37.135-55.057 86.601 86.601 0 0 0-8.53-55.577 82.409 82.409 0 0 0 12.29-30.718 87.573 87.573 0 0 0-14.963-66.244"
                fill="#FF3E00"
            />
            <path
                d="M106.889 270.841c-23.102 6.007-47.497-3.036-61.103-22.648a52.685 52.685 0 0 1-9.003-39.85 49.978 49.978 0 0 1 1.713-6.693l1.35-4.115 3.671 2.697a92.447 92.447 0 0 0 28.036 14.007l2.663.808-.245 2.659a16.067 16.067 0 0 0 2.89 10.656 17.143 17.143 0 0 0 18.397 6.828 15.786 15.786 0 0 0 4.403-1.935l71.67-45.672a14.922 14.922 0 0 0 6.734-9.977 15.923 15.923 0 0 0-2.713-12.011 17.156 17.156 0 0 0-18.404-6.832 15.78 15.78 0 0 0-4.396 1.933l-27.35 17.434a52.298 52.298 0 0 1-14.553 6.391c-23.101 6.007-47.497-3.036-61.101-22.649a52.681 52.681 0 0 1-9.004-39.849 49.428 49.428 0 0 1 22.34-33.114l71.664-45.677a52.218 52.218 0 0 1 14.563-6.398c23.101-6.007 47.497 3.036 61.101 22.648a52.685 52.685 0 0 1 9.004 39.85 50.559 50.559 0 0 1-1.713 6.692l-1.35 4.116-3.67-2.693a92.373 92.373 0 0 0-28.037-14.013l-2.664-.809.246-2.658a16.099 16.099 0 0 0-2.89-10.656 17.143 17.143 0 0 0-18.398-6.828 15.786 15.786 0 0 0-4.402 1.935l-71.67 45.674a14.898 14.898 0 0 0-6.73 9.975 15.9 15.9 0 0 0 2.709 12.012 17.156 17.156 0 0 0 18.404 6.832 15.841 15.841 0 0 0 4.402-1.935l27.345-17.427a52.147 52.147 0 0 1 14.552-6.397c23.101-6.006 47.497 3.037 61.102 22.65a52.681 52.681 0 0 1 9.003 39.848 49.453 49.453 0 0 1-22.34 33.12l-71.664 45.673a52.218 52.218 0 0 1-14.563 6.398"
                fill="#FFF"
            />
        </svg>
    </Backlight>

    <Backlight blur={3}>
        <svg
            class="size-10"
            preserveAspectRatio="xMidYMid"
            viewBox="0 0 256 289"
            ><path
                fill="#FFC131"
                d="M178.497 104.93c0 15.155-12.285 27.44-27.44 27.44-15.153 0-27.438-12.285-27.438-27.44 0-15.153 12.285-27.438 27.439-27.438s27.439 12.285 27.439 27.439Z"
            /><circle
                cx="104.911"
                cy="183.505"
                r="27.439"
                fill="#24C8DB"
                transform="rotate(180 104.91 183.505)"
            /><path
                fill="#FFC131"
                d="M207.93 192.86a104.766 104.766 0 0 1-36.168 14.717 73.586 73.586 0 0 0 3.617-33.176c29.953-10.472 49.82-38.964 49.292-70.69-.53-31.727-21.336-59.542-51.622-69.008-30.286-9.467-63.23 1.546-81.733 27.324a122.227 122.227 0 0 0-40.16 11.723C64.86 29.536 105.956-.445 152.244.005c46.288.45 86.794 31.224 99.636 75.697 12.841 44.473-5.026 92.103-43.95 117.157ZM52.404 92.832l25.693 3.118a73.586 73.586 0 0 1 3.243-14.593 104.766 104.766 0 0 0-28.936 11.475Z"
            /><path
                fill="#24C8DB"
                d="M47.913 95.577a104.766 104.766 0 0 1 36.419-14.842 73.46 73.46 0 0 0-4.116 33.3c-29.847 10.619-49.549 39.148-48.906 70.822.642 31.673 21.484 59.38 51.738 68.78 30.253 9.399 63.127-1.62 81.604-27.352a122.227 122.227 0 0 0 40.16-11.6C191.076 258.854 150 288.785 103.748 288.33c-46.252-.453-86.733-31.184-99.602-75.612-12.87-44.427 4.92-92.037 43.768-117.141Zm155.528 100.026-.5.25.5-.25Z"
            /></svg
        >
    </Backlight>

    <Backlight blur={3}>
        <svg fill="none" class="size-10" viewBox="0 0 54 33"
            ><g clip-path="url(#tailwindcss__a)"
                ><path
                    fill="#38bdf8"
                    fill-rule="evenodd"
                    d="M27 0c-7.2 0-11.7 3.6-13.5 10.8 2.7-3.6 5.85-4.95 9.45-4.05 2.054.513 3.522 2.004 5.147 3.653C30.744 13.09 33.808 16.2 40.5 16.2c7.2 0 11.7-3.6 13.5-10.8-2.7 3.6-5.85 4.95-9.45 4.05-2.054-.513-3.522-2.004-5.147-3.653C36.756 3.11 33.692 0 27 0zM13.5 16.2C6.3 16.2 1.8 19.8 0 27c2.7-3.6 5.85-4.95 9.45-4.05 2.054.514 3.522 2.004 5.147 3.653C17.244 29.29 20.308 32.4 27 32.4c7.2 0 11.7-3.6 13.5-10.8-2.7 3.6-5.85 4.95-9.45 4.05-2.054-.513-3.522-2.004-5.147-3.653C23.256 19.31 20.192 16.2 13.5 16.2z"
                    clip-rule="evenodd"
                /></g
            ><defs
                ><clipPath id="tailwindcss__a"
                    ><path fill="#fff" d="M0 0h54v32.4H0z" /></clipPath
                ></defs
            ></svg
        >
    </Backlight>
</div>
