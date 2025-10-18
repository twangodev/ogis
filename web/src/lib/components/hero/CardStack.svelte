<script lang="ts">
	import OGCard from './OGCard.svelte';
	import { onMount } from 'svelte';

	type Card = {
		id: number;
		title: string;
		description: string;
	};

	const cards: Card[] = [
		{
			id: 1,
			title: 'Dynamic Blog Post',
			description: 'Generate beautiful OG images for your blog posts'
		},
		{
			id: 2,
			title: 'Product Launch',
			description: 'Showcase your products with custom OG images'
		},
		{
			id: 3,
			title: 'Event Announcement',
			description: 'Create engaging event cards automatically'
		},
		{
			id: 4,
			title: 'Documentation Page',
			description: 'Professional docs with branded images'
		},
		{
			id: 5,
			title: 'Portfolio Project',
			description: 'Showcase your work with stunning previews'
		},
		{
			id: 6,
			title: 'Online Course',
			description: 'Eye-catching course thumbnails'
		},
		{
			id: 7,
			title: 'Newsletter Issue',
			description: 'Share-worthy newsletter graphics'
		},
		{
			id: 8,
			title: 'Podcast Episode',
			description: 'Dynamic episode artwork'
		},
		{
			id: 9,
			title: 'Case Study',
			description: 'Professional case study previews'
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