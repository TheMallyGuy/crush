<script lang="ts">
    import type { Snippet } from 'svelte'
    import { fly, fade } from 'svelte/transition'
    import { CircleCheck, CircleX, AlertTriangle, InfoIcon, Bell, X } from '@lucide/svelte'

    interface Props {
        variant?: 'info' | 'success' | 'warning' | 'danger' | 'default'
        title?: string
        description?: string
        dismissible?: boolean
        class?: string
        ondismiss?: () => void
        children?: Snippet
        footer?: Snippet
    }

    let {
        variant = 'info',
        title = '',
        description = '',
        dismissible = true,
        class: className = '',
        ondismiss,
        children,
        footer,
    }: Props = $props()

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

    let CurrentIcon = $derived(icons[variant])
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
        <CurrentIcon size={16} />
    </div>

    <div class="flex min-w-0 flex-1 flex-col gap-0.5">
        {#if title}
            <p class="text-sm font-semibold text-stone-100 leading-snug">
                {title}
            </p>
        {/if}
        {#if description}
            <p class="text-sm text-stone-400 leading-snug">
                {description}
            </p>
        {/if}
        {#if children}
            <div class="mt-2.5 flex gap-2">
                {@render children()}
            </div>
        {/if}
        {#if footer}
            <p class="mt-1 text-xs text-stone-500">
                {@render footer()}
            </p>
        {/if}
    </div>

    {#if dismissible}
        <button
            type="button"
            class="shrink-0 p-1 text-stone-500 transition-colors hover:bg-stone-800/50 hover:text-stone-200 focus:outline-none focus:ring-2 focus:ring-sapphire/50"
            aria-label="Dismiss"
            onclick={() => ondismiss?.()}
        >
            <X size={15} />
        </button>
    {/if}
</div>
