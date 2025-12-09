<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/state';
	import { playground } from '$lib/stores/playground.svelte';
	import TemplateSelector from '$lib/components/playground/TemplateSelector.svelte';
	import ContentEditor from '$lib/components/playground/ContentEditor.svelte';
	import MediaInputs from '$lib/components/playground/MediaInputs.svelte';
	import ColorCustomizer from '$lib/components/playground/ColorCustomizer.svelte';
	import PreviewPanel from '$lib/components/playground/PreviewPanel.svelte';
	import { Button } from '$lib/components/ui/button';
	import ShuffleIcon from '@lucide/svelte/icons/shuffle';

	let { data } = $props();

	onMount(() => {
		// Set templates from layout data
		playground.setTemplates(data.templates.all);
		// Initialize state from URL params on mount
		playground.initFromUrl(page.url.searchParams);
	});
</script>

<div class="container mx-auto max-w-7xl px-6 pt-8 pb-24">
	<!-- Header -->
	<div class="mb-8">
		<h1 class="text-3xl font-bold">Playground</h1>
		<p class="mt-2 text-muted-foreground">
			Generate custom Open Graph images in real-time. Customize templates, colors, and content.
		</p>
	</div>

	<!-- Main Layout: Side-by-side on desktop, stacked on mobile -->
	<div class="flex flex-col lg:flex-row gap-8">
		<!-- Controls Panel (Left Side) -->
		<div class="w-full lg:w-[400px] lg:shrink-0 space-y-6">
			<!-- Template Selector -->
			<section>
				<div class="flex items-center justify-between mb-3">
					<h2 class="text-sm font-medium">Template</h2>
					<Button variant="ghost" size="icon" class="size-7" onclick={() => playground.shuffleTemplates()}>
						<ShuffleIcon class="size-4" />
					</Button>
				</div>
				<TemplateSelector />
			</section>

			<!-- Content Editor -->
			<section>
				<h2 class="text-sm font-medium mb-3">Content</h2>
				<ContentEditor />
			</section>

			<!-- Media Inputs -->
			<section>
				<h2 class="text-sm font-medium mb-3">Media</h2>
				<MediaInputs />
			</section>

			<!-- Color Customizer -->
			<section>
				<h2 class="text-sm font-medium mb-3">Colors</h2>
				<ColorCustomizer />
			</section>
		</div>

		<!-- Preview Panel (Right Side) -->
		<div class="flex-1 min-w-0">
			<PreviewPanel />
		</div>
	</div>
</div>
