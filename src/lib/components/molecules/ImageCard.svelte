<script lang="ts">
    export let title = ''
    export let description = ''
    export let image = ''
    export let disabled = false

    let className = ''
    export { className as class }
</script>

<div
    class="group relative flex w-full flex-row overflow-hidden rounded-xl bg-anthracite border border-stone-800/20 transition-all duration-150
    {!disabled ? 'hover:bg-stone-900/50 hover:border-stone-700/40 hover:shadow-glow-sapphire/5' : 'opacity-50'} 
    {className}"
    {...$$restProps}
>
    <!-- Left Side: Image -->
    {#if image}
        <div class="relative w-48 shrink-0 overflow-hidden border-r border-stone-800/20">
            <img
                src={image}
                alt={title}
                class="h-full w-full object-cover transition-transform duration-500 group-hover:scale-105"
            />
            <div
                class="absolute inset-0 bg-gradient-to-l from-anthracite/40 to-transparent"
            ></div>
        </div>
    {/if}

    <!-- Right Side: Content & Actions -->
    <div class="flex flex-1 flex-col gap-4 p-5">
        <div class="flex flex-col gap-1.5">
            {#if title || $$slots.title}
                <h3
                    class="text-lg font-bold tracking-tight text-stone-100 transition-colors duration-150 group-hover:text-white"
                >
                    <slot name="title">{title}</slot>
                </h3>
            {/if}

            {#if description || $$slots.description}
                <p
                    class="text-sm font-medium leading-relaxed text-stone-500 transition-colors duration-150 group-hover:text-stone-400"
                >
                    <slot name="description">{description}</slot>
                </p>
            {/if}
        </div>

        <!-- Default slot for buttons - no need for slot="action" anymore -->
        <div class="mt-auto flex flex-wrap items-center gap-3 pt-2">
            <slot />
        </div>
    </div>
</div>
