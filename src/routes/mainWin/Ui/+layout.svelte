<script lang="ts">
    import Sidebar from '$lib/components/molecules/Sidebar.svelte'
    import LaunchBar from '$lib/components/molecules/LaunchBar.svelte'
</script>

<div class="flex h-screen w-screen overflow-hidden bg-black text-white">
    <Sidebar />

    <div class="flex-1 flex flex-col min-w-0">
        <main
            class="flex-1 flex flex-col items-stretch min-w-0 overflow-y-auto p-10 gap-6"
        >
            <slot />
        </main>

        <footer class="flex justify-end items-end shrink-0">
            <LaunchBar />
        </footer>
    </div>
</div>
