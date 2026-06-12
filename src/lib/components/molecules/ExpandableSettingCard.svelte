<script lang="ts">
  import type { Component } from 'svelte'
  import { ChevronDown } from '@lucide/svelte'
  import { slide } from 'svelte/transition'

  export let title = ''
  export let description = ''
  export let icon: Component | string | null = null
  export let isOpen = false
  let className = ''
  export { className as class }

  function toggle() {
    isOpen = !isOpen
  }
</script>

<div
    class="group relative flex w-full flex-col p-3 transition-all duration-150 {className}"
>
  <button
    type="button"
    class="flex items-center justify-between gap-5 text-left cursor-pointer w-full focus:outline-none"
    on:click={toggle}
  >
    <div class="flex items-center gap-5">
      {#if icon || $$slots.icon}
        <div class="flex h-10 w-10 shrink-0 items-center justify-center text-stone-400 transition-colors duration-150 overflow-hidden">
          <slot name="icon">
            {#if typeof icon === 'string'}
              <img src={icon} alt="" class="w-full h-full object-cover" />
            {:else if icon}
              <svelte:component this={icon} size={24} />
            {/if}
          </slot>
        </div>
      {/if}
      <div class="flex flex-col gap-0.5">
        {#if title || $$slots.title}
          <h3 class="text-base font-semibold tracking-tight text-stone-100">
            <slot name="title">{title}</slot>
          </h3>
        {/if}
        {#if description || $$slots.description}
          <p class="text-sm font-medium text-stone-300">
            <slot name="description">{description}</slot>
          </p>
        {/if}
      </div>
    </div>
    <div class="flex items-center gap-3">
      <slot name="action" />
      <div class="text-stone-500 transition-transform duration-200 {isOpen ? 'rotate-180' : ''}">
        <ChevronDown size={20} />
      </div>
    </div>
  </button>
  {#if isOpen}
    <div transition:slide={{ duration: 200 }}>
      <div class="mt-5 flex flex-col gap-4">
        <slot />
        {#if $$slots.footer}
          <div class="flex flex-col gap-4">
            <slot name="footer" />
          </div>
        {/if}
      </div>
    </div>
  {/if}
</div>