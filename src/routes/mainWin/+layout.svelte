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
    import MagicRings from '$lib/components/$lib/components/svelte-bits/MagicRings.svelte'
    import { getCurrentWindow } from '@tauri-apps/api/window'

    const isOverlay = getCurrentWindow().label === 'crushOverlay'

    onMount(async () => {
        if (!settings.loaded) {
            await settings.init()
        }
    })

    $effect(() => {
        if (!isOverlay && settings.lockedInMode) {
            document.body.classList.add('force-no-cursor')
        } else {
            document.body.classList.remove('force-no-cursor')
        }
        return () => {
            document.body.classList.remove('force-no-cursor')
        }
    })
</script>

{#if !isOverlay && settings.lockedInMode}
    <TargetCursor
        targetSelector=".cursor-target"
        spinDuration={1.9}
        hideDefaultCursor={true}
        hoverDuration={0.7}
        parallaxOn={false}
    />
{/if}

{#if !isOverlay && settings.redRings}
    <div class="fixed inset-0 z-50 pointer-events-none">
        <MagicRings
            color="#ff0095"
            colorTwo="#d400ff"
            speed={1}
            ringCount={6}
            attenuation={10}
            lineThickness={2}
            baseRadius={0.35}
            radiusStep={0.1}
            scaleRate={0.1}
            opacity={1}
            blur={0}
            noiseAmount={0.1}
            rotation={0}
            ringGap={1.5}
            fadeIn={0.7}
            fadeOut={0.5}
            followMouse={true}
            mouseInfluence={0.2}
            hoverScale={1.2}
            parallax={0.05}
            clickBurst={true}
        />
    </div>
{/if}

{#if isOverlay}
    <slot />
{:else}
    <div class="flex flex-col h-screen">
        <div class="flex flex-1 overflow-hidden">
            <Topbar />
            <slot />
        </div>
    </div>
{/if}

<style>
    :global(body.force-no-cursor),
    :global(body.force-no-cursor *) {
        cursor: none !important;
    }
</style>