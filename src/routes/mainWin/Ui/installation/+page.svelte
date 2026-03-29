<script lang="ts">
    import { invoke } from '@tauri-apps/api/core'
    import SettingCard from '$lib/components/molecules/SettingCard.svelte'
    import { HardDrive, FolderOpen, RefreshCcw } from '@lucide/svelte'
    import { onMount } from 'svelte'
    import Button from '$lib/components/atoms/Button.svelte'

    onMount(async () => {
        await invoke('set_rpc', {
            details: 'A roblox bootstrapper written from scratch',
            stateText: 'In Installation Route',
        })
    })
</script>

<div class="flex flex-col gap-10 max-w-3xl">
    <header class="flex flex-col gap-2">
        <h1 class="text-clamp-3xl font-black tracking-tight text-white uppercase">
            Installation
        </h1>
        <p class="text-stone-500 font-medium max-w-xl">
            Manage your Roblox directories and client versions with precision.
        </p>
    </header>

    <div class="flex flex-col gap-4">
        <SettingCard
            title="Installation Path"
            description="The current directory where Roblox is installed and managed by Crush."
            icon={HardDrive}
            delay={100}
        >
             <div slot="footer" class="flex gap-2">
                <Button variant="outline" size="sm" class="font-bold text-[10px] uppercase tracking-wider">
                    <FolderOpen size={14} class="mr-2" />
                    Open Folder
                </Button>
                <Button variant="secondary" size="sm" class="font-bold text-[10px] uppercase tracking-wider">
                    Change Path
                </Button>
             </div>
        </SettingCard>

        <SettingCard
            title="Force Reinstallation"
            description="Clean and reinstall Roblox if you're experiencing client-side issues."
            icon={RefreshCcw}
            delay={200}
        >
            <Button slot="action" variant="danger" size="sm" class="font-bold text-[10px] uppercase tracking-wider">
                Reinstall
            </Button>
        </SettingCard>
    </div>
</div>

<style>
    .text-clamp-3xl {
        font-size: clamp(1.875rem, 5vw, 3rem);
    }
</style>
