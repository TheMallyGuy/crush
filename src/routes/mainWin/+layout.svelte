<script context="module">
    import { waitLocale } from 'svelte-i18n'
    import './style.css'
    export const load = async () => {
        await waitLocale()
    }
</script>

<script>
    import Topbar from '$lib/components/organisms/Topbar.svelte'
    import './style.css'
    import './font.css'
</script>

<div class="flex flex-col h-screen bg-background text-foreground">
    <div class="flex flex-1 overflow-hidden">
        <Topbar />
        <slot />
    </div>
</div>
