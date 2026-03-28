<script lang="ts">
    import { createEventDispatcher } from 'svelte'

    export let checked = false
    export let disabled = false
    export let id: string | undefined = undefined

    let className = ''
    export { className as class }

    const dispatch = createEventDispatcher()

    function toggle() {
        if (!disabled) {
            checked = !checked
            dispatch('change', checked)
        }
    }
</script>

<button
    type="button"
    role="switch"
    aria-checked={checked}
    {disabled}
    {id}
    on:click={toggle}
    class="relative inline-flex h-6 w-11 shrink-0 cursor-pointer items-center rounded-full transition-all duration-200 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 focus:ring-offset-black disabled:cursor-not-allowed disabled:opacity-40
    {checked ? 'bg-blue-600' : 'bg-stone-700 hover:bg-stone-600'} 
    {className}"
>
    <span
        class="pointer-events-none inline-block h-4 w-4 transform rounded-full bg-white shadow-sm transition-transform duration-200 ease-in-out
        {checked ? 'translate-x-6' : 'translate-x-1'}"
    />
</button>
