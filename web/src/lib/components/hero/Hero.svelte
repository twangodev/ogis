<script lang="ts">
	import HeroContent from '$lib/components/sections/HeroContent.svelte';
	import LogoMarquee from '$lib/components/sections/LogoMarquee.svelte';
	import PerformanceChart from '$lib/components/sections/PerformanceChart.svelte';
	import { TemplateMarquee } from '$lib/components/templates';
</script>

<div class="overflow-x-hidden">
	<HeroContent />
	<LogoMarquee />
	<TemplateMarquee />
	<PerformanceChart />
</div>
