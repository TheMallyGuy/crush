<script lang="ts">
    import Button from '$lib/components/atoms/Button.svelte'
    import Textbox from '$lib/components/atoms/Textbox.svelte'
    import type { BoostrapConfigs, Installation, Integrations, interactiveAPI, RoValra } from '$lib/types'
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
            
            const windowManipulation: interactiveAPI = {
                enable: boostraperConfig.UseWindowControl ?? false,
                scopes: {
                    setTitle : boostraperConfig.TitleControlAllowed ?? true, // what the FUCK am i doing
                    focus: boostraperConfig.MoveWindowAllowed ?? true,
                    moveWindow: boostraperConfig.MoveWindowAllowed ?? true,
                    maximize: boostraperConfig.MoveWindowAllowed ?? true,
                    minimize: boostraperConfig.MoveWindowAllowed ?? true,
                    restore: boostraperConfig.MoveWindowAllowed ?? true,
                    setBorderless: boostraperConfig.MoveWindowAllowed ?? true
                }
            }
            
            const newIntegrations: Integrations = {
                sleepSchedule: {
                    enabled:  integrations?.sleepSchedule?.enabled ?? true,
                    visible: integrations?.sleepSchedule?.visible ?? false,
                },
                optimizer: integrations?.optimizer ?? false,
                closeCrashHandler: boostraperConfig.AutoCloseCrashHandler ?? boostraperConfig.DisableCrash ?? false,
                priority : integrations?.priority ?? "NORMAL_PRIORITY_CLASS",
                interactive: windowManipulation,
                activityWatching: boostraperConfig.EnableActivityTracking,
                discordRpc: integrations?.discordRpc ?? {
                    enable: boostraperConfig.UseDiscordRichPresence ?? false,
                    displayAccount: boostraperConfig.ShowAccountOnRichPresence ?? false,
                    letJoin: !(boostraperConfig.HideRPCButtons ?? false),
                },
                serverLocationNotifier:
                    boostraperConfig.ShowServerDetails ??
                    integrations?.serverLocationNotifier ??
                    false,
                roValra: roValra,
                gameCache: integrations?.gameCache ?? {},
                crushRpc:
                    boostraperConfig.UseDiscordRichPresence ??
                    integrations?.crushRpc ??
                    false,
            }
            
            const newInstallation: Installation = {
                vng: false, // currently no boostraper has this so just default to false
                version: "latest",
                forceReinstall: false,
                dontUpdate: boostraperConfig.UpdateRoblox ?? false,
                parallel: 4
            }

            config.set('discordRpcEnabled', boostraperConfig.ShowUsingFroststrapRPC ?? boostraperConfig.VoidRPC ?? true)

            await config.set('integrations', newIntegrations)
            await config.set('installation', newInstallation)
            await config.save()
        } catch (e) {
            console.error("Failed to import configs:", e)
        }
    }
</script>

<div class="flex flex-col h-full">
    <div class="flex flex-col gap-2 flex-1 overflow-y-auto min-h-0">
        <p class="text-stone-300 text-base">Export other boostraper configs that based on Bloxstrap to crush. (Tested : Bloxstrap, Frostrap, Voidstrap, Funkstrap)</p>
        <Textbox placeholder="C:\Users\Mally\AppData\Local\Bloxstrap" bind:value={userBasePath}/>
        <Button on:click={() => importConfigs(userBasePath)}>
            Export
        </Button>
        <div class="overflow-hidden border border-stone-800 max-h-[40vh]">
            <video class="w-full h-full object-cover" autoplay muted loop>
                <source src="/export.webm" type="video/webm"/>
            </video>
        </div>
    </div>
</div>