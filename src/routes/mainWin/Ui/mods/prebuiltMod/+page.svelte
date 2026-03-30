<script lang="ts">
    import SettingCard from "$lib/components/molecules/SettingCard.svelte";
    import Dropdown from "$lib/components/molecules/Dropdown.svelte";
    import Button from "$lib/components/atoms/Button.svelte";
    import { customFont } from "$lib/mods/prebuiltMod";
    import { getCurrentInstallation } from "$lib/downloadRoblox";
    import { open } from '@tauri-apps/plugin-dialog';
    import { appDataDir, join } from "@tauri-apps/api/path"

    let cursorOptions =  [
        { value: "default", label: "Default" },
        { value: "2006", label: "2006" },
        { value: "2007", label: "2007" }
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

</script>

<div class="flex flex-col gap-3">
    <SettingCard title="Cursor" description="Change your roblox cursor to the old one.">
        <Dropdown slot="action" options={cursorOptions} bind:value={cursorValue} />
    </SettingCard>

    <SettingCard title="Custom Font" description="Change your roblox font to whatever you like.">
        <Button slot="action" variant='secondary' on:click={handleCustomFont}>Change Font</Button>
    </SettingCard>
</div>
