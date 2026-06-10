<script lang="ts">
    import { createEventDispatcher } from 'svelte'
    import { backOut, cubicIn } from 'svelte/easing'

    export let title: string
    export let description: string = ''
    export let image: string | undefined = undefined

    const dispatch = createEventDispatcher()

    const isRich = !!image

    function islandIn(node: HTMLElement) {
        return {
            duration: 480,
            easing: backOut,
            css: (t: number) => `
                transform: scaleX(${0.2 + 0.8 * t}) scaleY(${0.3 + 0.7 * t});
                opacity: ${Math.min(1, t * 2.5)};
            `,
        }
    }

    function islandOut(_node: HTMLElement) {
        return {
            duration: 260,
            easing: cubicIn,
            css: (t: number) => `
                transform: scaleX(${0.2 + 0.8 * t}) scaleY(${0.3 + 0.7 * t});
                opacity: ${t * t};
            `,
        }
    }
</script>

<!-- svelte-ignore a11y-click-events-have-key-events a11y-no-static-element-interactions -->
<div
    in:islandIn
    out:islandOut
    class="origin-top overflow-hidden rounded-[22px] border border-white/[0.07]
           bg-[#0e0e0e]/95 shadow-[0_12px_40px_-4px_rgba(0,0,0,0.9),0_0_0_0.5px_rgba(255,255,255,0.04)]
           backdrop-blur-2xl
           {isRich ? 'w-77.5' : 'w-67.5'}"
    on:click={() => dispatch('dismiss')}
>
    {#if isRich}
        <div class="flex items-center gap-3 px-3 py-3">
            <div class="h-12.5 w-12.5 shrink-0 overflow-hidden rounded-[11px] ring-1 ring-white/10">
                <img src={image} alt="" class="h-full w-full object-cover" draggable="false" />
            </div>
            <div class="min-w-0 flex-1 pr-1">
                <p class="truncate text-[13px] font-semibold leading-snug text-white">{title}</p>
                {#if description}
                    <p class="mt-0.5 line-clamp-2 text-[11.5px] leading-snug text-stone-400">
                        {description}
                    </p>
                {/if}
            </div>
        </div>
    {:else}
        <div class="px-5 py-3.5">
            <p class="text-[13px] font-semibold leading-snug text-white">{title}</p>
            {#if description}
                <p class="mt-1 text-[11.5px] leading-snug text-stone-400">{description}</p>
            {/if}
        </div>
    {/if}
</div>
