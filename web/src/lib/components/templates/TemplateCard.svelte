<script lang="ts">
	import { RefreshCwIcon, ArrowRightIcon } from '@lucide/svelte';
	import { Button } from '$lib/components/ui/button';
	import { apiConfig } from '$lib/config/api.svelte';

	interface Props {
		name: string;
		label: string;
		description?: string;
		size?: 'sm' | 'md' | 'lg';
		class?: string;
	}

	let { name, label, description = '', size = 'md', class: className = '' }: Props = $props();

	// Optimize for thumbnails: WebP at 30% scale with quality 75
	const apiUrl = $derived(
		apiConfig.generateUrl({
			template: name,
			title: label,
			subtitle: 'Open Graph Template',
			description: 'Beautiful images for your links',
			format: 'webp',
			scale: '0.3',
			quality: '75'
		})
	);

	let imageLoaded = $state(false);
	let isHovered = $state(false);

	function handleLoad() {
		imageLoaded = true;
	}

	const sizeClasses = {
		sm: 'aspect-[1200/630]',
		md: 'aspect-[1200/630]',
		lg: 'aspect-[1200/630] min-h-[280px]'
	};
</script>

<a
	href="/playground?template={name}"
	class="group/card block overflow-hidden rounded-lg border border-border bg-card transition-all duration-200 hover:border-foreground/20 hover:shadow-md {className}"
	onmouseenter={() => (isHovered = true)}
	onmouseleave={() => (isHovered = false)}
>
	<!-- Image Container -->
	<div class="{sizeClasses[size]} relative overflow-hidden bg-muted">
		<img
			src={apiUrl}
			alt="{label} template preview"
			class="absolute inset-0 h-full w-full object-cover transition-all duration-300 {imageLoaded
				? 'opacity-100'
				: 'opacity-0'} {isHovered ? 'scale-[1.02]' : 'scale-100'}"
			onload={handleLoad}
		/>

		{#if !imageLoaded}
			<div class="absolute inset-0 flex items-center justify-center">
				<RefreshCwIcon class="size-5 animate-spin text-muted-foreground" />
			</div>
		{/if}

		<!-- Hover Overlay -->
		<div
			class="absolute inset-0 flex items-center justify-center bg-black/0 transition-colors duration-200 group-hover/card:bg-black/40"
		>
			<span
				class="flex items-center gap-1 text-sm font-medium text-white opacity-0 transition-opacity group-hover/card:opacity-100"
			>
				Try it <ArrowRightIcon class="size-4" />
			</span>
		</div>
	</div>

	<!-- Label -->
	<div class="border-t border-border px-3 py-2.5">
		<h3 class="text-sm font-medium">{label}</h3>
		{#if description}
			<p class="mt-0.5 line-clamp-1 text-xs text-muted-foreground">{description}</p>
		{/if}
	</div>
</a>
