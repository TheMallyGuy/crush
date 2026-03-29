<script lang="ts">
    export let clickable = false

    let className = ''
    export { className as class }
</script>

<div
    class="group relative flex w-full flex-col rounded-xl bg-[#0f0f0f] p-5 transition-all duration-200
  {clickable ? 'cursor-pointer hover:bg-[#161616] active:scale-[0.995]' : ''} 
  {className}"
    {...$$restProps}
>
    <slot />
</div>
