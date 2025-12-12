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
			title: 'Ship Faster with Vercel',
			description: 'Deploy web apps instantly with zero configuration',
			subtitle: 'Frontend Cloud Platform',
			logo: 'https://ogis.dev/demo/vercel.png',
			template: 'gradient-storm'
		},
		{
			id: 3,
			title: 'Build with Svelte',
			description: 'Runes, fine-grained reactivity, and next-gen performance',
			subtitle: 'Framework Release',
			logo: 'https://ogis.dev/demo/svelte.png',
			template: 'gradient-ember'
		},
		{
			id: 4,
			title: 'Tropical Paradise',
			description: 'Discover crystal clear waters and pristine beaches',
			subtitle: 'Travel Collection',
			image: 'https://images.unsplash.com/photo-1507525428034-b723cf961d3e?w=800&fm=png',
			template: 'gradient-tropics'
		},
		{
			id: 5,
			title: 'GitHub Copilot',
			description: 'Your AI pair programmer for faster, smarter coding',
			subtitle: 'AI-Powered Development',
			logo: 'https://ogis.dev/demo/github.png',
			template: 'gradient-midnight'
		},
		{
			id: 6,
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
