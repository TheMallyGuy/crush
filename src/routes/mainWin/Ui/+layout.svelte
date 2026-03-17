<script lang="ts">
  import Sidebar from "$lib/components/Sidebar.svelte";
</script>

<div class="flex h-screen">
  <Sidebar />

  <main class="flex-1 overflow-auto text-white p-6">
    <slot />
  </main>
</div>