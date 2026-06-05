<script lang="ts">
    import { onMount } from 'svelte'
    import { _ } from 'svelte-i18n'
    import ModCard from '$lib/components/molecules/ModCard.svelte'
    import { fetchMods } from '$lib/communityAdapter'
    import { type universalAdapter } from '$lib/communityAdapter'
    import { createNewMod, getModPath } from '$lib/mods/modManagement'
    import { importThemeFromDir } from '$lib/theme/themeLoader'
    import { mkdir, remove, writeFile } from '@tauri-apps/plugin-fs'
    import { fetch as tauriFetch } from '@tauri-apps/plugin-http' // prevent confusion
    import { appCacheDir, join } from '@tauri-apps/api/path'
    import { invoke } from '@tauri-apps/api/core'
    import { info } from '@tauri-apps/plugin-log'

    let mods: universalAdapter[] = []
    let loading = true
    let downloading: Record<string, boolean> = {}

    onMount(async () => {
        try {
            const [kliko, froststrap] = await Promise.all([
                fetchMods('kliko'),
                fetchMods('froststrap'),
            ])
            mods = [
                // prevent dups ids
                ...(Array.isArray(kliko) ? kliko : []).map((m) => ({
                    ...m,
                    id: `kliko-${m.id}`,
                })),
                ...(Array.isArray(froststrap) ? froststrap : []).map((m) => ({
                    ...m,
                    id: `froststrap-${m.id}`,
                })),
            ]
        } finally {
            loading = false
        }
    })

    async function handleDownload(mod: universalAdapter) {
        if (downloading[mod.id]) return
        downloading[mod.id] = true
        try {
            const zipName = `${mod.id}.zip`
            const cacheDir = await appCacheDir()
            const modsCacheDir = await join(cacheDir, 'modsCache')
            const zipPath = await join(modsCacheDir, zipName)

            await mkdir(modsCacheDir, { recursive: true })

            const response = await tauriFetch(mod.download)
            if (!response.ok)
                throw new Error(`Download failed: ${response.status}`)
            const bytes = new Uint8Array(await response.arrayBuffer())
            await writeFile(zipPath, bytes)

            if (mod.modType === 'boostrap') {
                const extractDir = await join(modsCacheDir, mod.id)
                await mkdir(extractDir, { recursive: true })
                await invoke('extract_zip', { zipPath, dest: extractDir })

                info(`importing bootstrap theme from ${extractDir}`)
                await importThemeFromDir(extractDir, mod.name)

                await remove(extractDir, { recursive: true }).catch(() => {})
            } else {
                const modName = `[${mod.adapter}] ${mod.name}`
                await createNewMod(modName)
                const destPath = await getModPath(modName)
                await invoke('extract_zip', { zipPath, dest: destPath })
            }

            await remove(zipPath).catch(() => {})
        } finally {
            downloading[mod.id] = false
            downloading = downloading
        }
    }
</script>

{#if loading}
    <div class="flex min-h-[60vh] items-center justify-center">
        <p class="text-sm text-stone-500">Loading mods…</p>
    </div>
{:else if mods.length === 0}
    <div class="flex min-h-[60vh] items-center justify-center">
        <p class="text-sm text-stone-500">No mods found.</p>
    </div>
{:else}
    <div
        class="grid gap-4 p-4"
        style="grid-template-columns: repeat(auto-fill, minmax(12rem, 1fr))"
    >
        {#each mods as mod (mod.id)}
            <ModCard
                name={mod.name}
                author={mod.author}
                adapter={mod.adapter}
                thumbnail={mod.thumbnail}
                downloading={downloading[mod.id] ?? false}
                on:click={() => handleDownload(mod)}
            />
        {/each}
    </div>
{/if}
