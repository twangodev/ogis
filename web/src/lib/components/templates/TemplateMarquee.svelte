<script lang="ts">
	import Marquee from '$lib/components/magic/Marquee.svelte';
	import ProgressiveBlur from '$lib/components/magic/ProgressiveBlur.svelte';
	import TemplateCard from './TemplateCard.svelte';
	import type { TemplateInfo } from './index';

	interface Props {
		title?: string;
		description?: string;
		base: TemplateInfo[];
		gradients: TemplateInfo[];
	}

	let {
		title = 'Templates',
		description,
		base,
		gradients
	}: Props = $props();

	// Split gradients into two halves
	const gradientsMid = Math.ceil(gradients.length / 2);
	const gradientsRow1 = $derived(gradients.slice(0, gradientsMid));
	const gradientsRow2 = $derived(gradients.slice(gradientsMid));
</script>

<section class="pt-4 pb-16">
	<!-- Header -->
	<div class="max-w-7xl mx-auto px-6 mb-8">
		<h2 class="text-3xl font-semibold md:text-4xl">{title}</h2>
		{#if description}
			<p class="text-muted-foreground mt-4 text-lg">{description}</p>
		{/if}
	</div>

	<!-- Marquee Container -->
	<div class="relative">
		<!-- Top row: Base templates -->
		<div class="mb-4">
			<Marquee pauseOnHover repeat={4} class="[--duration:60s] [--gap:1rem]">
				{#each base as template (template.name)}
					<div class="w-[220px] flex-none">
						<TemplateCard
							name={template.name}
							label={template.label}
						/>
					</div>
				{/each}
			</Marquee>
		</div>

		<!-- Middle row: First half of gradient templates (reversed) -->
		<div class="mb-4">
			<Marquee pauseOnHover repeat={3} reverse class="[--duration:90s] [--gap:1rem]">
				{#each gradientsRow1 as template (template.name)}
					<div class="w-[220px] flex-none">
						<TemplateCard
							name={template.name}
							label={template.label}
						/>
					</div>
				{/each}
			</Marquee>
		</div>

		<!-- Bottom row: Second half of gradient templates -->
		<Marquee pauseOnHover repeat={3} class="[--duration:100s] [--gap:1rem]">
			{#each gradientsRow2 as template (template.name)}
				<div class="w-[220px] flex-none">
					<TemplateCard
						name={template.name}
						label={template.label}
					/>
				</div>
			{/each}
		</Marquee>

		<!-- Fade edges -->
		<ProgressiveBlur
			class="pointer-events-none absolute left-0 top-0 h-full w-24"
			direction="left"
			blurIntensity={1}
		/>
		<ProgressiveBlur
			class="pointer-events-none absolute right-0 top-0 h-full w-24"
			direction="right"
			blurIntensity={1}
		/>
	</div>
</section>
