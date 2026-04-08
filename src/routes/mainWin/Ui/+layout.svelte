<script lang="ts">
    import Sidebar from '$lib/components/molecules/Sidebar.svelte'
    import LaunchBar from '$lib/components/molecules/LaunchBar.svelte'
    import { fly } from 'svelte/transition'
    import { page } from '$app/stores'
</script>

<div class="flex h-screen w-screen overflow-hidden bg-black text-white">
    <Sidebar />

    <div class="flex-1 flex flex-col min-w-0">
        <main
            class="flex-1 relative min-w-0 overflow-y-auto"
        >
            {#key $page.url.pathname}
                <div
                    in:fly={{ x: 20, duration: 200, delay: 200 }}
                    out:fly={{ x: -20, duration: 200 }}
                    class="absolute inset-0 p-10 flex flex-col items-stretch gap-6"
                >
                    <slot />
                </div>
            {/key}
        </main>

        <footer class="flex justify-end items-end shrink-0">
            <LaunchBar />
        </footer>
    </div>
</div>
