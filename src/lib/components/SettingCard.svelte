<script lang="ts">
    import type { ComponentType } from 'svelte'

    export let title = ''
    export let description = ''
    export let icon: ComponentType | null = null
    export let clickable = false

    let className = ''
    export { className as class }
</script>

<div
    class="group relative flex w-full flex-col rounded-xl bg-[#0f0f0f] p-5 transition-all duration-200
    {clickable ? 'cursor-pointer hover:bg-[#161616] active:scale-[0.995]' : ''} 
    {className}"
>
    <div class="flex items-center justify-between gap-5">
        <div class="flex items-center gap-5">
            {#if icon || $$slots.icon}
                <div
                    class="flex h-10 w-10 shrink-0 items-center justify-center text-gray-400"
                >
                    <slot name="icon">
                        <svelte:component this={icon} size={24} />
                    </slot>
                </div>
            {/if}

            <div class="flex flex-col gap-0.5">
                {#if title || $$slots.title}
                    <h3 class="text-base tracking-tight text-white">
                        <slot name="title">{title}</slot>
                    </h3>
                {/if}

                {#if description || $$slots.description}
                    <p class="text-sm font-medium text-gray-500">
                        <slot name="description">{description}</slot>
                    </p>
                {/if}
            </div>
        </div>

        <slot name="action" />
    </div>

    {#if $$slots.footer}
        <div class="mt-5 flex flex-col gap-4">
            <slot name="footer" />
        </div>
    {/if}
</div>
