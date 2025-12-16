<script lang="ts">
	import * as Tabs from '$lib/components/ui/tabs/index.js';
	import Link from '$lib/components/ui/link/link.svelte';
	import { Tween } from 'svelte/motion';
	import { cubicOut } from 'svelte/easing';
	
	const benchmarkData = {
		p50: { ogis: 6, vercel: 34 },
		p95: { ogis: 6, vercel: 50 },
		p99: { ogis: 6, vercel: 105 }
	};

	const throughput = {
		ogis: 365.89,
		vercel: 32.7
	};

	let selectedCategory = $state<'latency' | 'throughput'>('latency');
	let selectedPercentile = $state<'p50' | 'p95' | 'p99'>('p95');

	const currentLatencyData = $derived(benchmarkData[selectedPercentile]);
	const maxLatency = $derived(Math.max(currentLatencyData.ogis, currentLatencyData.vercel));
	const maxThroughput = $derived(Math.max(throughput.ogis, throughput.vercel));

	const latencySpeedup = $derived(Math.round(currentLatencyData.vercel / currentLatencyData.ogis));
	const throughputSpeedup = $derived(Math.round(throughput.ogis / throughput.vercel));

	// Animated bar widths
	const ogisWidth = new Tween(0, { duration: 300, easing: cubicOut });
	const vercelWidth = new Tween(0, { duration: 1000, easing: cubicOut });

	// Update bar widths based on selected category
	$effect(() => {
		if (selectedCategory === 'latency') {
			const ogisPercent = (currentLatencyData.ogis / maxLatency) * 100;
			const vercelPercent = (currentLatencyData.vercel / maxLatency) * 100;
			ogisWidth.set(ogisPercent);
			vercelWidth.set(vercelPercent);
		} else {
			const ogisPercent = (throughput.ogis / maxThroughput) * 100;
			const vercelPercent = (throughput.vercel / maxThroughput) * 100;
			ogisWidth.set(ogisPercent);
			vercelWidth.set(vercelPercent);
		}
	});
</script>

<section>
	<div class="py-12 md:py-24">
		<div class="relative mx-auto max-w-7xl px-6">
			<!-- Header -->
			<div class="mb-12 flex flex-col gap-6 md:flex-row md:items-start md:justify-between">
				<div class="text-left">
					<h2 class="text-3xl font-semibold md:text-4xl">Blazingly Fast Performance</h2>
					<p class="mt-4 text-lg text-muted-foreground">
						Up to {latencySpeedup}x faster latency and {throughputSpeedup}x higher throughput
						than{' '}
						<Link
							href="https://vercel.com/docs/functions/og-image-generation"
							external
							variant="primary"
						>
							@vercel/og
						</Link>
					</p>
				</div>

				<!-- Tabs -->
				<div class="flex items-center gap-4">
					<Tabs.Root
						value={selectedCategory}
						onValueChange={(v) => {
							if (v === 'latency' || v === 'throughput') {
								selectedCategory = v;
							}
						}}
					>
						<Tabs.List>
							<Tabs.Trigger value="latency">Latency</Tabs.Trigger>
							<Tabs.Trigger value="throughput">Throughput</Tabs.Trigger>
						</Tabs.List>
					</Tabs.Root>

					{#if selectedCategory === 'latency'}
						<Tabs.Root
							value={selectedPercentile}
							onValueChange={(v) => {
								if (v === 'p50' || v === 'p95' || v === 'p99') {
									selectedPercentile = v;
								}
							}}
						>
							<Tabs.List>
								<Tabs.Trigger value="p50">P50</Tabs.Trigger>
								<Tabs.Trigger value="p95">P95</Tabs.Trigger>
								<Tabs.Trigger value="p99">P99</Tabs.Trigger>
							</Tabs.List>
						</Tabs.Root>
					{/if}
				</div>
			</div>

			<!-- Racing Bars -->
			<div class="space-y-4 md:px-12">
				<!-- ogis bar -->
				<div class="flex items-center gap-4">
					<span class="w-28 text-right font-medium">ogis</span>
					<div class="relative h-10 flex-1 overflow-hidden rounded-lg bg-muted/30">
						<div
							class="absolute inset-y-0 left-0 flex min-w-32 items-center rounded-lg"
							style="width: {ogisWidth.current}%; background: oklch(0.488 0.18 280)"
						>
							<span class="absolute right-3 whitespace-nowrap font-mono text-sm font-semibold text-white">
								{#if selectedCategory === 'latency'}
									{currentLatencyData.ogis}ms
								{:else}
									{throughput.ogis} req/s
								{/if}
							</span>
						</div>
					</div>
				</div>

				<!-- vercel bar -->
				<div class="flex items-center gap-4">
					<span class="w-28 text-right font-medium text-muted-foreground">@vercel/og</span>
					<div class="relative h-10 flex-1 overflow-hidden rounded-lg bg-muted/30">
						<div
							class="absolute inset-y-0 left-0 flex min-w-32 items-center rounded-lg bg-muted-foreground/60"
							style="width: {vercelWidth.current}%"
						>
							<span class="absolute right-3 whitespace-nowrap font-mono text-sm font-semibold text-foreground">
								{#if selectedCategory === 'latency'}
									{currentLatencyData.vercel}ms
								{:else}
									{throughput.vercel} req/s
								{/if}
							</span>
						</div>
					</div>
				</div>
			</div>

			<!-- Footer -->
			<p class="mt-10 text-center text-sm text-muted-foreground">
				Benchmarks run on identical hardware.{' '}
				<Link href="https://github.com/twangodev/ogis/actions/workflows/rust.yml" external variant="primary">
					View methodology
				</Link>
			</p>
		</div>
	</div>
</section>
