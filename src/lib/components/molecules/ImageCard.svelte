<script lang="ts">
    import type { Snippet } from 'svelte'

    interface Props {
        title?: string
        description?: string
        image?: string
        disabled?: boolean
        class?: string
        children?: Snippet
        [key: string]: unknown
    }

    let {
        title = '',
        description = '',
        image = '',
        disabled = false,
        class: className = '',
        children,
        ...rest
    }: Props = $props()
</script>

<div
    class="group relative flex w-full flex-row overflow-hidden transition-all duration-150
    {!disabled
        ? 'hover:bg-stone-900/50 hover:border-stone-700/40'
        : 'opacity-50'}
    {className}"
    {...rest}
>
    {#if image}
        <div
            class="relative w-48 shrink-0 overflow-hidden border-r border-stone-800/20"
        >
            <img
                src={image}
                alt={title}
                class="h-full w-full object-cover transition-transform duration-500 group-hover:scale-105"
            />
            <div
                class="absolute inset-0 bg-linear-to-l from-black/40 to-black/40"
            ></div>
        </div>
    {/if}

    <div class="flex flex-1 flex-col gap-4 p-5">
        <div class="flex flex-col gap-1.5">
            {#if title}
                <h3
                    class="text-lg font-bold tracking-tight text-stone-100 transition-colors duration-150 group-hover:text-white"
                >
                    {title}
                </h3>
            {/if}

            {#if description}
                <p
                    class="text-sm font-medium leading-relaxed text-stone-300 transition-colors duration-150 group-hover:text-stone-200"
                >
                    {description}
                </p>
            {/if}
        </div>

        <div class="mt-auto flex flex-wrap items-center gap-3 pt-2">
            {@render children?.()}
        </div>
    </div>
</div>
