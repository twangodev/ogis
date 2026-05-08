<script lang="ts">
	import { apiConfig } from '$lib/config/api.svelte';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { Button } from '$lib/components/ui/button';
	import RotateCcwIcon from '@lucide/svelte/icons/rotate-ccw';

	let inputValue = $state(apiConfig.baseUrl);

	function handleInput(e: Event) {
		const target = e.target as HTMLInputElement;
		inputValue = target.value;
		apiConfig.baseUrl = target.value;
	}

	function handleReset() {
		apiConfig.reset();
		inputValue = apiConfig.baseUrl;
	}
</script>

<div class="space-y-2">
	<div class="flex items-center justify-between">
		<Label for="api-url">Server URL</Label>
		{#if apiConfig.isCustom}
			<Button variant="ghost" size="sm" class="h-6 px-2 text-xs" onclick={handleReset}>
				<RotateCcwIcon class="mr-1 size-3" />
				Reset
			</Button>
		{/if}
	</div>
	<Input
		id="api-url"
		type="url"
		placeholder={apiConfig.defaultUrl}
		value={inputValue}
		oninput={handleInput}
	/>
	<p class="text-xs text-muted-foreground">
		Point to your own OGIS instance for local development.
	</p>
</div>
