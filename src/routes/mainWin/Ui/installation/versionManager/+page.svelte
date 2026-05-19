<script lang="ts">
    import Button from '$lib/components/atoms/Button.svelte'
    import SortableList from '$lib/components/molecules/SortableList.svelte'
    import type { AppType } from '$lib/types'
    import { Play, Plus, Trash2 } from '@lucide/svelte'
    import { load } from '@tauri-apps/plugin-store'
    import { onMount } from 'svelte'

    type Versions = {
        appType: AppType
        id: string
        installedAt: string
        versionHash: string
    }

    type VersionsConfig = {
        currentlyUsing: Versions
        versions: Versions[]
    }

    let currentlyUsing: Versions | null = null
    let items: Versions[] = []

    onMount(async () => {
        const raw = await load('versions.json')

        const versions = await raw.get<Versions[]>('versions')
        const current = await raw.get<Versions>('currentlyUsing')

        console.log('versions:', versions)
        console.log('currentlyUsing:', current)

        items = versions ?? []
        currentlyUsing = current ?? null
    })
</script>

<div class="flex flex-col gap-8">
    <div class="flex items-center justify-between">
        <div>
            <h1 class="text-3xl font-bold tracking-tight text-stone-100">
                Versions manager
            </h1>
            <p class="text-stone-400 mt-1">
                Easily manage your roblox versions.
            </p>
        </div>
    </div>

    <Button variant="primary" size="md">
        <Plus class="size-4 mr-2" />
    </Button>

    <div class="space-y-4">
        {#if items.length === 0}
            <div
                class="p-8 border-2 border-dashed border-stone-800 rounded-xl text-center"
            >
                <p class="text-stone-500">No installations yet.</p>
            </div>
        {:else}
            <SortableList {items} let:item>
                <div class="flex items-center justify-between w-full pr-2">
                    <div class="flex flex-col gap-0.5">
                        <span class="text-sm font-medium text-stone-100">
                            {item.id}
                            {#if currentlyUsing?.id === item.id}
                                <span class="ml-2 text-xs text-green-400"
                                    >(active)</span
                                >
                            {/if}
                        </span>
                        <span class="text-xs text-stone-400"
                            >{item.versionHash}</span
                        >
                    </div>
                    <div class="flex items-center gap-1.5">
                        <Button size="sm" variant="ghost">
                            <Play class="size-4" />
                        </Button>
                        <Button size="sm" variant="ghost">
                            <Trash2 class="size-4" />
                        </Button>
                    </div>
                </div>
            </SortableList>
        {/if}
    </div>
</div>
