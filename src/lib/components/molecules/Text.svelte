<script lang="ts">
    import { onMount } from 'svelte'
    import { _ } from 'svelte-i18n'

    let text = $state('')
    let props = $props()

    onMount(() => {
        const items = $_('pages.choiceWin.funFacts.items', {
            values: {},
        }) as unknown as string[]
        if (Array.isArray(items) && items.length > 0) {
            text = items[Math.floor(Math.random() * items.length)]
        } else {
            text = $_('pages.choiceWin.funFacts.fallback')
        }
    })
</script>

<p class="text-sm text-gray-600 {props.class}">{text}</p>
