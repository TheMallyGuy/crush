<script>
  export let title = "";
  export let description = "";
  export let icon = null;
  export let clickable = false;

  let className = "";
  export { className as class };
</script>

<div
  class="group relative flex w-full max-w-sm flex-col rounded-lg border border-gray-200 bg-white p-4 shadow-sm transition-all duration-200 
  {clickable ? 'cursor-pointer hover:border-gray-300 hover:shadow-md active:scale-[0.98]' : ''} 
  {className}"
>
  <div class="flex items-start gap-4">
    {#if icon || $$slots.icon}
      <div class="flex h-10 w-10 shrink-0 items-center justify-center rounded-md bg-gray-50 text-gray-600">
        <slot name="icon">
           <svelte:component this={icon} size={20} />
        </slot>
      </div>
    {/if}

    <div class="flex flex-col gap-1">
      {#if title || $$slots.title}
        <h3 class="text-sm font-semibold leading-tight text-gray-900">
          <slot name="title">{title}</slot>
        </h3>
      {/if}
      
      {#if description || $$slots.description}
        <p class="text-xs leading-normal text-gray-500">
          <slot name="description">{description}</slot>
        </p>
      {/if}
    </div>
  </div>

  {#if $$slots.footer}
    <div class="mt-4 flex items-center gap-2">
      <slot name="footer" />
    </div>
  {/if}
</div>