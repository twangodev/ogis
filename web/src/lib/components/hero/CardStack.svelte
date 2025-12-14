<script lang="ts">
	import OGCard from './OGCard.svelte';
	import { onMount } from 'svelte';

	type Card = {
		id: number;
		title: string;
		description: string;
		subtitle?: string;
		logo?: string;
		image?: string;
		template?: string;
		params?: Record<string, string>;
	};

	type Props = {
		activeCard?: Card;
	};

	let { activeCard = $bindable() }: Props = $props();

	const cards: Card[] = [
		{
			id: 1,
			title: 'Open Graph Images',
			description: 'Fast, free, and beautiful image generation powered by Rust',
			subtitle: 'Open Source • Always Free',
			logo: 'https://ogis.dev/logo-light.png',
			template: 'twilight'
		},
		{
			id: 2,
			title: 'Introducing ChatGPT',
			description: 'Get instant answers, find creative inspiration, and learn something new',
			subtitle: 'OpenAI',
			logo: 'https://ogis.dev/demo/openai.png',
			template: 'gradient-storm',
			params: {
				background: '171212',
				blob_slate: '000000',
				blob_dark_slate: '383838',
				blob_gray: '1f1f1f',
				blob_light_gray: '292929',
				blob_silver: '1c1c1c'
			}
		},
		{
			id: 3,
			title: 'Claude by Anthropic',
			description: 'AI assistant built to be helpful, harmless, and honest',
			subtitle: 'Next-Gen AI',
			logo: 'https://ogis.dev/demo/anthropic.png',
			template: 'gradient-peach'
		},
		{
			id: 4,
			title: 'Stripe Payments',
			description: 'Financial infrastructure for the internet economy',
			subtitle: 'Developer-First APIs',
			logo: 'https://ogis.dev/demo/stripe.png',
			template: 'gradient-galaxy'
		},
		{
			id: 5,
			title: 'Build with Supabase',
			description: 'The open source Firebase alternative with Postgres',
			subtitle: 'Backend Platform',
			logo: 'https://ogis.dev/demo/supabase.png',
			template: 'gradient-jade'
		},
		{
			id: 7,
			title: 'Tailwind CSS',
			description: 'Rapidly build modern websites without leaving your HTML',
			subtitle: 'Utility-First CSS',
			logo: 'https://ogis.dev/demo/tailwind.png',
			template: 'gradient-sapphire'
		},
		{
			id: 9,
			title: 'Ship Faster with Vercel',
			description: 'Deploy web apps instantly with zero configuration',
			subtitle: 'Frontend Cloud',
			logo: 'https://ogis.dev/demo/vercel.png',
			template: 'gradient-storm'
		},
		{
			id: 10,
			title: 'Build with Svelte',
			description: 'Runes, fine-grained reactivity, and next-gen performance',
			subtitle: 'Framework Release',
			logo: 'https://ogis.dev/demo/svelte.png',
			template: 'gradient-ember'
		},
		{
			id: 11,
			title: 'GitHub Copilot',
			description: 'Your AI pair programmer for faster, smarter coding',
			subtitle: 'AI-Powered Development',
			logo: 'https://ogis.dev/demo/github.png',
			template: 'gradient-midnight'
		},
		{
			id: 12,
			title: 'Discord Communities',
			description: 'Connect with developers, gamers, and creators',
			subtitle: 'Chat & Collaborate',
			logo: 'https://ogis.dev/demo/discord.png',
			template: 'gradient-cobalt'
		}
	];

	// Cycling state
	let currentStartIndex = $state(Math.floor(Math.random() * cards.length));
	let hoveredCard = $state<number | null>(null);
	let isPaused = $state(false);
	let cycleInterval: ReturnType<typeof setInterval> | null = null;

	// Get 3 cards starting from current index with wrapping
	const displayedCards = $derived([
		cards[currentStartIndex % cards.length],
		cards[(currentStartIndex + 1) % cards.length],
		cards[(currentStartIndex + 2) % cards.length]
	]);

	// Update activeCard whenever displayedCards changes (middle card is index 1)
	$effect(() => {
		activeCard = displayedCards[1];
	});

	// Calculate vertical offset to center the whole stack
	const verticalOffset = 80; // (3 - 1) * 80 / 2

	function startCycle() {
		if (cycleInterval) return;
		cycleInterval = setInterval(() => {
			if (!isPaused) {
				currentStartIndex = (currentStartIndex + 1) % cards.length;
			}
		}, 3000); // Cycle every 3 seconds
	}

	function stopCycle() {
		if (cycleInterval) {
			clearInterval(cycleInterval);
			cycleInterval = null;
		}
	}

	onMount(() => {
		startCycle();
		return () => stopCycle();
	});
</script>

<div
	class="relative flex h-[350px] w-full max-w-[500px] items-center justify-center"
	style="perspective: 1000px;"
	role="region"
	aria-label="Open Graph image examples"
	onmouseenter={() => (isPaused = true)}
	onmouseleave={() => {
		isPaused = false;
		hoveredCard = null;
	}}
>
	{#each displayedCards as card, i (card.id)}
		<OGCard
			title={card.title}
			description={card.description}
			subtitle={card.subtitle}
			logo={card.logo}
			image={card.image}
			template={card.template}
			params={card.params}
			index={i}
			totalCards={3}
			isHovered={hoveredCard === i}
			isDimmed={hoveredCard !== null && hoveredCard !== i}
			{verticalOffset}
			onHover={() => (hoveredCard = i)}
			onLeave={() => (hoveredCard = null)}
		/>
	{/each}
</div>
