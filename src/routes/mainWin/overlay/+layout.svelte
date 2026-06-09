<script lang="ts">
    import '../style.css'
    let { children } = $props()
</script>

<svelte:head>
    <style>
        html, body { background: transparent !important; }
    </style>
</svelte:head>

{@render children()}
