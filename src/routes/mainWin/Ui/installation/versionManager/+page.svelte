<script lang="ts">
    import Button from '$lib/components/atoms/Button.svelte'
    import Textbox from '$lib/components/atoms/Textbox.svelte'
    import Dialog from '$lib/components/molecules/Dialog.svelte'
    import SortableList from '$lib/components/molecules/SortableList.svelte'
    import type { AppType } from '$lib/types'
    import { Play, Plus, Trash2 } from '@lucide/svelte'
    import { load, Store } from '@tauri-apps/plugin-store'
    import { onMount } from 'svelte'
    import { _ } from 'svelte-i18n'

    type Versions = {
        appType: AppType
        id: string
        installedAt: string
        versionHash: string
    }

    type VersionsConfig = {
        currentlyUsing: Versions
        versions: Versions[]
    }

    let currentlyUsing: Versions | null = null
    let items: Versions[] = []

    let raw: Store

    let createDialog: boolean

    let resolveCreate: ((value: string | null) => void) | null = null
    let versionName: string
    let versionHash: string

    async function handleNewVersion() {
        createDialog = true

        const IsReslove = await new Promise<string | null>((resolve) => {
            resolveCreate = resolve
        })

        createDialog = false

        if (IsReslove) {
            let versions = await raw.get<Versions[]>('versions')

            versions?.push({
                appType: 'player',
                id: versionName,
                installedAt: new Date().toISOString(),
                versionHash: versionHash,
            })

            await raw.set('versions', versions)
            await raw.save()
            await loadConfig()
        }
    }

    let deleteDialog: boolean

    let resolveDelete: ((value: boolean | null) => void) | null = null
    let deleteId: string | null = null

    async function popVersion() {
        deleteDialog = true

        const IsReslove = await new Promise<boolean | null>((resolve) => {
            resolveDelete = resolve
        })

        if (IsReslove) {
            if (!deleteId) return

            let versions = await raw.get<Versions[]>('versions')

            const filtered = versions?.filter((v) => v.id !== deleteId) ?? []

            await raw.set('versions', filtered)
            await raw.save()

            if (currentlyUsing?.id === deleteId) {
                await raw.set('currentlyUsing', null)
                currentlyUsing = null
            }

            deleteDialog = false

            await loadConfig()
        }
    }

    async function makePrimary(id: string) {
        const versions = await raw.get<Versions[]>('versions')

        const item = versions?.find((v) => v.id === id)

        if (!item) return

        await raw.set('currentlyUsing', item)
        await raw.save()

        currentlyUsing = item
    }

    async function loadConfig() {
        raw = await load('versions.json')

        const versions = await raw.get<Versions[]>('versions')
        const current = await raw.get<Versions>('currentlyUsing')

        console.log('versions:', versions)
        console.log('currentlyUsing:', current)

        items = versions ?? []
        currentlyUsing = current ?? null
    }

    onMount(async () => {
        await loadConfig()
    })
</script>

<Dialog
    bind:open={deleteDialog}
    on:close={() => resolveDelete?.(false)}
    title={$_('pages.installations.versionManager.dialogs.delete.title')}
    description={$_(
        'pages.installations.versionManager.dialogs.delete.description'
    )}
>
    <div slot="actions">
        <Button
            variant="secondary"
            size="sm"
            on:click={() => resolveDelete?.(false)}
        >
            {$_('pages.installations.versionManager.dialogs.delete.cancel')}
        </Button>
        <Button
            variant="danger"
            size="sm"
            on:click={() => resolveDelete?.(true)}
        >
            {$_('pages.installations.versionManager.dialogs.delete.confirm')}
        </Button>
    </div>
</Dialog>

<Dialog
    bind:open={createDialog}
    on:close={() => resolveCreate?.(null)}
    title={$_("pages.installations.versionManager.dialogs.new.title")}
>
    <Textbox
        label={$_("pages.installations.versionManager.dialogs.new.textbox1.title")}
        placeholder={$_("pages.installations.versionManager.dialogs.new.textbox1.placeholder")}
        bind:value={versionName}
    />

    <Textbox
        label="Version hash"
        placeholder="version-ecoai3i1ojfoi1"
        bind:value={versionHash}
    />

    <div slot="actions">
        <Button
            variant="secondary"
            size="sm"
            on:click={() => resolveCreate?.(null)}
        >
            {$_("pages.installations.versionManager.dialogs.new.cancel")}
        </Button>
        <Button
            variant="primary"
            size="sm"
            on:click={() => resolveCreate?.(versionName)}
        >
            {$_("pages.installations.versionManager.dialogs.new.confirm")}
        </Button>
    </div>
</Dialog>

<div class="flex flex-col gap-8">
    <div class="flex items-center justify-between">
        <div>
            <h1 class="text-3xl font-bold tracking-tight text-stone-100">
                {$_('pages.installations.versionManager.title')}
            </h1>
            <p class="text-stone-400 mt-1">
                {$_('pages.installations.versionManager.description')}
            </p>
        </div>
    </div>

    <Button variant="primary" size="md" on:click={handleNewVersion}>
        <Plus class="size-4 mr-2" />
    </Button>

    <div class="space-y-4">
        {#if items.length === 0}
            <div
                class="p-8 border-2 border-dashed border-stone-800 rounded-xl text-center"
            >
                <p class="text-stone-500">
                    {$_('pages.installations.versionManager.none')}
                </p>
            </div>
        {:else}
            <SortableList {items} let:item>
                <div class="flex items-center justify-between w-full pr-2">
                    <div class="flex flex-col gap-0.5">
                        <span class="text-sm font-medium text-stone-100">
                            {item.id}
                            {#if currentlyUsing?.id === item.id}
                                <span class="ml-2 text-xs text-green-400"
                                    >(active)</span
                                >
                            {/if}
                        </span>
                        <span class="text-xs text-stone-400"
                            >{item.versionHash}</span
                        >
                    </div>
                    <div class="flex items-center gap-1.5">
                        <Button
                            size="sm"
                            variant="ghost"
                            on:click={() => {
                                makePrimary(item.id)
                            }}
                        >
                            <Play class="size-4" />
                        </Button>
                        <Button
                            size="sm"
                            variant="ghost"
                            on:click={() => {
                                deleteId = item.id
                                popVersion()
                            }}
                        >
                            <Trash2 class="size-4" />
                        </Button>
                    </div>
                </div>
            </SortableList>
        {/if}
    </div>
</div>
