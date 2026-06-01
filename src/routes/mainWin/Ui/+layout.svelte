<script lang="ts">
    import * as Sidebar from '$lib/components/ui/sidebar/index.js'

    import AppSidebar from '$lib/components/molecules/AppSidebar.svelte'
    import LaunchBar from '$lib/components/molecules/LaunchBar.svelte'
    import { preparePageTransition } from '$lib/pageTransition'
    import NotifyContainer from '$lib/components/molecules/NotifyContainer.svelte'

    let { children } = $props()
</script>

<div class="flex-1 flex min-h-0 overflow-hidden">
    <Sidebar.Provider>
        <AppSidebar />

        <div class="flex-1 flex flex-col min-w-0 h-full relative">
            <main class="flex-1 relative min-w-0 flex flex-col overflow-hidden">
                <div
                    class="flex-1 p-10 flex flex-col items-stretch gap-6 overflow-y-auto scrollbar-hide"
                    style="view-transition-name: page-content"
                >
                    {@render children?.()}
                </div>
            </main>

            <footer class="flex justify-end items-end p-4 shrink-0">
                <LaunchBar />
            </footer>
        </div>
    </Sidebar.Provider>
</div>
