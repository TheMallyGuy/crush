<script lang="ts">
    import type { Snippet } from 'svelte'

    interface Props {
        clickable?: boolean
        class?: string
        children?: Snippet
        [key: string]: unknown
    }

    let {
        clickable = false,
        class: className = '',
        children,
        ...rest
    }: Props = $props()
</script>

<div
    class="group relative flex w-full flex-col bg-anthracite/40 p-5 transition-all duration-150 border border-stone-800/20
  {clickable ? 'cursor-pointer hover:bg-stone-900/50 active:scale-[0.995]' : ''}
  {className}"
    {...rest}
>
    {@render children?.()}
</div>
