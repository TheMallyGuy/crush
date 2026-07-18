<script lang="ts">
    import { onMount } from 'svelte'
    import { island, initIslandListener } from '$lib/island'
    import DynamicIslandNotify from '$lib/components/molecules/DynamicIslandNotify.svelte'
    import { flip } from 'svelte/animate'
    import { operating_system } from '$lib/stores/operating_system.svelte'
    import { getCurrentWindow } from '@tauri-apps/api/window'

    async function sleep(ms: number): Promise<void> {
        return new Promise((resolve) => setTimeout(resolve, ms))
    }

    onMount(() => {
        const unlisten = initIslandListener()
        return () => {
            unlisten.then((fn) => fn())
        }

        sleep(3000)

        if ($operating_system === 'macos') {
            getCurrentWindow().close()
        }
    })
</script>

<div
    class="flex h-screen w-screen flex-col items-center gap-2 overflow-hidden pt-2 select-none"
>
    {#each $island as n (n.id)}
        <div animate:flip={{ duration: 220 }}>
            <DynamicIslandNotify
                title={n.title}
                description={n.description}
                image={n.image}
                ondismiss={() => island.dismiss(n.id)}
            />
        </div>
    {/each}
</div>
