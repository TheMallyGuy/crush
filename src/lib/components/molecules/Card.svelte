<script lang="ts">
    import * as Card from '$lib/components/ui/card/index.js'

    export let clickable = false

    let className = ''
    export { className as class }
</script>

<Card.Root
    class="group relative flex w-full flex-col bg-sidebar/20 backdrop-blur-md transition-all duration-150 border-sidebar-border/40
    {clickable
        ? 'cursor-pointer hover:bg-sidebar-accent/50 active:scale-[0.995]'
        : ''}
    {className}"
    {...$$restProps}
>
    <slot />
</Card.Root>
