<script lang="ts">
	import CallToAction from '$lib/components/sections/CallToAction.svelte';
	import HeroContent from '$lib/components/sections/HeroContent.svelte';
	import I18nShowcase from '$lib/components/sections/I18nShowcase.svelte';
	import LogoMarquee from '$lib/components/sections/LogoMarquee.svelte';
	import PerformanceChart from '$lib/components/sections/PerformanceChart.svelte';
	import { TemplateMarquee } from '$lib/components/templates';
	import type { TemplateInfo } from '$lib/components/templates';
	import type { CloudflareStats } from '$lib/cloudflare';

	interface Props {
		templates: {
			base: TemplateInfo[];
			gradients: TemplateInfo[];
		};
		stats: CloudflareStats;
	}

	let { templates, stats }: Props = $props();
</script>

<div class="overflow-x-hidden">
	<HeroContent {stats} />
	<TemplateMarquee
		base={templates.base}
		gradients={templates.gradients}
		description="{templates.base.length +
			templates.gradients
				.length} designs, fully customizable. Adjust colors, text, images, and more, or design your own template."
	/>
	<PerformanceChart />
	<I18nShowcase />
	<LogoMarquee />
	<CallToAction />
</div>
