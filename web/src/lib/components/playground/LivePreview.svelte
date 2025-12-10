<script lang="ts">
	import { playground } from '$lib/stores/playground.svelte';
	import { AlertCircleIcon, RefreshCwIcon } from '@lucide/svelte';
	import { Button } from '$lib/components/ui/button';
	import ImagePreview from './ImagePreview.svelte';

	let imageError = $state(false);

	// Reset error state when preview URL changes
	$effect(() => {
		playground.previewUrl;
		imageError = false;
	});

	function retry() {
		imageError = false;
	}
</script>

<div class="relative overflow-hidden rounded-lg border border-border">
	<ImagePreview
		src={playground.previewUrl}
		alt="Open Graph preview"
		showLabel={true}
		labelText="Generating..."
	/>

	{#if imageError}
		<div
			class="absolute inset-0 flex flex-col items-center justify-center gap-3 bg-muted text-muted-foreground"
		>
			<AlertCircleIcon class="size-8" />
			<p class="text-sm">Failed to generate preview</p>
			<Button variant="outline" size="sm" onclick={retry}>
				<RefreshCwIcon class="mr-2 size-4" />
				Retry
			</Button>
		</div>
	{/if}
</div>
