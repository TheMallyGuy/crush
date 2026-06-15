<script lang="ts">
    import { goto } from '$app/navigation'
    import Button from '$lib/components/atoms/Button.svelte'
    import LoadingOverlay from '$lib/components/atoms/LoadingOverlay.svelte'
    import SettingCard from '$lib/components/molecules/SettingCard.svelte'
    import Switch from '$lib/components/atoms/Switch.svelte'
    import Textbox from '$lib/components/atoms/Textbox.svelte'
    import Dialog from '$lib/components/molecules/Dialog.svelte'
    import { onMount } from 'svelte'
    import { _ } from 'svelte-i18n'
    import { cloud } from '$lib/stores/cloudSync.svelte'

    let passwordDialog = $state(false)
    let passwordInput = $state('')

    function openPasswordDialog() {
        passwordInput = ''
        passwordDialog = true
    }

    async function savePassword() {
        if (!passwordInput) return
        await cloud.setPassword(passwordInput)
        passwordInput = ''
        passwordDialog = false
    }

    onMount(() => {
        cloud.init()
    })
</script>

<Dialog
    open={passwordDialog}
    onclose={() => (passwordDialog = false)}
    title="Encryption password"
    description="Your config is encrypted with this password before it's uploaded. It never leaves this device. If you lose it, your synced config can't be recovered."
>
    <div class="flex flex-col gap-4">
        <Textbox
            type="password"
            placeholder="Enter a password"
            bind:value={passwordInput}
            onenter={savePassword}
        />
        <div class="flex justify-end gap-2">
            <Button
                variant="secondary"
                onclick={() => (passwordDialog = false)}>Cancel</Button
            >
            <Button onclick={savePassword} disabled={!passwordInput}>
                Save
            </Button>
        </div>
    </div>
</Dialog>

<div class="relative flex flex-col gap-8 min-h-screen">
    <LoadingOverlay
        visible={cloud.isInAuth}
        blur={false}
        message="Continue in your broswer"
    />

    <div class="flex items-center justify-between">
        <div>
            <h1 class="text-3xl font-bold tracking-tight text-stone-100">
                Cloud config
            </h1>
            <p class="text-stone-400 mt-1">
                Sync your config to the cloud, sync to your devices with ease
            </p>
        </div>
        <div class="flex items-center gap-2">
            <Button variant="secondary" onclick={() => goto('../settings')}>
                Back
            </Button>
        </div>
    </div>

    {#if !cloud.isLoggedIn}
        <div
            class="flex flex-col gap-3 items-center justify-center py-32 text-stone-400 border border-dashed border-stone-800 rounded-lg"
        >
            You're not logged in.
            {#if cloud.authError}
                <p class="text-red-400 text-sm">{cloud.authError}</p>
            {/if}
            <Button onclick={() => cloud.login()} disabled={cloud.isInAuth}>
                Login in broswer
            </Button>
        </div>
    {:else}
        <SettingCard
            title="Encryption password"
            description={cloud.hasPassword
                ? 'Your config is end-to-end encrypted. The password stays on this device.'
                : 'Set a password to end-to-end encrypt your config before it syncs.'}
        >
            {#snippet action()}
                <Button variant="secondary" onclick={openPasswordDialog}>
                    {cloud.hasPassword ? 'Change' : 'Set password'}
                </Button>
            {/snippet}
        </SettingCard>

        {#if !cloud.hasPassword}
            <p class="text-amber-400/90 text-sm -mt-2">
                Set an encryption password to enable syncing.
            </p>
        {/if}

        <SettingCard
            title="Sync from cloud"
            description="Sync your config from the cloud"
        >
            {#snippet action()}
                <Button
                    onclick={() => cloud.syncFromCloud()}
                    disabled={cloud.isSyncing || !cloud.hasPassword}
                    >Sync</Button
                >
            {/snippet}
        </SettingCard>

        <SettingCard
            title="Sync to cloud"
            description="Sync your config to the cloud"
        >
            {#snippet action()}
                <Button
                    variant="danger"
                    onclick={() => cloud.syncToCloud()}
                    disabled={cloud.isSyncing || !cloud.hasPassword}
                    >Sync</Button
                >
            {/snippet}
        </SettingCard>

        <SettingCard
            title="Auto sync"
            description="Automatically sync your config to your device"
        >
            {#snippet action()}
                <Switch
                    checked={cloud.autoSync}
                    onchange={(checked: boolean) => cloud.setAutoSync(checked)}
                />
            {/snippet}
        </SettingCard>
    {/if}
</div>
