<script lang="ts">
    import type { BloxstrapConfig, Installation, Integrations, RoValra } from '$lib/types'
    import { invoke } from '@tauri-apps/api/core'
    import { load } from '@tauri-apps/plugin-store'

    async function importConfigs(basePath: string) {
        try {
            const boostraperConfig: BloxstrapConfig = await invoke("export_boostrapconfig", {
                boostrapConfigPath: basePath
            })

            const config = await load("config.json")
            const integrations = await config.get<Integrations>('integrations')
            
            const roValra: RoValra = {
                joinServerForYouValue: false
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
    <p class="text-stone-300 text-base">Still working on this one. Comeback later when crush drop bangers!</p>
</div>