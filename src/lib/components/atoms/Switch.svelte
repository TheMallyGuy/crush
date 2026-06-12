<script lang="ts">
    interface Props {
        checked?: boolean
        disabled?: boolean
        id?: string
        label?: string
        class?: string
        onchange?: (checked: boolean) => void
    }

    let {
        checked = $bindable(),
        disabled = false,
        id = undefined,
        label = 'Toggle',
        class: className = '',
        onchange,
    }: Props = $props()

    function toggle() {
        if (!disabled) {
            checked = !checked
            onchange?.(checked)
        }
    }
</script>

<button
    type="button"
    role="switch"
    aria-checked={checked}
    aria-label={label}
    {disabled}
    {id}
    onclick={toggle}
    class="relative cursor-target inline-flex h-6 w-11 shrink-0 cursor-pointer items-center transition-all duration-150 focus:outline-none focus:ring-2 focus:ring-sapphire/50 focus:ring-offset-2 focus:ring-offset-obsidian disabled:cursor-not-allowed disabled:opacity-40
    {checked
        ? 'bg-sapphire'
        : 'bg-stone-700 hover:bg-stone-600'}
    {className}"
>
    <span
        class="pointer-events-none inline-block h-4 w-4 transform bg-white shadow-sm transition-transform duration-150 ease-in-out
        {checked ? 'translate-x-6' : 'translate-x-1'}"
    ></span>
</button>
