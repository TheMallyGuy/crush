<script lang="ts">
    import { createEventDispatcher } from 'svelte'
    import { fly, fade } from 'svelte/transition'
    import { CircleCheck, CircleX, AlertTriangle, InfoIcon, Bell, X } from '@lucide/svelte'

    export let variant: 'info' | 'success' | 'warning' | 'danger' | 'default' = 'info'
    export let title: string = ''
    export let description: string = ''
    export let dismissible: boolean = true

    let className = ''
    export { className as class }

    const dispatch = createEventDispatcher()

    const icons = {
        info:    InfoIcon,
        success: CircleCheck,
        warning: AlertTriangle,
        danger:  CircleX,
        default: Bell,
    }

    const styles = {
        info:    { wrap: 'border-blue-500/20 bg-blue-500/5',   icon: 'bg-blue-500/10 text-blue-400' },
        success: { wrap: 'border-green-500/20 bg-green-500/5', icon: 'bg-green-500/10 text-green-400' },
        warning: { wrap: 'border-amber-500/20 bg-amber-500/5', icon: 'bg-amber-500/10 text-amber-400' },
        danger:  { wrap: 'border-red-500/20 bg-red-500/5',     icon: 'bg-red-500/10 text-red-400' },
        default: { wrap: 'border-stone-700/40 bg-stone-900/30', icon: 'bg-stone-800/60 text-stone-400' },
    }
</script>

<div
    in:fly={{ y: -6, duration: 200 }}
    out:fade={{ duration: 150 }}
    class="flex items-start gap-3 border p-3.5 transition-all
        {styles[variant].wrap} {className}"
    role="alert"
    aria-live="polite"
>
    <div class="flex h-8 w-8 shrink-0 items-center justify-center mt-0.5
        {styles[variant].icon}">
        <svelte:component this={icons[variant]} size={16} />
    </div>

    <div class="flex min-w-0 flex-1 flex-col gap-0.5">
        {#if title || $$slots.title}
            <p class="text-sm font-semibold text-stone-100 leading-snug">
                <slot name="title">{title}</slot>
            </p>
        {/if}
        {#if description || $$slots.description}
            <p class="text-sm text-stone-400 leading-snug">
                <slot name="description">{description}</slot>
            </p>
        {/if}
        {#if $$slots.default}
            <div class="mt-2.5 flex gap-2">
                <slot />
            </div>
        {/if}
        {#if $$slots.footer}
            <p class="mt-1 text-xs text-stone-500">
                <slot name="footer" />
            </p>
        {/if}
    </div>

    {#if dismissible}
        <button
            type="button"
            class="shrink-0 p-1 text-stone-500 transition-colors hover:bg-stone-800/50 hover:text-stone-200 focus:outline-none focus:ring-2 focus:ring-sapphire/50"
            aria-label="Dismiss"
            on:click={() => dispatch('dismiss')}
        >
            <X size={15} />
        </button>
    {/if}
</div>