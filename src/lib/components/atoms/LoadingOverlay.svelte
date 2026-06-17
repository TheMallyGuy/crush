<script lang="ts">
    import { fade } from 'svelte/transition'
    import Text from '../molecules/Text.svelte'

    interface Props {
        visible?: boolean
        message?: string
        blur?: boolean
        class?: string
        [key: string]: unknown
    }

    let {
        visible = true,
        message = '',
        blur = true,
        class: className = '',
        ...rest
    }: Props = $props()
</script>

{#if visible}
    <div
        transition:fade={{ duration: 150 }}
        class="absolute inset-0 z-50 flex flex-col items-center justify-center gap-3
      bg-obsidian/60 {blur ? 'backdrop-blur-sm' : ''}
      border border-stone-800/20 {className}"
        {...rest}
    >
        <div class="relative flex items-center justify-center w-8 h-8">
            <svg class="absolute" width="32" height="32" viewBox="0 0 32 32">
                <circle
                    cx="16"
                    cy="16"
                    r="12"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    class="text-stone-800"
                />
            </svg>
            <svg
                class="absolute animate-spin"
                width="32"
                height="32"
                viewBox="0 0 32 32"
                style="animation-duration: 900ms;"
            >
                <circle
                    cx="16"
                    cy="16"
                    r="12"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-dasharray="18 57"
                    class="text-sapphire"
                />
            </svg>
        </div>

        {#if message}
            <p class="text-xs font-medium tracking-wide text-stone-500">
                {message}
            </p>
            <Text />
        {/if}
    </div>
{/if}
