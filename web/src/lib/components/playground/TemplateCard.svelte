<script lang="ts">
	import ImagePreview from './ImagePreview.svelte';

	interface Props {
		name: string;
		label: string;
		selected?: boolean;
		onclick?: () => void;
	}

	let { name, label, selected = false, onclick }: Props = $props();

	// Generate thumbnail URL with default content
	const thumbnailUrl = $derived(
		`https://img.ogis.dev/?template=${name}&title=Preview&description=Template%20Preview`
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
	class="relative rounded-lg overflow-hidden border-2 transition-all duration-200 hover:border-primary/50 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring {selected
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
		class="absolute bottom-0 left-0 right-0 bg-gradient-to-t from-black/80 to-transparent px-2 py-1"
	>
		<span class="text-xs font-medium text-white">{label}</span>
	</div>
</button>