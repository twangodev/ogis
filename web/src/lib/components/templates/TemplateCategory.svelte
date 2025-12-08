<script lang="ts">
	import { ChevronLeftIcon, ChevronRightIcon } from '@lucide/svelte';
	import { Button } from '$lib/components/ui/button';
	import TemplateCard from './TemplateCard.svelte';
	import type { TemplateInfo } from './index';

	interface Props {
		title: string;
		description?: string;
		templates: readonly TemplateInfo[];
		layout?: 'scroll' | 'grid' | 'bento';
	}

	let { title, description = '', templates, layout = 'scroll' }: Props = $props();

	let scrollContainer = $state<HTMLDivElement | null>(null);
	let canScrollLeft = $state(false);
	let canScrollRight = $state(true);

	function updateScrollState() {
		if (!scrollContainer) return;
		canScrollLeft = scrollContainer.scrollLeft > 0;
		canScrollRight =
			scrollContainer.scrollLeft < scrollContainer.scrollWidth - scrollContainer.clientWidth - 10;
	}

	function scrollBy(direction: 'left' | 'right') {
		if (!scrollContainer) return;
		const scrollAmount = 360;
		scrollContainer.scrollBy({
			left: direction === 'left' ? -scrollAmount : scrollAmount,
			behavior: 'smooth'
		});
	}
</script>

<section class="py-16">
	<!-- Header -->
	<div class="max-w-6xl mx-auto px-6 mb-6">
		<div class="flex items-end justify-between">
			<div>
				<h2 class="text-2xl font-medium">{title}</h2>
				{#if description}
					<p class="text-muted-foreground mt-1">{description}</p>
				{/if}
			</div>

			{#if layout === 'scroll'}
				<div class="hidden md:flex items-center gap-1">
					<Button
						variant="ghost"
						size="icon"
						disabled={!canScrollLeft}
						onclick={() => scrollBy('left')}
					>
						<ChevronLeftIcon class="size-4" />
					</Button>
					<Button
						variant="ghost"
						size="icon"
						disabled={!canScrollRight}
						onclick={() => scrollBy('right')}
					>
						<ChevronRightIcon class="size-4" />
					</Button>
				</div>
			{/if}
		</div>
	</div>

	<!-- Templates -->
	{#if layout === 'scroll'}
		<div
			bind:this={scrollContainer}
			onscroll={updateScrollState}
			class="flex gap-4 overflow-x-auto px-6 md:px-[max(1.5rem,calc((100vw-72rem)/2+1.5rem))] pb-4 scrollbar-hide"
		>
			{#each templates as template (template.name)}
				<div class="flex-none w-[300px]">
					<TemplateCard
						name={template.name}
						label={template.label}
					/>
				</div>
			{/each}
		</div>
	{:else if layout === 'grid'}
		<div class="max-w-6xl mx-auto px-6">
			<div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
				{#each templates as template (template.name)}
					<TemplateCard
						name={template.name}
						label={template.label}
					/>
				{/each}
			</div>
		</div>
	{:else if layout === 'bento'}
		<div class="max-w-6xl mx-auto px-6">
			<div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
				{#each templates as template, i (template.name)}
					{@const isLarge = i === 0}
					<div class="{isLarge ? 'sm:col-span-2' : ''}">
						<TemplateCard
							name={template.name}
							label={template.label}
							size={isLarge ? 'lg' : 'md'}
						/>
					</div>
				{/each}
			</div>
		</div>
	{/if}
</section>

<style>
	.scrollbar-hide {
		-ms-overflow-style: none;
		scrollbar-width: none;
	}
	.scrollbar-hide::-webkit-scrollbar {
		display: none;
	}
</style>
