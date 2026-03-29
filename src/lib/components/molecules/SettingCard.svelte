<script lang="ts">
    import type { Component } from 'svelte'

    export let title = ''
    export let description = ''
    export let icon: Component<any> | null = null
    export let clickable = false
    export let delay = 0

    let className = ''
    export { className as class }
</script>

<div
    class="group relative flex w-full flex-col rounded-card bg-anthracite p-5 transition-all duration-300 border border-white/5 animate-in fill-mode-forwards
    {clickable ? 'cursor-pointer hover:bg-anthracite-light hover:border-white/10 active:scale-[0.995]' : ''}
    {className}"
    style="animation-delay: {delay}ms; opacity: 0;"
>
    <div class="flex items-center justify-between gap-5">
        <div class="flex items-center gap-5">
            {#if icon || $$slots.icon}
                <div
                    class="flex h-12 w-12 shrink-0 items-center justify-center rounded-item bg-obsidian text-stone-400 group-hover:text-sapphire transition-colors border border-white/5"
                >
                    <slot name="icon">
                        <svelte:component this={icon} size={22} />
                    </slot>
                </div>
            {/if}

            <div class="flex flex-col gap-0.5">
                {#if title || $$slots.title}
                    <h3 class="text-base font-bold tracking-tight text-white group-hover:text-sapphire transition-colors">
                        <slot name="title">{title}</slot>
                    </h3>
                {/if}

                {#if description || $$slots.description}
                    <p class="text-sm font-medium text-stone-500 group-hover:text-stone-400 transition-colors">
                        <slot name="description">{description}</slot>
                    </p>
                {/if}
            </div>
        </div>

        <div class="flex items-center gap-3">
            <slot name="action" />
        </div>
    </div>

    {#if $$slots.footer}
        <div class="mt-6 pt-6 flex flex-col gap-4 border-t border-white/5">
            <slot name="footer" />
        </div>
    {/if}
</div>

<style>
    @keyframes in {
        from {
            opacity: 0;
            transform: translateY(8px);
        }
        to {
            opacity: 1;
            transform: translateY(0);
        }
    }
    .animate-in {
        animation: in 0.4s cubic-bezier(0.16, 1, 0.3, 1) forwards;
    }
</style>
