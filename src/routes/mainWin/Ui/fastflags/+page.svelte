<script lang="ts">
    import ClickableCard from "$lib/components/molecules/ClickableCard.svelte";
    import { Settings2, SquarePen } from "@lucide/svelte";
    import { goto } from '$app/navigation';
    import { _ } from "svelte-i18n";
    import { onMount } from "svelte"
    import { invoke } from '@tauri-apps/api/core'


    function handleEditorClick() {
        goto('/mainWin/Ui/fastflags/editor');
    }
    function handlePresetClick() {
        goto('/mainWin/Ui/fastflags/preset');
    }


    onMount(() => {
        invoke('set_rpc', {
            details: $_('rpc.general'),
            stateText: $_('rpc.mod'),
        })
    })

</script>

<div class="flex flex-col gap-5 justify-center items-center h-full w-full">
    <h1 class="text-3xl font-semibold">Please choose an action.</h1>
    <div class="flex flex-col gap-2 items-center">
        <div class="flex flex-col gap-2">
            <ClickableCard icon={SquarePen} title="Editor" description="Edit Feature Flags." size="sm" on:click={handleEditorClick}></ClickableCard>
            <ClickableCard icon={Settings2} title="Preset" description="Use Feature Flag Preset built in in crush." size="sm" on:click={handlePresetClick}></ClickableCard>
        </div>
    </div>
</div>