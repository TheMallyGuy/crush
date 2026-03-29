<script>
    import Topbar from '$lib/ui/organisms/Topbar.svelte'
    import './style.css'
    import './font.css'
</script>

<div class="flex flex-col h-screen bg-stone-950 text-stone-200">
    <div class="flex flex-1 overflow-hidden">
        <Topbar />

        <slot />
    </div>
</div>
