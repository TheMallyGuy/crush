<script lang="ts">
    import ClickableCard from "$lib/components/molecules/ClickableCard.svelte";
    import { Archive, Hammer, HouseHeart } from "@lucide/svelte";
    import { goto } from '$app/navigation';
    import { _ } from "svelte-i18n";
    import { onMount } from "svelte"
    import { invoke } from '@tauri-apps/api/core'


    function handleModManagementClick() {
        goto('/mainWin/Ui/mods/modManagement');
    }
    function handlePrebuiltModsClick() {
        goto('/mainWin/Ui/mods/prebuiltMod');
    }
    function handleCommunityModsClick() {
        goto('/mainWin/Ui/mods/communityMod');
    }

    onMount(() => {
        invoke('set_rpc', {
            details: $_('rpc.general'),
            stateText: $_('rpc.mod'),
        })
    })

</script>

<div class="flex flex-col gap-4">
    <div
        class="flex gap-2"
    >
        <ClickableCard icon={Archive} title={$_("pages.mod.tab.modManagement.title")} description={$_("pages.mod.tab.modManagement.description")} size="sm" on:click={handleModManagementClick}></ClickableCard>
        <ClickableCard icon={Hammer} title={$_("pages.mod.tab.prebuiltMods.title")} description={$_("pages.mod.tab.prebuiltMods.description")} size="sm" on:click={handlePrebuiltModsClick}></ClickableCard>
        <ClickableCard icon={HouseHeart} title={$_("pages.mod.tab.communityMods.title")} description={$_("pages.mod.tab.communityMods.description")} size="sm" on:click={handleCommunityModsClick}></ClickableCard>
    </div>

    <slot />
</div>