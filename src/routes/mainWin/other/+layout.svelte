<script lang="ts">
    import Topbar from "$lib/components/organisms/Topbar.svelte";
</script>

<div class="flex flex-col h-screen w-screen overflow-hidden  text-white">
  <Topbar />
  <div class="flex-1 flex flex-col">
    <main class="flex-1 relative min-w-0 overflow-hidden">
      <div class="absolute inset-0 p-10 flex flex-col items-stretch gap-6 overflow-y-auto scrollbar-hide"
          style="view-transition-name: page-content">
        <slot />
      </div>
    </main>
  </div>
</div>