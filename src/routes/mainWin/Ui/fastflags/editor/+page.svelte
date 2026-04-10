<script lang="ts">
    import { onMount } from 'svelte'
    import FastFlagTable from '$lib/components/organisms/FastFlagTable.svelte'
    import { getLatestVersion } from '$lib/downloadRoblox'
    import {
        getFastFlags,
        saveFastFlags,
    } from '$lib/fastflag/fastflagManagement'
    import { invoke } from '@tauri-apps/api/core'
    import { _ } from 'svelte-i18n'
    import { goto } from '$app/navigation'
    import Button from '$lib/components/atoms/Button.svelte'

    let flags: Record<string, string> = {}
    let version = ''

    onMount(async () => {
        await invoke('set_rpc', {
            details: $_('rpc.general'),
            stateText: $_('rpc.fastflag'),
        })
        version = await getLatestVersion()
        flags = await getFastFlags(version)
    })

    async function handleDelete(event: CustomEvent<string>) {
        const { [event.detail]: _, ...rest } = flags
        flags = rest
        await saveFastFlags(version, flags)
    }

    async function handleAdd(
        event: CustomEvent<{ name: string; value: string }>
    ) {
        const { name, value } = event.detail
        if (name in flags) return
        flags = { ...flags, [name]: value }
        await saveFastFlags(version, flags)
    }

    async function handleUpdate(
        event: CustomEvent<{ name: string; value: string }>
    ) {
        const { name, value } = event.detail
        flags = { ...flags, [name]: value }
        await saveFastFlags(version, flags)
    }

    function handleSearch(event: CustomEvent<string>) {
        const query = event.detail
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
        on:delete={handleDelete}
        on:add={handleAdd}
        on:update={handleUpdate}
        on:search={handleSearch}
    />
</div>
