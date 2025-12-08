<script lang="ts">
	import { playground } from '$lib/stores/playground.svelte';
	import { AlertCircleIcon, RefreshCwIcon } from '@lucide/svelte';
	import { Button } from '$lib/components/ui/button';
	import ImagePreview from './ImagePreview.svelte';

	let imageError = $state(false);

	// Debounce the preview URL updates
	let debounceTimer: ReturnType<typeof setTimeout> | null = null;
	let previewUrl = $state(playground.apiUrl);
	let lastApiUrl = $state(playground.apiUrl);

	$effect(() => {
		const newUrl = playground.apiUrl;

		if (newUrl !== lastApiUrl) {
			lastApiUrl = newUrl;

			if (debounceTimer) clearTimeout(debounceTimer);

			debounceTimer = setTimeout(() => {
				previewUrl = newUrl;
				imageError = false;
			}, 300);
		}
	});

	function retry() {
		imageError = false;
		previewUrl = `${playground.apiUrl}&_t=${Date.now()}`;
	}
</script>

<div class="relative rounded-lg overflow-hidden border border-border">
	<ImagePreview
		src={previewUrl}
		alt="Open Graph preview"
		showLabel={true}
		labelText="Generating..."
	/>

	{#if imageError}
		<div
			class="absolute inset-0 flex flex-col items-center justify-center gap-3 text-muted-foreground bg-muted"
		>
			<AlertCircleIcon class="size-8" />
			<p class="text-sm">Failed to generate preview</p>
			<Button variant="outline" size="sm" onclick={retry}>
				<RefreshCwIcon class="size-4 mr-2" />
				Retry
			</Button>
		</div>
	{/if}
</div>