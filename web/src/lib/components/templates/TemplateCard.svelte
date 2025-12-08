<script lang="ts">
	import { RefreshCwIcon, ArrowRightIcon } from '@lucide/svelte';
	import { Button } from '$lib/components/ui/button';

	interface Props {
		name: string;
		label: string;
		description?: string;
		size?: 'sm' | 'md' | 'lg';
		class?: string;
	}

	let {
		name,
		label,
		description = '',
		size = 'md',
		class: className = ''
	}: Props = $props();

	const apiUrl = `https://img.ogis.dev/?template=${name}&title=${encodeURIComponent(label)}&subtitle=Open%20Graph%20Template&description=Beautiful%20images%20for%20your%20links`;

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
	class="group/card block rounded-lg overflow-hidden bg-card border border-border transition-all duration-200 hover:border-foreground/20 hover:shadow-md {className}"
	onmouseenter={() => (isHovered = true)}
	onmouseleave={() => (isHovered = false)}
>
	<!-- Image Container -->
	<div class="{sizeClasses[size]} relative overflow-hidden bg-muted">
		<img
			src={apiUrl}
			alt="{label} template preview"
			class="absolute inset-0 w-full h-full object-cover transition-all duration-300 {imageLoaded
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
			class="absolute inset-0 bg-black/0 group-hover/card:bg-black/40 transition-colors duration-200 flex items-center justify-center"
		>
			<span
				class="text-white text-sm font-medium opacity-0 group-hover/card:opacity-100 transition-opacity flex items-center gap-1"
			>
				Try it <ArrowRightIcon class="size-4" />
			</span>
		</div>
	</div>

	<!-- Label -->
	<div class="px-3 py-2.5 border-t border-border">
		<h3 class="font-medium text-sm">{label}</h3>
		{#if description}
			<p class="text-xs text-muted-foreground mt-0.5 line-clamp-1">{description}</p>
		{/if}
	</div>
</a>
