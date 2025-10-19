<script lang="ts">
	import { BarChart, Labels } from 'layerchart';
	import * as Chart from '$lib/components/ui/chart/index.js';
	import * as Tabs from '$lib/components/ui/tabs/index.js';
	import Link from '$lib/components/ui/link/link.svelte';

	const benchmarkData = {
		p50: [
			{ provider: 'ogis', warm: 11, cold: 204 },
			{ provider: '@vercel/og', warm: 35, cold: 613 },
			{ provider: 'Puppeteer', warm: 823, cold: 2547 }
		],
		p95: [
			{ provider: 'ogis', warm: 14, cold: 257 },
			{ provider: '@vercel/og', warm: 39, cold: 763 },
			{ provider: 'Puppeteer', warm: 1216, cold: 3089 }
		],
		p99: [
			{ provider: 'ogis', warm: 19, cold: 309 },
			{ provider: '@vercel/og', warm: 47, cold: 1003 },
			{ provider: 'Puppeteer', warm: 1524, cold: 3622 }
		]
	};

	let selectedMetric = $state('p95');

	const chartData = $derived(benchmarkData[selectedMetric as keyof typeof benchmarkData]);

	const chartConfig = {
		warm: {
			label: 'Warm',
			color: 'oklch(0.488 0.18 280)'
		},
		cold: {
			label: 'Cold',
			color: 'oklch(0.7 0 0 / 0.3)'
		}
	} satisfies Chart.ChartConfig;
</script>

<section>
	<div class="py-12 md:py-24">
		<div class="relative mx-auto max-w-7xl px-6">
			<div class="mb-12 flex flex-col gap-6 md:flex-row md:items-start md:justify-between">
				<div class="text-left">
					<h2 class="text-3xl font-semibold md:text-4xl">Blazingly Fast Performance</h2>
					<p class="text-muted-foreground mt-4 text-lg">
						Up to 220x faster than traditional approaches, 54x faster than{' '}
						<Link
							href="https://vercel.com/docs/functions/og-image-generation"
							external
							variant="primary"
						>
							@vercel/og
						</Link>
					</p>
				</div>

				<Tabs.Root value={selectedMetric} onValueChange={(v) => (selectedMetric = v ?? 'p95')}>
					<Tabs.List>
						<Tabs.Trigger value="p50">P50</Tabs.Trigger>
						<Tabs.Trigger value="p95">P95</Tabs.Trigger>
						<Tabs.Trigger value="p99">P99</Tabs.Trigger>
					</Tabs.List>
				</Tabs.Root>
			</div>

			<Chart.Container config={chartConfig} class="h-[250px] w-full md:px-48">
				<BarChart
					data={chartData}
					orientation="horizontal"
					y="provider"
					axis="y"
					seriesLayout="group"
					series={[
						{ key: 'warm', label: 'Warm', color: chartConfig.warm.color },
						{ key: 'cold', label: 'Cold', color: chartConfig.cold.color }
					]}
					labels={{
						offset: 8,
						format: (d) => `${d}ms`
					}}
					props={{
						bars: {
							stroke: 'none'
						},
						labels: {
						}
					}}
				>
					{#snippet tooltip()}
						<Chart.Tooltip />
					{/snippet}
				</BarChart>
			</Chart.Container>
		</div>
	</div>
</section>
