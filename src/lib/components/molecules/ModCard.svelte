<script lang="ts">
    import { _ } from 'svelte-i18n'
    import { Download } from '@lucide/svelte'
    import Button from '$lib/components/atoms/Button.svelte'

    interface Props {
        name?: string
        author?: string
        category?: string
        thumbnail?: string
        downloading?: boolean
        adapter?: string
        class?: string
        onclick?: (event: MouseEvent) => void
        [key: string]: unknown
    }

    let {
        name = '',
        author = '',
        category = '',
        thumbnail = '',
        downloading = false,
        adapter = '',
        class: className = '',
        onclick,
        ...rest
    }: Props = $props()
</script>

<div
    class="group relative flex w-full flex-col overflow-hidden bg-anthracite border border-stone-800/30 transition-all duration-150 hover:border-stone-700/50 {className}"
    {...rest}
>
    <div class="relative h-44 w-full shrink-0 overflow-hidden bg-stone-900">
        {#if thumbnail}
            <img
                src={thumbnail}
                alt={name}
                class="h-full w-full object-cover transition-transform duration-500 group-hover:scale-105"
            />
        {:else}
            <div class="h-full w-full bg-stone-800/60"></div>
        {/if}
    </div>

    <div class="flex flex-col gap-3 p-3">
        <div class="flex flex-col gap-0.5">
            <p class="truncate text-sm font-bold text-stone-100">{name}</p>
            {#if author || category}
                <p class="truncate text-xs font-medium text-stone-400">
                    {#if author}{$_('pages.mod.tab.communityMods.itemCard.by', {
                            values: { name: author },
                        })}
                    {/if}{#if author && category}
                        |
                    {/if}{category}
                </p>
                <p class="truncate text-xs font-medium text-stone-400">
                    {$_('pages.mod.tab.communityMods.itemCard.adapter', {
                        values: { library: adapter },
                    })}
                </p>
            {/if}
        </div>

        <div class="flex items-center gap-2">
            <Button
                variant="primary"
                size="sm"
                disabled={downloading}
                class="flex-1 gap-1.5"
                {onclick}
            >
                <Download size={13} />
                {downloading ? '...' : 'Download'}
            </Button>
        </div>
    </div>
</div>
