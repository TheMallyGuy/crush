<script lang="ts">
    import Button from '$lib/components/atoms/Button.svelte'
    import ExpandableSettingCard from '$lib/components/molecules/ExpandableSettingCard.svelte'
    import type { BuildInfo } from '$lib/types'
    import { Heart } from '@lucide/svelte'
    import { Info } from '@lucide/svelte'
    import { invoke } from '@tauri-apps/api/core'
    import { openUrl } from '@tauri-apps/plugin-opener'
    import { onMount } from 'svelte'

    const Arona = '/Arona.png'
    let info: BuildInfo
    let hash: string
    let buildtime: string

    onMount(async () => {
        info = await invoke('crush')

        hash = info.hash
        buildtime = info.build_date
    })

    async function handleDonate() {
        openUrl('https://github.com/TheMallyGuy')
    }
</script>

<div class="flex flex-col gap-4">
    <div class="flex items-center justify-between">
        <div>
            <h1 class="text-3xl font-bold tracking-tight text-stone-100">
                Settings
            </h1>
        </div>
    </div>

    <ExpandableSettingCard
        title="About"
        description="Infomation about this crush build"
        icon={Info}
    >
        <div>
            <p class="sm">Built on : {hash}</p>
            <p class="sm">At : {buildtime}</p>
        </div>
    </ExpandableSettingCard>

    <ExpandableSettingCard
        title="Donate"
        description="Love crush? Donate to Mally to support his work!"
        icon={Arona}
    >
        <Button variant="secondary" on:click={handleDonate}>Donate</Button>
    </ExpandableSettingCard>
</div>
