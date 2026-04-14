<script lang="ts">
    import Button from '$lib/components/atoms/Button.svelte'
    import Textbox from '$lib/components/atoms/Textbox.svelte'
    import type { BoostrapConfigs, Installation, Integrations, RoValra } from '$lib/types'
    import { invoke } from '@tauri-apps/api/core'
    import { load } from '@tauri-apps/plugin-store'

    let userBasePath: string

    async function importConfigs(basePath: string) {
        try {
            const boostraperConfig: BoostrapConfigs = await invoke("export_boostrapconfig", {
                boostrapConfigPath: basePath
            })

            const config = await load("config.json")
            const integrations = await config.get<Integrations>('integrations')
            
            const roValra: RoValra = {
                joinServerForYouValue: boostraperConfig.EnableBetterMatchmaking ?? false
            }
            
            const newIntegrations: Integrations = {
                serverLocationNotifier: boostraperConfig.ShowServerDetails ?? false,
                roValra: roValra,
                gameCache: {}, // none
                ...integrations,
                crushRpc: boostraperConfig.UseDiscordRichPresence ?? false
            }

            await config.set('integrations', newIntegrations)
            await config.save()
        } catch (e) {
            console.error("Failed to import configs:", e)
        }
    }
</script>

<div class="flex flex-col gap-2">
    <p class="text-stone-300 text-base">Export other boostraper configs that based on Bloxstrap to crush. (Tested : Bloxstrap, Frostrap)</p>

    <Textbox placeholder="C:\Users\Mally\AppData\Local\Bloxstrap" bind:value={userBasePath}/>

    <Button on:click={() => importConfigs(userBasePath)}>
        Export
    </Button>
</div>