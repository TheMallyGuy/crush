<script lang="ts">
    import SettingCard from "$lib/components/molecules/SettingCard.svelte";
    import Dropdown from "$lib/components/molecules/Dropdown.svelte";
    import { ALargeSmall, MousePointer2 } from "@lucide/svelte";
    import Button from "$lib/components/atoms/Button.svelte";
    import { customCursor, customFont, ensureCursor, getModIdByName } from "$lib/mods/prebuiltMod";
    import { getCurrentInstallation } from "$lib/downloadRoblox";
    import { open } from '@tauri-apps/plugin-dialog';
    import { appDataDir, join, resolveResource } from "@tauri-apps/api/path"
    import { info } from "@tauri-apps/plugin-log"
    import { load } from "@tauri-apps/plugin-store"
    import { onMount } from "svelte"
    import { _ } from "svelte-i18n";

    let config

    let cursorOptions =  [
        { value: "default", label: $_("pages.mod.tab.prebuiltMods.cursorCard.dropdown.default") },
        { value: "2006", label: "2006" },
        { value: "2013", label: "2013" }
    ];

    let cursorValue = "default";

    async function handleCustomFont() {
        const file = await open({
            multiple: false,
            filters: [{
                name: 'Font',
                extensions: ['ttf']
            }]
        });
        if (!file) return

        const currentInstallation = await getCurrentInstallation()
        if (!currentInstallation) return

        const rblxFamilies = await join(currentInstallation.installPath, "content", "fonts", "families")
        customFont(file, rblxFamilies)
    }

    async function handleCustomCursor() {
        const config = await load("config.json")
        
        if (cursorValue == "default") {
            await config.set("Cursor", "default")
            ensureCursor()
            return
        }

        const Arrow = await resolveResource(`resources/Mods/Cursors/${cursorValue}/ArrowCursor.png`)
        const Far = await resolveResource(`resources/Mods/Cursors/${cursorValue}/ArrowFarCursor.png`)
        
        await config.set("Cursor", cursorValue)
        customCursor(Arrow, Far)
    }

    onMount(async () => {
        config = await load("config.json")

        cursorValue = await config.get('Cursor') ?? "default"
    })

</script>

<div class="flex flex-col gap-3">
    <SettingCard title={$_("pages.mod.tab.prebuiltMods.cursorCard.title")} description={$_("pages.mod.tab.prebuiltMods.cursorCard.description")} icon={MousePointer2}>
        <Dropdown slot="action" options={cursorOptions} bind:value={cursorValue} on:change={handleCustomCursor} />
    </SettingCard>

    <SettingCard title={$_("pages.mod.tab.prebuiltMods.customFontCard.title")} description={$_("pages.mod.tab.prebuiltMods.customFontCard.description")} icon={ALargeSmall}>
        <Button slot="action" variant='secondary' on:click={handleCustomFont}>
            {$_("pages.mod.tab.prebuiltMods.customFontCard.button")}
        </Button>
    </SettingCard>
</div>
