<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import CardStack from '$lib/components/hero/CardStack.svelte';
	import URLBar from '$lib/components/hero/URLBar.svelte';
	import type { CloudflareStats } from '$lib/cloudflare';
	import { apiConfig } from '$lib/config/api.svelte';
	import NumberFlow from '@number-flow/svelte';
	import { onMount } from 'svelte';

	type CTA = {
		text: string;
		href: string;
	};

	type Card = {
		id: number;
		title: string;
		description: string;
		subtitle?: string;
		logo?: string;
		image?: string;
		template?: string;
	};

	type Props = {
		title?: string;
		description?: string;
		primaryCta?: CTA;
		secondaryCta?: CTA;
		stats: CloudflareStats;
	};

	let {
		title = 'Open Graph Images for Everyone',
		description = 'Blazingly fast Open Graph images, powered by Rust. Free to use, fully customizable, and easy to integrate with any framework.',
		primaryCta = { text: 'Quick Start', href: '#' },
		secondaryCta = { text: 'Deploy Your Own', href: '#' },
		stats
	}: Props = $props();

	let activeCard = $state<Card | undefined>();

	// Animated stats - start at 0, then animate to actual values
	let displayedRequests = $state(0);
	let displayedBytesInTb = $state(0);

	onMount(() => {
		setTimeout(() => {
			displayedRequests = stats.requests;
			displayedBytesInTb = stats.bytes / 1024 ** 4; // Convert to TB
		}, 100);
	});

	// Build URL from active card with all available parameters
	const url = $derived.by(() => {
		if (!activeCard) return '';
		return apiConfig.generateUrl({
			template: activeCard.template,
			title: activeCard.title,
			description: activeCard.description,
			subtitle: activeCard.subtitle,
			logo: activeCard.logo,
			image: activeCard.image
		});
	});
</script>

<section>
	<div class="pb-24 md:pb-24 lg:pt-28">
		<div class="relative mx-auto max-w-7xl px-6">
			<div class="lg:grid lg:grid-cols-2 lg:items-center">
				<!-- Left side: Hero content -->
				<div class="mx-auto max-w-3xl text-center lg:mx-0 lg:text-left">
					<h1 class="mt-8 text-4xl font-medium text-balance md:text-5xl lg:mt-16 xl:text-6xl">
						{title}
					</h1>
					<p class="mt-8 max-w-xl text-lg text-pretty">
						{description}
					</p>

					<div
						class="mt-12 flex flex-col items-center justify-center gap-2 sm:flex-row lg:justify-start"
					>
						<Button href={primaryCta.href} size="lg" class="px-5 text-base">
							<span class="text-nowrap">{primaryCta.text}</span>
						</Button>
						<Button size="lg" variant="ghost" class="px-5 text-base" href={secondaryCta.href}>
							<span class="text-nowrap">{secondaryCta.text}</span>
						</Button>
					</div>

					<!-- Stats -->
					<div
						class="mt-8 flex flex-col items-center gap-6 text-sm text-muted-foreground sm:flex-row lg:justify-start"
					>
						<div class="flex items-center gap-2">
							<span class="font-semibold text-foreground">
								<NumberFlow
									value={displayedRequests}
									format={{ notation: 'compact', maximumFractionDigits: 3 }}
								/>
							</span>
							<span>images generated</span>
						</div>
						<div class="hidden h-4 w-px bg-border sm:block"></div>
						<div class="flex items-center gap-2">
							<span class="font-semibold text-foreground">
								<NumberFlow
									value={displayedBytesInTb}
									format={{ maximumFractionDigits: 3 }}
									suffix=" TB"
								/>
							</span>
							<span>data served</span>
						</div>
					</div>
				</div>

				<!-- Right side: 3D Card Stack -->
				<div class="flex items-center justify-center">
					<CardStack bind:activeCard />
				</div>
			</div>

			<!-- URL Bar -->
			<div class="mt-16 pt-12">
				<URLBar {url} showLabel={true} />
			</div>
		</div>
	</div>
</section>
