<script lang="ts">
    import Button from '$lib/components/atoms/Button.svelte'
    import Checkbox from '$lib/components/atoms/Checkbox.svelte'
    import Dialog from '$lib/components/molecules/Dialog.svelte'
    import SortableList from '$lib/components/molecules/SortableList.svelte'
    import Tabs from '$lib/components/molecules/Tabs.svelte'
    import { notify } from '$lib/notify'
    import { deepLinkUrl } from '$lib/stores/deeplink'
    import { Plus, RefreshCw } from '@lucide/svelte'
    import { invoke } from '@tauri-apps/api/core'
    import { WebviewWindow } from '@tauri-apps/api/webviewWindow'
    import { getCurrentWindow } from '@tauri-apps/api/window'
    import { error, info } from '@tauri-apps/plugin-log'
    import { load } from '@tauri-apps/plugin-store'
    import { onMount } from 'svelte'
    import { _ } from 'svelte-i18n'

    let addDialog: boolean = $state(false)
    let activeTab = $state('browser')
    let openedWebview = $state(false)
    let vngAccount = $state(false)

    let deleteDialog: boolean = $state(false)
    let resolveDelete: ((value: boolean) => void) | null = null

    let quickSignCode = $state<string | null>(null)
    let quickSignStatus = $state('')
    let quickSignPolling = $state(false)

    let loginViaWebviewStatus = $state('')

    let accountsData = $state<Account[]>([])

    async function deleteAccount(id: string) {
        deleteDialog = true

        const confirmed = await new Promise<boolean>((resolve) => {
            resolveDelete = (value: boolean) => resolve(value)
        })

        deleteDialog = false

        if (confirmed) {
            accountsData = accountsData.filter((account) => account.id !== id)

            // save to disk
            const store = await load('accountData.json')
            store.set('accounts', accountsData)
            await store.save()

            info(`deleted (${id})`)
        }
    }

    type Account = {
        id: string
        username: string
        userId: number
        Cookie: string
        type: 'vng' | 'global'
    }

    function openWebview() {
        info('user requested webview login')
        openedWebview = true
        invoke('clear_cookies')
        invoke('create_or_focus_window', {
            label: 'loginToRoblox',
            title: 'Login to roblox',
            url: 'https://roblox.com/login',
            width: 400,
            height: 600,
            decorations: true,
        })
    }

    async function checkWebviewCookie() {
        try {
            info('checking cookies')
            const rawCookie = await invoke<string>('export_all_cookies')

            const cookieValue = extractRoblosecurity(rawCookie)

            if (!cookieValue) {
                // what the fuck is this shit??????? do i write bullshit when im high
                loginViaWebviewStatus = $_(
                    'pages.accountManagement.dialogs.add.tabs.webview.failed'
                )
                return
            }

            const window = await WebviewWindow.getByLabel('loginToRoblox')
            if (window) {
                await window.close()
                openedWebview = false
            }

            const cookie = `.ROBLOSECURITY=${cookieValue}`

            const data = await invoke<{
                id: number
                displayName: string
                name: string
            }>('validate_roblox_cookie', { cookie })

            notify.send({
                variant: 'success',
                title: $_(
                    'pages.accountManagement.notifiers.addAccountSuccess.title'
                ),
                description: $_(
                    'pages.accountManagement.notifiers.addAccountSuccess.description',
                    { values: { username: data.id } }
                ),
                duration: 5000,
            })

            addDialog = false
            await saveNewUser(
                cookie,
                data.name,
                data.id,
                vngAccount ? 'vng' : 'global',
                data.displayName
            )
        } catch (err) {
            openedWebview = false
            loginViaWebviewStatus = $_(
                'pages.accountManagement.dialogs.add.tabs.webview.failed'
            )
            error(`Error checking cookie: ${err}`)
        }
    }

    async function saveNewUser(
        cookie: string,
        username: string,
        userId: number,
        type: 'vng' | 'global',
        name: string
    ) {
        const encryptedCookie: string = await invoke('encrypt_cookie_data', {
            decrypted: cookie,
        })

        const newAccount: Account = {
            id: name,
            username,
            userId,
            Cookie: encryptedCookie,
            type,
        }

        accountsData = accountsData
            .filter((account) => account.id !== name)
            .concat(newAccount)

        // save to disk
        const store = await load('accountData.json')
        store.set('accounts', accountsData)
        await store.save()

        info(`saved (${name})`)
    }

    async function popUser(id: string) {
        const account = accountsData.find((account) => account.id === id)
        if (!account) {
            error(`account with id ${id} not found!`)
            return
        }

        const store = await load('accountData.json')
        store.set(
            'accounts',
            accountsData.filter((a) => a.id !== id)
        )
        await store.save()
        info(`popped (${id})`)
    }

    function extractRoblosecurity(raw: string): string | null {
        for (const line of raw.split('\n')) {
            const trimmed = line.replace(/^#HttpOnly_/, '')
            const parts = trimmed.split('\t')
            if (
                parts.length >= 7 &&
                (parts[5] === '.ROBLOSECURITY' || parts[5] === 'rbxas')
            ) {
                return parts[6]
            }
        }
        return null
    }

    async function validateAccounts() {
        notify.send({
            variant: 'info',
            title: $_(
                'pages.accountManagement.notifiers.validateAccounts.title'
            ),

            description: $_(
                'pages.accountManagement.notifiers.validateAccounts.description'
            ),
        })

        for (const account of accountsData) {
            try {
                const cookie = await getDecryptedCookie(account)
                const data = await invoke<{
                    id: number
                    displayName: string
                    name: string
                }>('validate_roblox_cookie', { cookie })
                info(
                    `account ${account.id} is valid, authenticated as ${data.displayName} (${data.id})!`
                )
            } catch (err) {
                error(`account ${account.id} is invalid! error: ${err}`)
                notify.send({
                    variant: 'danger',
                    title: $_(
                        'pages.accountManagement.notifiers.validateAccountsInvaild.title',
                        { values: { account: account.id } }
                    ),

                    description: $_(
                        'pages.accountManagement.notifiers.validateAccountsInvaild.description',
                        { values: { account: account.id } }
                    ),
                })
                popUser(account.id)
            }
        }
        notify.send({
            variant: 'info',
            title: $_(
                'pages.accountManagement.notifiers.validateAccountsSucess.title'
            ),
            description: $_(
                'pages.accountManagement.notifiers.validateAccountsSucess.description'
            ),
        })
    }

    async function startQuickSignIn() {
        try {
            quickSignStatus = $_(
                'pages.accountManagement.dialogs.add.tabs.quickSignIn.creating'
            )
            quickSignPolling = true

            const creation = await invoke<{
                code: string
                privateKey: string
                expirationTime: string
            }>('quick_sign_create')
            quickSignCode = creation.code
            quickSignStatus = $_(
                'pages.accountManagement.dialogs.add.tabs.quickSignIn.approval'
            )

            // poll
            const result = await invoke<{
                status: string
                cookie: string | null
            }>('quick_sign_poll', {
                code: creation.code,
                privateKey: creation.privateKey,
            })

            if (result.status !== 'Validated' || !result.cookie) {
                quickSignStatus =
                    result.status === 'Cancelled'
                        ? $_(
                              'pages.accountManagement.dialogs.add.tabs.quickSignIn.cancelled'
                          )
                        : $_(
                              'pages.accountManagement.dialogs.add.tabs.quickSignIn.timeout'
                          )
                quickSignPolling = false
                quickSignCode = null
                return
            }

            const cookie = `.ROBLOSECURITY=${result.cookie}`
            const data = await invoke<{
                id: number
                displayName: string
                name: string
            }>('validate_roblox_cookie', { cookie })

            notify.send({
                variant: 'success',
                title: $_(
                    'pages.accountManagement.notifiers.addAccountSuccess.title'
                ),
                description: $_(
                    'pages.accountManagement.notifiers.addAccountSuccess.description',
                    { values: { username: data.displayName } }
                ),
                duration: 5000,
            })
            addDialog = false
            quickSignCode = null
            quickSignPolling = false
            quickSignStatus = ''

            await saveNewUser(
                cookie,
                data.name,
                data.id,
                vngAccount ? 'vng' : 'global',
                data.displayName
            )
        } catch (err) {
            quickSignStatus = `Failed: ${err}`
            quickSignPolling = false
            quickSignCode = null
            error(`quick sign in error: ${err}`)
        }
    }

    async function playGame(deeplink: string) {
        const win = getCurrentWindow()

        await deepLinkUrl.set(deeplink)

        await invoke('create_or_focus_window', {
            label: 'CrushBoostrap',
            url: 'boostrapWin',
            title: 'Crush',
            width: 500.0,
            height: 350.0,
            minWidth: 500,
            minHeight: 350.0,
        })

        setTimeout(() => {
            // wait before killing to prevent crash
            getCurrentWindow().close()
        }, 100)
    }

    async function useAccount(id: string) {
        const account = accountsData.find((account) => account.id === id)
        if (!account) return

        try {
            const cookie = await getDecryptedCookie(account)
            const csrf = await invoke<string>('get_csrf_token', { cookie })
            const ticket = await invoke<string>('get_auth_ticket', {
                cookie,
                csrf,
                placeId: 0,
            })

            const browserTrackerId = Math.floor(
                Math.random() * (2147483647 - 1000000000) + 1000000000
            ).toString()
            const launchUri = `roblox-player:1+launchmode:play+gameinfo:${ticket}+browsertrackerid:${browserTrackerId}+robloxLocale:en_us+gameLocale:en_us+channel:`

            await playGame(launchUri)
            notify.send({
                variant: 'success',
                title: 'Launched!',
                description: `Launching as ${account.username}`,
                duration: 5000,
            })
        } catch (err) {
            error(`launch failed: ${err}`)
            notify.send({
                variant: 'danger',
                title: $_(
                    'pages.accountManagement.notifiers.launchError.title'
                ),
                description: `${err}`,
                duration: 5000,
            })
        }
    }

    async function getDecryptedCookie(account: Account): Promise<string> {
        const decrypted = await invoke<string>('decrypt_cookie_data', {
            encrypted: account.Cookie,
        })
        return decrypted.startsWith('.ROBLOSECURITY=')
            ? decrypted
            : `.ROBLOSECURITY=${decrypted}`
    }

    let tabs = $state([
        {
            label: $_('pages.accountManagement.dialogs.add.tabs.webview.tab'),
            value: 'webView',
        },
        {
            label: $_(
                'pages.accountManagement.dialogs.add.tabs.quickSignIn.tab'
            ),
            value: 'quickSignIn',
        },
    ])

    onMount(async () => {
        const store = await load('accountData.json')
        const saved = await store.get<typeof accountsData>('accounts')
        if (saved) accountsData = saved
    })
</script>

<Dialog
    bind:open={deleteDialog}
    on:close={() => resolveDelete?.(false)}
    title={$_('pages.accountManagement.dialogs.delete.title')}
    description={$_('pages.accountManagement.dialogs.delete.description')}
>
    <div slot="actions">
        <Button
            variant="secondary"
            size="sm"
            on:click={() => resolveDelete?.(false)}
        >
            {$_('pages.accountManagement.dialogs.delete.cancel')}
        </Button>
        <Button
            variant="danger"
            size="sm"
            on:click={() => resolveDelete?.(true)}
        >
            {$_('pages.accountManagement.dialogs.delete.confirm')}
        </Button>
    </div>
</Dialog>

<Dialog
    bind:open={addDialog}
    on:close={() => (addDialog = false)}
    title={$_('pages.accountManagement.dialogs.add.title')}
    description={$_('pages.accountManagement.dialogs.add.description')}
>
    <Tabs {tabs} bind:activeTab>
        {#if activeTab === 'webView'}
            {#if openedWebview}
                <Button
                    variant="primary"
                    size="sm"
                    on:click={checkWebviewCookie}
                    >{$_(
                        'pages.accountManagement.dialogs.add.tabs.webview.check'
                    )}</Button
                >
            {:else if loginViaWebviewStatus}
                <p class="text-red-400">{loginViaWebviewStatus}</p>
                <Button
                    variant="primary"
                    size="sm"
                    class="flex w-max justify-between"
                    on:click={openWebview}
                    >{$_(
                        'pages.accountManagement.dialogs.add.tabs.webview.open'
                    )}</Button
                >
            {:else}
                <Button
                    variant="primary"
                    size="sm"
                    class="flex w-max justify-between"
                    on:click={openWebview}
                    >{$_(
                        'pages.accountManagement.dialogs.add.tabs.webview.open'
                    )}</Button
                >
            {/if}
        {:else if activeTab === 'quickSignIn'}
            <div class="flex flex-col gap-4">
                {#if quickSignCode}
                    <p class="text-muted-foreground">
                        {$_(
                            'pages.accountManagement.dialogs.add.tabs.quickSignIn.title'
                        )}
                    </p>
                    <div class="flex items-center justify-center w-full">
                        <code
                            class="text-2xl font-bold tracking-tight text-foreground"
                            >{quickSignCode}</code
                        >
                    </div>
                    <p class="text-sm text-muted-foreground">{quickSignStatus}</p>
                {:else}
                    <p class="text-muted-foreground">
                        {quickSignStatus ||
                            $_(
                                'pages.accountManagement.dialogs.add.tabs.quickSignIn.title'
                            )}
                    </p>
                    <Button
                        variant="primary"
                        size="sm"
                        disabled={quickSignPolling}
                        on:click={startQuickSignIn}
                    >
                        {$_(
                            'pages.accountManagement.dialogs.add.tabs.quickSignIn.start'
                        )}
                    </Button>
                {/if}
            </div>
        {/if}
    </Tabs>
    <div slot="actions" class="flex items-center justify-end gap-2">
        <Checkbox bind:checked={vngAccount}
            >{$_(
                'pages.accountManagement.dialogs.add.tabs.quickSignIn.checkbox'
            )}</Checkbox
        >
    </div>
</Dialog>

<div class="flex flex-col gap-5">
    <div class="flex items-center justify-between">
        <div>
            <h1 class="text-3xl font-bold tracking-tight text-foreground">
                {$_('pages.accountManagement.accountManagement')}
            </h1>
            <p class="text-muted-foreground mt-1">
                {$_('pages.accountManagement.description')}
            </p>
        </div>
    </div>

    <div class="flex items-center justify-between">
        <Button variant="primary" size="md" on:click={() => (addDialog = true)}>
            <Plus class="size-4 mr-2" />
            {$_('pages.accountManagement.addAnAccount')}
        </Button>
        <Button variant="secondary" size="md" on:click={validateAccounts}>
            <RefreshCw class="size-4 mr-2" />
            {$_('pages.accountManagement.validateAccounts')}
        </Button>
    </div>

    <SortableList items={accountsData ?? []} let:item>
        <div class="flex items-center justify-between w-full pr-2">
            <div class="flex flex-col gap-0.5">
                <span class="text-sm font-medium text-foreground">
                    {item.id}
                    <span
                        class="ml-2 text-xs {item.type === 'vng'
                            ? 'text-orange-400'
                            : 'text-blue-400'}"
                        >{item.type === 'vng' ? 'VNG' : 'Global'}</span
                    >
                </span>
                <span class="text-xs text-muted-foreground"
                    >{item.username} ({item.userId})</span
                >
            </div>
            <div class="flex items-center gap-1.5">
                <Button
                    variant="secondary"
                    size="sm"
                    on:click={() => useAccount(item.id)}>{$_("pages.accountManagement.use")}</Button
                >
                <Button
                    variant="danger"
                    size="sm"
                    on:click={() => deleteAccount(item.id)}>{$_("pages.accountManagement.delete")}</Button
                >
            </div>
        </div>
    </SortableList>
</div>
