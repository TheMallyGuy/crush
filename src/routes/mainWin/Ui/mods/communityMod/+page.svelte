<script lang="ts">
    import Button from "$lib/components/atoms/Button.svelte";
    import { Frown } from "@lucide/svelte";
    import { openUrl } from "@tauri-apps/plugin-opener";
    import { _ } from "svelte-i18n";

    async function handleForm() {
        await openUrl("https://n8n-xpkqsnsc.ap-southeast-1.clawcloudrun.com/form/Crush/Mod-submission")
    }
</script>

<div class="flex flex-col items-center justify-center min-h-[60vh] gap-6 px-6 text-center">
    <Frown class="w-20 h-10 flex items-center justify-center"/>

  <div class="flex flex-col gap-2 max-w-sm">
    <h2 class="text-xl font-semibold tracking-tight">{$_("pages.mod.tab.communityMods.notice.title")}</h2>
    <p class="text-muted-foreground text-sm leading-relaxed">
      {$_("pages.mod.tab.communityMods.notice.description")}
    </p>
  </div>

  <Button size="lg" class="gap-2" on:click={handleForm}>
    {$_("pages.mod.tab.communityMods.notice.submissionButton")}
  </Button>
</div>