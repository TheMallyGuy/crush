<script lang="ts">
    import { Check, Circle } from '@lucide/svelte'

    export let steps: { label: string; description?: string }[] = []
    export let currentStep = 0


    function getStepState(index: number) {
        if (index < currentStep) return 'completed'
        if (index === currentStep) return 'active'
        return 'upcoming'
    }
    $: stepStates = steps.map((_, i) => {
        if (i < currentStep) return 'completed'
        if (i === currentStep) return 'active'
        return 'upcoming'
    })
</script>

<aside
    class="flex flex-col h-screen bg-anthracite text-stone-400 px-7 pb-7 pt-14 transition-all duration-150 w-64 "
>
    <div class="flex flex-col items-center sm:items-start mb-10 gap-2 mt-1">
        <img
            src="/Crush.svg"
            alt="Logo"
            class="w-10 h-10 mb-2"
        />
        <h2 class="text-stone-100 font-bold text-lg tracking-tight">First launch? Welcome to Crush!</h2>
        <p class="text-xs text-stone-500 tracking-wider">Crush hello</p>
    </div>

    <nav class="flex flex-col gap-6">
        {#each steps as step, i}
            {@const stepState = stepStates[i]}
            <div class="flex gap-4 items-start group">
                <div class="relative flex flex-col items-center">
                    <div
                        class="w-6 h-6 rounded-full flex items-center justify-center transition-all duration-300 z-10
                        {stepState === 'completed' ? 'bg-sapphire text-white' : 
                         stepState === 'active' ? 'bg-stone-800 text-white' : 
                         'bg-stone-900 text-stone-600'}"
                    >
                        {#if stepState === 'completed'}
                            <Check size={14} strokeWidth={3} />
                        {:else}
                            <span class="text-[10px] font-bold">{i + 1}</span>
                        {/if}
                    </div>

                    {#if i < steps.length - 1}
                        <div
                            class="absolute top-6 w-0.5 h-6 transition-colors duration-300
                            {stepState === 'completed' ? 'bg-sapphire' : 'bg-stone-800'}"
                        ></div>
                    {/if}
                </div>

                <div class="flex flex-col gap-0.5 pt-0.5">
                    <span
                        class="text-sm font-semibold transition-colors duration-300
                        {stepState === 'active' ? 'text-stone-100' : 
                         stepState === 'completed' ? 'text-stone-300' : 'text-stone-600'}"
                    >
                        {step.label}
                    </span>
                    {#if step.description}
                        <span
                            class="text-[11px] leading-tight transition-colors duration-300
                            {stepState === 'active' ? 'text-stone-400' : 'text-stone-700'}"
                        >
                            {step.description}
                        </span>
                    {/if}
                </div>
            </div>
        {/each}
    </nav>

</aside>


