<script lang="ts">
	import OGCard from './OGCard.svelte';

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

	// Randomly select 3 cards from the pool
	function getRandomCards(count: number = 3): Card[] {
		const shuffled = [...cards].sort(() => Math.random() - 0.5);
		return shuffled.slice(0, count);
	}

	const displayedCards = getRandomCards(3);
	let hoveredCard = $state<number | null>(null);

	// Calculate vertical offset to center the whole stack
	// With 3 cards at 80px spacing, the spread is (2 * 80) = 160px
	// Offset by half to center: 80px
	const verticalOffset = ((displayedCards.length - 1) * 80) / 2;
</script>

<div class="relative w-full max-w-[500px] h-[350px] flex items-center justify-center" style="perspective: 1000px;">
	{#each displayedCards as card, i (card.id)}
		<OGCard
			title={card.title}
			description={card.description}
			index={i}
			totalCards={displayedCards.length}
			isHovered={hoveredCard === i}
			isDimmed={hoveredCard !== null && hoveredCard !== i}
			verticalOffset={verticalOffset}
			onHover={() => hoveredCard = i}
			onLeave={() => hoveredCard = null}
		/>
	{/each}
</div>