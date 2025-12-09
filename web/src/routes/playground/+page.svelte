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
	<div class="flex flex-col gap-8 lg:flex-row">
		<!-- Controls Panel (Left Side) -->
		<div class="w-full space-y-6 lg:w-[400px] lg:shrink-0">
			<!-- Template Selector -->
			<section>
				<div class="mb-3 flex items-center justify-between">
					<h2 class="text-sm font-medium">Template</h2>
					<Button
						variant="ghost"
						size="icon"
						class="size-7"
						onclick={() => playground.shuffleTemplates()}
					>
						<ShuffleIcon class="size-4" />
					</Button>
				</div>
				<TemplateSelector />
			</section>

			<!-- Content Editor -->
			<section>
				<h2 class="mb-3 text-sm font-medium">Content</h2>
				<ContentEditor />
			</section>

			<!-- Media Inputs -->
			<section>
				<h2 class="mb-3 text-sm font-medium">Media</h2>
				<MediaInputs />
			</section>

			<!-- Color Customizer -->
			<section>
				<h2 class="mb-3 text-sm font-medium">Colors</h2>
				<ColorCustomizer />
			</section>
		</div>

		<!-- Preview Panel (Right Side) -->
		<div class="min-w-0 flex-1">
			<PreviewPanel />
		</div>
	</div>
</div>
