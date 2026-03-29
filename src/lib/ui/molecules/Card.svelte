<script lang="ts">
    export let clickable = false

    let className = ''
    export { className as class }
</script>

<div
    class="group relative flex w-full flex-col rounded-xl bg-anthracite p-5 transition-all duration-150 border border-stone-800/20
  {clickable
        ? 'cursor-pointer hover:bg-stone-900/50 active:scale-[0.995]'
        : ''}
  {className}"
    {...$$restProps}
>
    <slot />
</div>
