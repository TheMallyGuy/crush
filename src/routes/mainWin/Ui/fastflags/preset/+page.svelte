<script lang="ts">
    import { onMount } from 'svelte'
    import { getLatestVersion } from '$lib/downloadRoblox'
    import {
        getFastFlags,
        saveFastFlags,
    } from '$lib/fastflag/fastflagManagement'
    import { invoke } from '@tauri-apps/api/core'
    import { _ } from 'svelte-i18n'
    import { goto } from '$app/navigation'
    import Button from '$lib/components/atoms/Button.svelte'
    import SettingCard from '$lib/components/molecules/SettingCard.svelte'

    let flags: Record<string, string> = {}
    let version = ''

    let msaaropdownItems = {
        "0" : "x0",
        "1" : "x1",
        "2" : "x2",
        "4" : "x4",
        "8" : "x8"
    }
    let msaaDropdownDefault: string = "0"
    
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

</script>

<SettingCard 
  title="Anti-Aliasing Quality (MSAA)" 
  description="Smooths jagged edges. Higher values look better but may reduce performance." 
>
</SettingCard>

<SettingCard 
  title="Pause Voxelier" 
  description="Stops voxelization to improve performance" 
/>

<SettingCard 
  title="Grass waving animation" 
  description="Adjusts the intensity of grass movement" 
/>

<SettingCard 
  title="Overwrite texture quality level" 
  description="Forces a custom texture quality level" 
/>