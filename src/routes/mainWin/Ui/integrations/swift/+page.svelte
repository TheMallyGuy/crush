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
    let perferedRegion = $state('Singapore')

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
            }
        } catch (e) {
            console.error('Failed to load settings', e)
        }
    }

    async function saveSettings() {
        try {
            const newInte: Integrations = {
                swifttunnel: {
                    enable: enableSwifttunnel,
                    enableRouting: enableRouting,
                    perferedRegion: perferedRegion,
                },
            }
            const store = await load('config.json')
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
            authError = 'Login timed out. Please try again.'
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
        message="Please continue in your browser. Auto-cancels after 1 min."
    />

    <div class="flex items-center justify-between">
        <div>
            <h1 class="text-3xl font-bold tracking-tight text-stone-100">
                SwiftTunnel
            </h1>
            <p class="text-stone-400 mt-1">
                Configure SwiftTunnel right in crush.
            </p>
        </div>
        <div class="flex items-center gap-2">
            <Button variant="secondary" onclick={() => goto('../integrations')}>
                Back
            </Button>
        </div>
    </div>

    {#if !isLoggedIn}
        <div
            class="flex flex-col gap-3 items-center justify-center py-32 text-stone-400 border border-dashed border-stone-800 rounded-lg"
        >
            You're not logged in.
            {#if authError}
                <p class="text-red-400 text-sm">{authError}</p>
            {/if}
            <Button onclick={auth} disabled={isInAuthSwift}>
                Login via browser
            </Button>
        </div>
    {:else}
        <SettingCard
            title="Enable Tunneling"
            description="Enable tunneling to route Roblox through the VPN"
        >
            <Switch
                slot="action"
                bind:checked={enableSwifttunnel}
                on:change={saveSettings}
            />
        </SettingCard>

        <SettingCard
            title="Enable Roblox Routing"
            description="Route all roblox API request to the VPN. This can be use to evade blocked game in some country."
        >
            <Switch
                slot="action"
                disabled={!enableSwifttunnel}
                bind:checked={enableRouting}
                on:change={saveSettings}
            />
        </SettingCard>

        <SettingCard
            title="Prefered Region"
            description="Config your perfered region"
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
                Server List
            </h2>

            {#if serverList.length === 0}
                <p class="text-stone-500 text-sm">No servers available.</p>
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
