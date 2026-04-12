<script lang="ts">
    import WizardSidebar from '$lib/components/molecules/WizardSidebar.svelte'
    import Button from '$lib/components/atoms/Button.svelte'
    import { page } from '$app/stores';
    import { steps, currentStep, isFirst, isLast, next, back } from '$lib/stores/wizard'
    import { invoke } from '@tauri-apps/api/core'
    import { getCurrentWindow } from '@tauri-apps/api/window'
    import { load } from '@tauri-apps/plugin-store';


    $: {
        const idx = steps.findIndex(s => s.path === $page.url.pathname)
        if (idx !== -1) currentStep.set(idx)
    }

    async function imBadAtNamingHandlerFunctions() {
        const config = await load("config.json")

        await config.set("firstLaunch", false)

        await invoke('create_or_focus_window', {
            label: 'CrushMainWindow',
            url: 'mainWin/Ui/intergrations',
            title: 'Crush',
            width: 1000,
            height: 600,
            minWidth: 1000,
            minHeight: 600,
        })

        setTimeout(() => {
            // wait before killing to prevent crash
            getCurrentWindow().close()
        }, 100)
    }
</script>

<div class="flex h-screen w-screen bg-black text-white">
    <WizardSidebar {steps} currentStep={$currentStep} />

    <div class="flex-1 flex flex-col p-10 gap-10">
        <div class="flex-grow flex flex-col gap-4">
            <h1 class="text-4xl font-bold">{steps[$currentStep].label}</h1>

            <slot />
        </div>

        <div class="flex justify-end gap-3 pt-6">
            {#if !$isFirst}
                <Button on:click={back} variant="secondary">Back</Button>
            {/if}

            {#if !$isLast}
                <Button on:click={next}>Next</Button>
            {:else}
                <Button on:click={imBadAtNamingHandlerFunctions}>Finish</Button>
            {/if}
        </div>
    </div>
</div>