<script context="module">
  import { waitLocale } from 'svelte-i18n';
  export const load = async () => {
    await waitLocale();
  };
</script>

<script>
  import Topbar from '$lib/components/organisms/Topbar.svelte';
  import './style.css';
  import './font.css';
</script>

<div class="flex flex-col h-screen bg-stone-950 text-stone-200">
  <div class="flex flex-1 overflow-hidden">
    <Topbar />
    <slot />
  </div>
</div>