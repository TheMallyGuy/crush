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
                loginViaWebviewStatus =
                    'Could not find .ROBLOSECURITY cookie. Make sure you are logged in.'
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
                title: 'That was a total success!',
                description: `Hello ${data.displayName}! You can now use the account manager!`,
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
            loginViaWebviewStatus =
                'Failed to get cookie, make sure you are logged in and try again.'
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
            title: 'Validating accounts',

            description: "Hang tight, we're validating your accounts",
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
                    title: `${account.id} is invaild`,

                    description: `The account ${account.id} is invaild. We've removed it from the list.`,
                })
                popUser(account.id)
            }
        }
        notify.send({
            variant: 'info',
            title: 'Validate Complete',
            description: 'All invaild accounts (if present) have been removed',
        })
    }

    async function startQuickSignIn() {
        try {
            quickSignStatus = 'Creating code...'
            quickSignPolling = true

            const creation = await invoke<{
                code: string
                privateKey: string
                expirationTime: string
            }>('quick_sign_create')
            quickSignCode = creation.code
            quickSignStatus = 'Waiting for approval...'

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
                        ? 'Cancelled.'
                        : 'Timed out or failed.'
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
                title: 'Signed in!',
                description: `Hello ${data.displayName}! You can now use the account manager!`,
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
                title: 'Launch failed!',
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
        { label: 'Login via WebView (browser)', value: 'webView' },
        { label: 'Login via quick sign in', value: 'quickSignIn' },
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
    title="Confirm deletion"
    description="Are you sure you want to delete this account? This action cannot be undone."
>
    <div slot="actions">
        <Button
            variant="secondary"
            size="sm"
            on:click={() => resolveDelete?.(false)}
        >
            Cancel
        </Button>
        <Button
            variant="danger"
            size="sm"
            on:click={() => resolveDelete?.(true)}
        >
            Confirm
        </Button>
    </div>
</Dialog>

<Dialog
    bind:open={addDialog}
    on:close={() => (addDialog = false)}
    title="Add an account"
    description="Add a new account to the list"
>
    <Tabs {tabs} bind:activeTab>
        {#if activeTab === 'webView'}
            {#if openedWebview}
                <Button
                    variant="primary"
                    size="sm"
                    on:click={checkWebviewCookie}>Check cookie</Button
                >
            {:else if loginViaWebviewStatus}
                <p class="text-red-400">{loginViaWebviewStatus}</p>
                <Button
                    variant="primary"
                    size="sm"
                    class="flex w-max justify-between"
                    on:click={openWebview}>Open webview</Button
                >
            {:else}
                <Button
                    variant="primary"
                    size="sm"
                    class="flex w-max justify-between"
                    on:click={openWebview}>Open webview</Button
                >
            {/if}
        {:else if activeTab === 'quickSignIn'}
            <div class="flex flex-col gap-4">
                {#if quickSignCode}
                    <p class="text-stone-400">
                        Enter this code in the Roblox app under Quick Sign In:
                    </p>
                    <div class="flex items-center justify-center w-full">
                        <code
                            class="text-2xl font-bold tracking-tight text-stone-100"
                            >{quickSignCode}</code
                        >
                    </div>
                    <p class="text-sm text-stone-400">{quickSignStatus}</p>
                {:else}
                    <p class="text-stone-400">
                        {quickSignStatus || 'Sign in using quick sign in.'}
                    </p>
                    <Button
                        variant="primary"
                        size="sm"
                        disabled={quickSignPolling}
                        on:click={startQuickSignIn}
                    >
                        Start Quick Sign In
                    </Button>
                {/if}
            </div>
        {/if}
    </Tabs>
    <div slot="actions" class="flex items-center justify-end gap-2">
        <Checkbox bind:checked={vngAccount}
            >Mark as an VNG account (Do not check this if you dont know what
            you're doing)</Checkbox
        >
    </div>
</Dialog>

<div class="flex flex-col gap-5">
    <div class="flex items-center justify-between">
        <div>
            <h1 class="text-3xl font-bold tracking-tight text-stone-100">
                Account Management
            </h1>
            <p class="text-stone-400 mt-1">Swtich between accounts made easy</p>
        </div>
    </div>

    <div class="flex items-center justify-between">
        <Button variant="primary" size="md" on:click={() => (addDialog = true)}>
            <Plus class="size-4 mr-2" /> Add an account
        </Button>
        <Button variant="secondary" size="md" on:click={validateAccounts}>
            <RefreshCw class="size-4 mr-2" /> Validate accounts
        </Button>
    </div>

    <SortableList items={accountsData ?? []} let:item>
        <div class="flex items-center justify-between w-full pr-2">
            <div class="flex flex-col gap-0.5">
                <span class="text-sm font-medium text-stone-100">
                    {item.id}
                    <span
                        class="ml-2 text-xs {item.type === 'vng'
                            ? 'text-orange-400'
                            : 'text-blue-400'}"
                        >{item.type === 'vng' ? 'VNG' : 'Global'}</span
                    >
                </span>
                <span class="text-xs text-stone-400"
                    >{item.username} ({item.userId})</span
                >
            </div>
            <div class="flex items-center gap-1.5">
                <Button
                    variant="secondary"
                    size="sm"
                    on:click={() => useAccount(item.id)}>Use</Button
                >
                <Button
                    variant="danger"
                    size="sm"
                    on:click={() => deleteAccount(item.id)}>Delete</Button
                >
            </div>
        </div>
    </SortableList>
</div>
