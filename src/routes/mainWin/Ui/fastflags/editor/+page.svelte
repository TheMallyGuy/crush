<script lang="ts">
    import { onMount } from 'svelte'
    import FastFlagTable from '$lib/components/organisms/FastFlagTable.svelte'
    import {
        getFastFlags,
        saveFastFlags,
    } from '$lib/fastflag/fastflagManagement'
    import { invoke } from '@tauri-apps/api/core'
    import { _ } from 'svelte-i18n'
    import { goto } from '$app/navigation'
    import { get } from 'svelte/store'
    import { launchAppType } from '$lib/stores/launchAppType'
    import type { AppType } from '$lib/types'
    import Button from '$lib/components/atoms/Button.svelte'

    let flags: Record<string, string> = $state({})
    let appType: AppType = $state('player')

    onMount(async () => {
        appType = (get(launchAppType) as AppType) || 'player'

        flags = await getFastFlags(appType)

        console.log('Current fast flags:', flags)

        invoke('set_rpc', {
            details: $_('rpc.general'),
            stateText: $_('rpc.fastflag'),
        })
    })

    async function handleDelete(name: string) {
        const latestFlags = await getFastFlags(appType)
        const { [name]: _, ...rest } = latestFlags
        flags = rest
        await saveFastFlags(flags, appType)
    }

    async function handleAdd(payload: { name: string; value: string }) {
        const { name, value } = payload
        const latestFlags = await getFastFlags(appType)
        if (name in latestFlags) return
        flags = { ...latestFlags, [name]: value }
        await saveFastFlags(flags, appType)
    }

    async function handleUpdate(payload: { name: string; value: string }) {
        const { name, value } = payload
        const latestFlags = await getFastFlags(appType)
        flags = { ...latestFlags, [name]: value }
        await saveFastFlags(flags, appType)
    }

    function handleSearch(query: string) {
        console.log(`Searching for: ${query}`)
    }
</script>

<div class="flex flex-col gap-8">
    <div class="flex items-center justify-between">
        <div>
            <h1 class="text-3xl font-bold tracking-tight text-stone-100">
                {$_('pages.fastflag.fastflag')}
            </h1>
            <p class="text-stone-400 mt-1">
                {$_('pages.fastflag.description')}
            </p>
        </div>
        <div class="flex items-center gap-2">
            <Button variant="secondary" onclick={() => goto('../fastflags')}>
                {$_('pages.fastflag.generalBack')}
            </Button>
        </div>
    </div>
    <FastFlagTable
        {flags}
        ondelete={handleDelete}
        onadd={handleAdd}
        onupdate={handleUpdate}
        onsearch={handleSearch}
        onimport={async (imported) => {
            const latestFlags = await getFastFlags(appType)
            flags = { ...latestFlags, ...imported }
            await saveFastFlags(flags, appType)
        }}
    />
</div>
