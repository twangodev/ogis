<script lang="ts">
	import ImagePreview from './ImagePreview.svelte';
	import { apiConfig } from '$lib/config/api.svelte';

	interface Props {
		name: string;
		label: string;
		selected?: boolean;
		onclick?: () => void;
	}

	let { name, label, selected = false, onclick }: Props = $props();

	// Generate thumbnail URL with default content
	// Optimize for small thumbnails: WebP at 20% scale with quality 70
	const thumbnailUrl = $derived(
		apiConfig.generateUrl({
			template: name,
			title: 'Preview',
			description: 'Template Preview',
			format: 'webp',
			scale: '0.2',
			quality: '70'
		})
	);

	// Use IntersectionObserver for lazy loading
	let shouldLoad = $state(false);
	let containerRef = $state<HTMLButtonElement | null>(null);

	$effect(() => {
		if (!containerRef) return;

		const observer = new IntersectionObserver(
			(entries) => {
				if (entries[0].isIntersecting) {
					shouldLoad = true;
					observer.disconnect();
				}
			},
			{ rootMargin: '100px' }
		);

		observer.observe(containerRef);

		return () => observer.disconnect();
	});
</script>

<button
	bind:this={containerRef}
	type="button"
	class="relative w-full overflow-hidden rounded-lg border-2 transition-all duration-200 hover:border-primary/50 focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none {selected
		? 'border-primary ring-2 ring-primary/20'
		: 'border-border'}"
	{onclick}
>
	{#if shouldLoad}
		<ImagePreview src={thumbnailUrl} alt={label} />
	{:else}
		<div class="aspect-[1200/630] bg-muted"></div>
	{/if}

	<!-- Label -->
	<div
		class="absolute right-0 bottom-0 left-0 bg-gradient-to-t from-black/80 to-transparent px-2 py-1"
	>
		<span class="text-xs font-medium text-white">{label}</span>
	</div>
</button>
