<script lang="ts">
    import { notify } from '$lib/notify'
    import Notify from './Notify.svelte'
    import { flip } from 'svelte/animate'
</script>

<div class="fixed bottom-20 right-4 z-50 flex flex-col gap-2 w-80">
    {#each $notify as n (n.id)}
        <div animate:flip={{ duration: 200 }}>
            <Notify
                variant={n.variant}
                title={n.title}
                description={n.description}
                on:dismiss={() => notify.dismiss(n.id)}
            />
        </div>
    {/each}
</div>