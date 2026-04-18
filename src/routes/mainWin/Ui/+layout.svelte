<script lang="ts">
    import Sidebar from '$lib/components/molecules/Sidebar.svelte'
    import LaunchBar from '$lib/components/molecules/LaunchBar.svelte'
    import { preparePageTransition } from '$lib/page-transition'

    preparePageTransition()
</script>

<div class="flex h-screen w-screen overflow-hidden bg-transparent text-white">
    <Sidebar />
    <div class="flex-1 flex flex-col min-w-0">
        <main class="flex-1 relative min-w-0 overflow-hidden">
            <div class="absolute inset-0 p-10 flex flex-col items-stretch gap-6 overflow-y-auto scrollbar-hide"
                style="view-transition-name: page-content">
                <slot />
            </div>
        </main>
        <footer class="flex justify-end items-end shrink-0">
            <LaunchBar />
        </footer>
    </div>
</div>