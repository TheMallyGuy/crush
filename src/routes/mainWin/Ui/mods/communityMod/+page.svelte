<script lang="ts">
    import Button from "$lib/components/atoms/Button.svelte";
    import { Frown } from "@lucide/svelte";
    import { openUrl } from "@tauri-apps/plugin-opener";

    async function handleForm() {
        await openUrl("https://n8n-yhiwxdlu.ap-southeast-1.clawcloudrun.com/form/Crush/Mod-submission")
    }
</script>

<div class="flex flex-col items-center justify-center min-h-[60vh] gap-6 px-6 text-center">
    <Frown class="w-20 h-10 flex items-center justify-center"/>

  <div class="flex flex-col gap-2 max-w-sm">
    <h2 class="text-xl font-semibold tracking-tight">No mods yet</h2>
    <p class="text-muted-foreground text-sm leading-relaxed">
      We don't have any community mods yet. Be the first to contribute and help
      us grow the collection!
    </p>
  </div>

  <Button size="lg" class="gap-2" on:click={handleForm} disabled={true}>
    Submit a mod
  </Button>
</div>