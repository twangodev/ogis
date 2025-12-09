<script lang="ts">
	import { playground } from '$lib/stores/playground.svelte';
	import { Button } from '$lib/components/ui/button';
	import ColorPicker from './ColorPicker.svelte';
	import { RotateCcwIcon, ChevronDownIcon, ChevronUpIcon } from '@lucide/svelte';

	let expanded = $state(false);

	// Get available colors for current template
	const templateColors = $derived(playground.currentTemplate?.colors ?? []);

	// Check if any colors are customized
	const hasCustomColors = $derived(Object.values(playground.colors).some((v) => v && v.length > 0));

	function handleColorChange(key: string, value: string) {
		playground.updateColor(key, value);
	}

	function resetColors() {
		playground.resetColors();
	}

	// Format color key for display (e.g., "title_text" -> "Title Text")
	function formatColorKey(key: string): string {
		return key
			.split('_')
			.map((word) => word.charAt(0).toUpperCase() + word.slice(1))
			.join(' ');
	}
</script>

<div class="space-y-3">
	<button
		type="button"
		class="flex w-full items-center justify-between text-left"
		onclick={() => (expanded = !expanded)}
	>
		<span class="text-sm text-muted-foreground">
			{templateColors.length} customizable colors
			{#if hasCustomColors}
				<span class="text-primary">(modified)</span>
			{/if}
		</span>
		{#if expanded}
			<ChevronUpIcon class="size-4 text-muted-foreground" />
		{:else}
			<ChevronDownIcon class="size-4 text-muted-foreground" />
		{/if}
	</button>

	{#if expanded}
		<div class="space-y-3 pt-2">
			{#if hasCustomColors}
				<Button variant="outline" size="sm" class="w-full" onclick={resetColors}>
					<RotateCcwIcon class="mr-2 size-4" />
					Reset to defaults
				</Button>
			{/if}

			<div class="grid grid-cols-2 gap-3">
				{#each templateColors as colorKey}
					<ColorPicker
						label={formatColorKey(colorKey)}
						value={playground.colors[colorKey] ?? ''}
						onchange={(value) => handleColorChange(colorKey, value)}
					/>
				{/each}
			</div>
		</div>
	{/if}
</div>
