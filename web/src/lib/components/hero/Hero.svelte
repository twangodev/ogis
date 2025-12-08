<script lang="ts">
	import HeroContent from '$lib/components/sections/HeroContent.svelte';
	import LogoMarquee from '$lib/components/sections/LogoMarquee.svelte';
	import PerformanceChart from '$lib/components/sections/PerformanceChart.svelte';
	import { TemplateMarquee } from '$lib/components/templates';
	import type { TemplateInfo } from '$lib/components/templates';

	interface Props {
		templates: {
			base: TemplateInfo[];
			gradients: TemplateInfo[];
		};
	}

	let { templates }: Props = $props();
</script>

<div class="overflow-x-hidden">
	<HeroContent />
	<TemplateMarquee
		base={templates.base}
		gradients={templates.gradients}
		description="{templates.base.length + templates.gradients.length} designs, fully customizable. Adjust colors, text, images, and more, or design your own template."
	/>
	<PerformanceChart />
	<LogoMarquee />
</div>
