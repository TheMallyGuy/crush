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
    title={$_('pages.settings.cloudSync.encryptionPasswordsCard.dialog.title')}
    description={$_(
        'pages.settings.cloudSync.encryptionPasswordsCard.dialog.description'
    )}
>
    <div class="flex flex-col gap-4">
        <Textbox
            type="password"
            placeholder={$_(
                'pages.settings.cloudSync.encryptionPasswordsCard.dialog.placeholder'
            )}
            bind:value={passwordInput}
            onenter={savePassword}
        />
        <div class="flex justify-end gap-2">
            <Button variant="secondary" onclick={() => (passwordDialog = false)}
                >{$_(
                    'pages.settings.cloudSync.encryptionPasswordsCard.dialog.cancel'
                )}</Button
            >
            <Button onclick={savePassword} disabled={!passwordInput}>
                {$_(
                    'pages.settings.cloudSync.encryptionPasswordsCard.dialog.save'
                )}
            </Button>
        </div>
    </div>
</Dialog>

<div class="relative flex flex-col gap-8 min-h-screen">
    <LoadingOverlay
        visible={cloud.isInAuth}
        blur={false}
        message={$_('pages.settings.cloudSync.spinner')}
    />

    <div class="flex items-center justify-between">
        <div>
            <h1 class="text-3xl font-bold tracking-tight text-stone-100">
                {$_('pages.settings.cloudSync.cloudSync')}
            </h1>
            <p class="text-stone-400 mt-1">
                {$_('pages.settings.cloudSync.description')}
            </p>
        </div>
        <div class="flex items-center gap-2">
            <Button variant="secondary" onclick={() => goto('../settings')}>
                {$_('pages.settings.cloudSync.back')}
            </Button>
        </div>
    </div>

    {#if !cloud.isLoggedIn}
        <div
            class="flex flex-col gap-3 items-center justify-center py-32 text-stone-400 border border-dashed border-stone-800 rounded-lg"
        >
            {$_('pages.settings.cloudSync.notLoggedIn')}
            {#if cloud.authError}
                <p class="text-red-400 text-sm">{cloud.authError}</p>
            {/if}
            <Button onclick={() => cloud.login()} disabled={cloud.isInAuth}>
                {$_('pages.settings.cloudSync.loginWithBroswer')}
            </Button>
        </div>
    {:else}
        <SettingCard
            title={$_(
                'pages.settings.cloudSync.encryptionPasswordsCard.changePasswords'
            )}
            description={cloud.hasPassword
                ? $_(
                      'pages.settings.cloudSync.encryptionPasswordsCard.descriptionAlt'
                  )
                : $_(
                      'pages.settings.cloudSync.encryptionPasswordsCard.description'
                  )}
        >
            {#snippet action()}
                <Button variant="secondary" onclick={openPasswordDialog}>
                    {cloud.hasPassword
                        ? $_(
                              'pages.settings.cloudSync.encryptionPasswordsCard.changePasswords'
                          )
                        : $_(
                              'pages.settings.cloudSync.encryptionPasswordsCard.setPasswords'
                          )}
                </Button>
            {/snippet}
        </SettingCard>

        {#if !cloud.hasPassword}
            <p class="text-amber-400/90 text-sm -mt-2">
                {$_(
                    'pages.settings.cloudSync.encryptionPasswordsCard.noEncryption'
                )}
            </p>
        {/if}

        <SettingCard
            title={$_('pages.settings.cloudSync.syncFromCloud.title')}
            description={$_(
                'pages.settings.cloudSync.syncFromCloud.description'
            )}
        >
            {#snippet action()}
                <Button
                    onclick={() => cloud.syncFromCloud()}
                    disabled={cloud.isSyncing || !cloud.hasPassword}
                    >{$_(
                        'pages.settings.cloudSync.syncFromCloud.button'
                    )}</Button
                >
            {/snippet}
        </SettingCard>

        <SettingCard
            title={$_('pages.settings.cloudSync.syncToCloud.title')}
            description={$_('pages.settings.cloudSync.syncToCloud.description')}
        >
            {#snippet action()}
                <Button
                    variant="danger"
                    onclick={() => cloud.syncToCloud()}
                    disabled={cloud.isSyncing || !cloud.hasPassword}
                    >{$_('pages.settings.cloudSync.syncToCloud.button')}</Button
                >
            {/snippet}
        </SettingCard>

        <SettingCard
            title={$_('pages.settings.cloudSync.autoSync.title')}
            description={$_('pages.settings.cloudSync.autoSync.description')}
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
