<script>
  import Topbar from "./Topbar.svelte";
  import "./style.css";
  import "fluent-svelte/theme.css";
</script>

<div class="flex flex-col h-screen bg-stone-950 text-stone-200">
  <Topbar />

  <div class="flex flex-1 overflow-hidden">
    <slot />
  </div>
</div>