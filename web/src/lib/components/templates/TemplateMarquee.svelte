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
</script>

<section class="py-16">
	<!-- Header -->
	<div class="max-w-6xl mx-auto px-6 mb-6">
		<h2 class="text-2xl font-medium">{title}</h2>
		{#if description}
			<p class="text-muted-foreground mt-1">{description}</p>
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

		<!-- Bottom row: Gradient templates (reversed) -->
		<Marquee pauseOnHover repeat={3} reverse class="[--duration:80s] [--gap:1rem]">
			{#each gradients as template (template.name)}
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
