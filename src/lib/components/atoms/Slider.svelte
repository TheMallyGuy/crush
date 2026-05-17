<script lang="ts">
	interface Props {
		value: number
		min?: number
		max?: number
		step?: number
		label?: string
		id?: string
		disabled?: boolean
		class?: string
	}

	let {
		value = $bindable(0),
		min = 0,
		max = 100,
		step = 1,
		label = '',
		id = undefined,
		disabled = false,
		class: className = ''
	}: Props = $props()

	let percentage = $derived(max > min ? ((value - min) / (max - min)) * 100 : 0)
</script>

<div class="flex flex-col gap-2 w-full {className} {disabled ? 'opacity-50' : ''}">
	{#if label}
		<div class="flex justify-between items-center px-0.5">
			<label
				for={id}
				class="text-[11px] font-semibold tracking-tight text-stone-500 uppercase"
			>
				{label}
			</label>
			<span class="font-mono text-[11px] tabular-nums text-stone-400">
				{value}
			</span>
		</div>
	{/if}

	<div class="group relative flex h-5 items-center">
		<div
			class="h-4 w-full rounded-full border border-stone-800/40 bg-stone-900/50"
		></div>

		<div
			class="absolute h-4 rounded-full bg-white transition-all duration-75"
			style="width: {percentage}%"
		></div>

		<input
			{id}
			type="range"
			{min}
			{max}
			{step}
			{disabled}
			bind:value
			class="z-10 absolute h-full w-full cursor-pointer opacity-0 disabled:cursor-not-allowed"
		/>

		<div
			class="pointer-events-none absolute top-1/2 -translate-y-1/2 transition-all duration-75"
			style="left: {percentage}%"
		>
			<div
				class="h-4.5 w-3 -translate-x-1/2 rounded-full bg-white transition-transform duration-150 group-hover:scale-110 group-active:scale-95"
			></div>
		</div>
	</div>
</div>
