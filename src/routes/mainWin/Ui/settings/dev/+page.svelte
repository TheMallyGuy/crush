<script lang="ts">
    import { emit } from '@tauri-apps/api/event'
    import { onMount } from 'svelte'

    onMount(async () => {
        await emit('ai:listen')
    })
</script>

<p style="color: rgb(120,113,108); font-size: 12px; padding: 16px;">
    AI trigger fired — check the top of your screen.
</p>
