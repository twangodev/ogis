<script lang="ts">
	import OGCard from './OGCard.svelte';
	import { onMount } from 'svelte';

	type Card = {
		id: number;
		title: string;
		description: string;
		subtitle?: string;
		logo?: string;
	};

	type Props = {
		activeCard?: Card;
	};

	let { activeCard = $bindable() }: Props = $props();

	const cards: Card[] = [
		{
			id: 1,
			title: 'Open Graph Images',
			description: 'ogis is a fast, free, and beautiful platform for open graph image generation.',
			subtitle: 'Open Source • Always Free',
			logo: 'https://ogis.dev/logo-light.png'
		},
		{
			id: 2,
			title: 'Blog Post',
			description: 'How to build scalable web applications with modern frameworks',
			subtitle: 'Engineering • 12 min read'
		},
		{
			id: 3,
			title: 'Documentation',
			description: 'Complete API reference and integration guides for developers',
			subtitle: 'Developer Docs • v2.0'
		},
		{
			id: 4,
			title: 'Product Launch',
			description: 'Introducing our newest feature with enhanced performance',
			subtitle: 'Product Updates'
		},
		{
			id: 5,
			title: 'Tech Conference',
			description: 'Join developers worldwide for talks, workshops, and networking',
			subtitle: 'March 15-16 • Virtual'
		},
		{
			id: 6,
			title: 'Weekly Newsletter',
			description: 'The latest in web development, frameworks, and best practices',
			subtitle: 'Issue #47 • Feb 2025'
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
	class="relative w-full max-w-[500px] h-[350px] flex items-center justify-center"
	style="perspective: 1000px;"
	role="region"
	aria-label="Open Graph image examples"
	onmouseenter={() => isPaused = true}
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
			index={i}
			totalCards={3}
			isHovered={hoveredCard === i}
			isDimmed={hoveredCard !== null && hoveredCard !== i}
			verticalOffset={verticalOffset}
			onHover={() => hoveredCard = i}
			onLeave={() => hoveredCard = null}
		/>
	{/each}
</div>