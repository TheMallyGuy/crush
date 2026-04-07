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
            class="flex-1 flex flex-col items-stretch min-w-0 overflow-y-auto p-10 gap-6"
        >
            {#key $page.url.pathname}
                <div
                    in:fly={{ x: 20, duration: 200, delay: 200 }}
                    out:fly={{ x: -20, duration: 200 }}
                    class="flex flex-col gap-6"
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
