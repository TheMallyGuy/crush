<script lang="ts">
    import SettingCard from "$lib/components/molecules/SettingCard.svelte";
    import Dropdown from "$lib/components/molecules/Dropdown.svelte";
    import Button from "$lib/components/atoms/Button.svelte";

    let cursorOptions =  [
        { value: "default", label: "Default" },
        { value: "2006", label: "2006" },
        { value: "2007", label: "2007" }
    ];
    let cursorValue = "default";
</script>

<div class="flex flex-col gap-3">
    <SettingCard title="Cursor" description="Change your roblox cursor to the old one.">
        <Dropdown slot="action" options={cursorOptions} bind:value={cursorValue} />
    </SettingCard>

    <SettingCard title="Custom Font" description="Change your roblox font to whatever you like.">
        <Button slot="action" variant='secondary'>Change Font</Button>
    </SettingCard>
</div>
