<script lang="ts">
    import { goto } from '$app/navigation'
    import Button from '$lib/components/atoms/Button.svelte'
    import LoadingOverlay from '$lib/components/atoms/LoadingOverlay.svelte'
    import Switch from '$lib/components/atoms/Switch.svelte'
    import Dropdown from '$lib/components/molecules/Dropdown.svelte'
    import SettingCard from '$lib/components/molecules/SettingCard.svelte'
    import type { Integrations } from '$lib/types'
    import { invoke } from '@tauri-apps/api/core'
    import { load } from '@tauri-apps/plugin-store'
    import { onMount } from 'svelte'
    import { _ } from 'svelte-i18n'

    interface Server {
        country_code: string
        ip: string
        name: string
        phantun_available: boolean
        phantun_port: number | null
        port: number
        region: string
        ping: number | null
    }

    interface DropdownOption {
        label: string
        value: string
    }

    const TIMEOUT_MS = 60_000

    let isLoggedIn = $state(false)
    let isInAuthSwift = $state(false)
    let enableSwifttunnel = $state(false)
    let enableRouting = $state(true)
    let closeWhenTuff = $state(false)
    let perferedRegion = $state('Singapore')
    let assetsRouting = $state(false)
    let enableCountryBan = $state(false)

    let perferedRegionItems = $state<DropdownOption[]>([])

    let serverList = $state<Server[]>([])
    let authError = $state<string | null>(null)
    let timeoutId: ReturnType<typeof setTimeout> | null = null

    function countryFlag(code: string): string {
        return code
            .toUpperCase()
            .split('')
            .map((c) => String.fromCodePoint(0x1f1e6 - 65 + c.charCodeAt(0)))
            .join('')
    }

    function clearAuthTimeout() {
        if (timeoutId !== null) {
            clearTimeout(timeoutId)
            timeoutId = null
        }
    }

    async function fetchServers() {
        try {
            const raw = await invoke<string>('swift_get_servers')
            const parsed = JSON.parse(raw)
            serverList = parsed.servers as Server[]

            const regionNames = new Intl.DisplayNames(['en'], {
                type: 'region',
            })

            const countriesMap = new Map<string, string>()

            serverList.forEach((item) => {
                const fullName =
                    regionNames.of(item.country_code.toUpperCase()) ??
                    item.country_code

                if (!countriesMap.has(fullName)) {
                    countriesMap.set(fullName.toLowerCase(), fullName)
                }
            })

            perferedRegionItems = Array.from(countriesMap.entries()).map(
                ([value, label]) => ({ label, value })
            )

            console.log(serverList)
        } catch (e) {
            console.error('Failed to fetch servers', e)
        }
    }

    async function loadSettings() {
        try {
            const store = await load('config.json')
            const integrations = await store.get<Integrations>('integrations')
            if (integrations) {
                enableSwifttunnel = integrations.swifttunnel?.enable ?? false
                enableRouting = integrations.swifttunnel?.enableRouting ?? true
                perferedRegion =
                    integrations.swifttunnel?.perferedRegion ?? 'singapore'
                assetsRouting = integrations.swifttunnel?.assetsRouting ?? false
                closeWhenTuff =
                    integrations.swifttunnel?.disconnectWhenRobloxClosed ??
                    false
                enableCountryBan =
                    integrations.swifttunnel?.enableCountryBan ?? false
            }
        } catch (e) {
            console.error('Failed to load settings', e)
        }
    }

    async function saveSettings() {
        try {
            const store = await load('config.json')
            const current = await store.get<Integrations>('integrations')
            const newInte: Integrations = {
                ...current,
                swifttunnel: {
                    enable: enableSwifttunnel,
                    enableRouting: enableRouting,
                    disconnectWhenRobloxClosed: closeWhenTuff,
                    perferedRegion: perferedRegion,
                    assetsRouting: assetsRouting,
                    enableCountryBan: enableCountryBan,
                },
            }
            await store.set('integrations', newInte)
            await store.save()
        } catch (e) {
            console.error('Failed to save settings', e)
        }
    }

    async function auth() {
        authError = null
        isInAuthSwift = true

        timeoutId = setTimeout(() => {
            isInAuthSwift = false
            authError = $_('pages.integrations.swiftTunnel.loginTimeout')
            timeoutId = null
        }, TIMEOUT_MS)

        try {
            await invoke('start_browser_login')
            isLoggedIn = await invoke<boolean>('swift_is_logged_in')
            if (isLoggedIn) {
                await fetchServers()
                await loadSettings()
            }
        } catch (e) {
            authError = String(e)
            isLoggedIn = false
        } finally {
            clearAuthTimeout()
            isInAuthSwift = false
        }
    }

    onMount(async () => {
        try {
            await loadSettings()

            isLoggedIn = await invoke<boolean>('swift_is_logged_in')
            if (isLoggedIn) await fetchServers()
        } catch (e) {
            console.error('onMount check failed', e)
        }
    })
</script>

<div class="relative flex flex-col gap-8 min-h-screen">
    <LoadingOverlay
        visible={isInAuthSwift}
        blur={false}
        message={$_('pages.integrations.swiftTunnel.loadingMessage')}
    />

    <div class="flex items-center justify-between">
        <div>
            <h1 class="text-3xl font-bold tracking-tight text-stone-100">
                {$_('pages.integrations.swiftTunnel.title')}
            </h1>
            <p class="text-stone-400 mt-1">
                {$_('pages.integrations.swiftTunnel.description')}
            </p>
        </div>
        <div class="flex items-center gap-2">
            <Button variant="secondary" onclick={() => goto('../integrations')}>
                {$_('pages.integrations.swiftTunnel.back')}
            </Button>
        </div>
    </div>

    {#if !isLoggedIn}
        <div
            class="flex flex-col gap-3 items-center justify-center py-32 text-stone-400 border border-dashed border-stone-800 rounded-lg"
        >
            {$_('pages.integrations.swiftTunnel.notLoggedIn')}
            {#if authError}
                <p class="text-red-400 text-sm">{authError}</p>
            {/if}
            <Button onclick={auth} disabled={isInAuthSwift}>
                {$_('pages.integrations.swiftTunnel.loginButton')}
            </Button>
        </div>
    {:else}
        <SettingCard
            title={$_(
                'pages.integrations.swiftTunnel.enableTunnelingCard.title'
            )}
            description={$_(
                'pages.integrations.swiftTunnel.enableTunnelingCard.description'
            )}
        >
            <Switch
                slot="action"
                bind:checked={enableSwifttunnel}
                on:change={saveSettings}
            />
        </SettingCard>

        <SettingCard
            title={$_('pages.integrations.swiftTunnel.enableRoutingCard.title')}
            description={$_(
                'pages.integrations.swiftTunnel.enableRoutingCard.description'
            )}
        >
            <Switch
                slot="action"
                disabled={!enableSwifttunnel}
                bind:checked={enableRouting}
                on:change={saveSettings}
            />
        </SettingCard>

        <SettingCard
            title="Disconnect on Roblox is closed"
            description="Option to kill SwiftTunnel when roblox is closed"
        >
            <Switch
                slot="action"
                disabled={!enableSwifttunnel}
                bind:checked={closeWhenTuff}
                on:change={saveSettings}
            />
        </SettingCard>

        <SettingCard
            title="Route assets to multiple relays"
            description="This route assets to multiple relays, improving assets load speed. Might break authenthication, launch from web to avoid login."
        >
            <Switch
                slot="action"
                disabled={!enableSwifttunnel}
                bind:checked={assetsRouting}
                on:change={saveSettings}
            />
        </SettingCard>

        <SettingCard
            title={$_(
                'pages.integrations.swiftTunnel.enableCountryBanCard.title'
            )}
            description={$_(
                'pages.integrations.swiftTunnel.enableCountryBanCard.description'
            )}
        >
            <Switch
                slot="action"
                disabled={!enableSwifttunnel}
                bind:checked={enableCountryBan}
                on:change={saveSettings}
            />
        </SettingCard>

        <SettingCard
            title={$_(
                'pages.integrations.swiftTunnel.preferredRegionCard.title'
            )}
            description={$_(
                'pages.integrations.swiftTunnel.preferredRegionCard.description'
            )}
        >
            <Dropdown
                slot="action"
                options={perferedRegionItems}
                bind:value={perferedRegion}
                on:change={saveSettings}
            />
        </SettingCard>

        <div class="flex flex-col gap-3">
            <h2 class="text-xl font-bold tracking-tight text-stone-100">
                {$_('pages.integrations.swiftTunnel.serverList.title')}
            </h2>

            {#if serverList.length === 0}
                <p class="text-stone-500 text-sm">
                    {$_('pages.integrations.swiftTunnel.serverList.empty')}
                </p>
            {:else}
                {#each serverList as server}
                    <SettingCard
                        title="{countryFlag(server.country_code)} {server.name}"
                        description={server.region}
                    />
                {/each}
            {/if}
        </div>
    {/if}
</div>
