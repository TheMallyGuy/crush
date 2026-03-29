<script lang="ts">
    import ClickableCard from "$lib/components/molecules/ClickableCard.svelte";
    import { Archive, Hammer, HouseHeart } from "@lucide/svelte";
    import { goto } from '$app/navigation';

    function handleModManagementClick() {
        goto('/mainWin/Ui/mods/modManagement');
    }
    function handlePrebuiltModsClick() {
        goto('/mainWin/Ui/mods/prebuiltMod');
    }
    function handleCommunityModsClick() {
        goto('/mainWin/Ui/mods/communityMod');
    }
</script>

<div class="flex flex-col gap-4">
    <div
        class="flex gap-2 "
    >
        <ClickableCard icon={Archive} title="Mod Management" description="Manage your installed mods" size="sm" on:click={handleModManagementClick}></ClickableCard>
        <ClickableCard icon={Hammer} title="Prebuilt Mods" description="Install ready-to-use mod packages" size="sm" on:click={handlePrebuiltModsClick}></ClickableCard>
        <ClickableCard icon={HouseHeart} title="Community Mods" description="Explore mods created by the community" size="sm" on:click={handleCommunityModsClick}></ClickableCard>
    </div>

    <slot />
</div>