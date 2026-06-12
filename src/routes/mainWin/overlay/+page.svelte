<script lang="ts">
    import { onMount } from 'svelte'
    import { island, initIslandListener } from '$lib/island'
    import DynamicIslandNotify from '$lib/components/molecules/DynamicIslandNotify.svelte'
    import { flip } from 'svelte/animate'

    onMount(() => {
        const unlisten = initIslandListener()
        return () => {
            unlisten.then(fn => fn())
        }
    })
</script>

<div class="flex h-screen w-screen flex-col items-center gap-2 overflow-hidden pt-2 select-none">
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
