<script lang="ts">
    export let title = ''
    export let description = ''
    export let image = ''
    export let disabled = false

    let className = ''
    export { className as class }
</script>

<div
    class="group relative flex w-full flex-row overflow-hidden rounded-xl bg-card/40 backdrop-blur-md transition-all duration-150 
    {!disabled ? 'hover:bg-muted/50 hover:border-border hover:shadow-glow-sapphire/5' : 'opacity-50'} 
    {className}"
    {...$$restProps}
>
    {#if image}
        <div class="relative w-48 shrink-0 overflow-hidden border-r border-border">
            <img
                src={image}
                alt={title}
                class="h-full w-full object-cover transition-transform duration-500 group-hover:scale-105"
            />
            <div
                class="absolute inset-0 bg-gradient-to-l from-black/40 to-transparent"
            ></div>
        </div>
    {/if}

    <div class="flex flex-1 flex-col gap-4 p-5">
        <div class="flex flex-col gap-1.5">
            {#if title || $$slots.title}
                <h3
                    class="text-lg font-bold tracking-tight text-foreground transition-colors duration-150 group-hover:text-foreground"
                >
                    <slot name="title">{title}</slot>
                </h3>
            {/if}

            {#if description || $$slots.description}
                <p
                    class="text-sm font-medium leading-relaxed text-muted-foreground transition-colors duration-150 group-hover:text-muted-foreground/80"
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
