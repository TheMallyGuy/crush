<script lang="ts">
    import Sidebar from '$lib/components/Sidebar.svelte'
</script>

<div class="flex h-screen w-screen overflow-hidden bg-black text-white">
    <Sidebar />

    <main
        class="flex-1 flex flex-col items-stretch min-w-0 overflow-y-auto p-10 gap-6"
    >
        <slot />
    </main>
</div>
