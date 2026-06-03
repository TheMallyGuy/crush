<script context="module">
    import { waitLocale } from 'svelte-i18n'
    export const loadLocale = async () => {
        await waitLocale()
    }
</script>

<script>
    import { onMount } from 'svelte'
    import { settings } from '$lib/stores/settings.svelte'
    import Topbar from '$lib/components/organisms/Topbar.svelte'
    import TargetCursor from '$lib/components/$lib/components/svelte-bits/TargetCursor.svelte'
    import './style.css'
    import './font.css'

    onMount(async () => {
        if (!settings.loaded) {
            await settings.init()
        }
    })

    $effect(() => {
        if (settings.lockedInMode) {
            document.body.classList.add('force-no-cursor');
        } else {
            document.body.classList.remove('force-no-cursor');
        }
        return () => {
            document.body.classList.remove('force-no-cursor');
        };
    });
</script>

{#if settings.lockedInMode}
    <TargetCursor
        targetSelector=".cursor-target"
        spinDuration={1.9}
        hideDefaultCursor={true}
        hoverDuration={0.7}
        parallaxOn={false}
    />
{/if}

<div class="flex flex-col h-screen">
    <div class="flex flex-1 overflow-hidden">
        <Topbar />
        <slot />
    </div>
</div>

<style>
    :global(body.force-no-cursor),
    :global(body.force-no-cursor *) {
        cursor: none !important;
    }
</style>