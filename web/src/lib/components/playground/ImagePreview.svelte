<script lang="ts">
	import { RefreshCwIcon } from '@lucide/svelte';

	interface Props {
		src: string;
		alt?: string;
		class?: string;
		aspectRatio?: string;
		showLabel?: boolean;
		labelText?: string;
	}

	let {
		src,
		alt = 'Preview',
		class: className = '',
		aspectRatio = 'aspect-[1200/630]',
		showLabel = false,
		labelText = 'Generating...'
	}: Props = $props();

	let imageLoaded = $state(false);

	// Reset loaded state when src changes
	$effect.pre(() => {
		// This runs before DOM updates, so we can reset state immediately
		src; // Track src as dependency
		imageLoaded = false;
	});

	function handleLoad() {
		imageLoaded = true;
	}
</script>

<div class="{aspectRatio} relative overflow-hidden bg-muted {className}">
	{#key src}
		<img
			{src}
			{alt}
			class="absolute inset-0 h-full w-full object-cover transition-all duration-500 {imageLoaded
				? 'blur-0 scale-100'
				: 'scale-105 blur-md'}"
			onload={handleLoad}
		/>
	{/key}

	{#if !imageLoaded}
		<div class="absolute inset-0 flex items-center justify-center">
			{#if showLabel}
				<div
					class="flex items-center gap-2 rounded-full border border-border bg-background/90 px-4 py-2 text-foreground shadow-lg"
				>
					<RefreshCwIcon class="size-4 animate-spin" />
					<span class="text-sm font-medium">{labelText}</span>
				</div>
			{:else}
				<RefreshCwIcon class="size-4 animate-spin text-muted-foreground" />
			{/if}
		</div>
	{/if}
</div>
