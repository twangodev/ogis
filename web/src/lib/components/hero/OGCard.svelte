<script lang="ts">
	import { apiConfig } from '$lib/config/api.svelte';

	type Props = {
		title: string;
		description: string;
		subtitle?: string;
		logo?: string;
		image?: string;
		template?: string;
		params?: Record<string, string>;
		index: number;
		totalCards: number;
		isHovered: boolean;
		isDimmed: boolean;
		verticalOffset: number;
		onHover: () => void;
		onLeave: () => void;
	};

	let {
		title,
		description,
		subtitle,
		logo,
		image,
		template,
		params,
		index,
		totalCards,
		isHovered,
		isDimmed,
		verticalOffset,
		onHover,
		onLeave
	}: Props = $props();

	// Build imageUrl from all available parameters
	const imageUrl = $derived.by(() => {
		return apiConfig.generateUrl({
			title,
			description,
			subtitle,
			logo,
			image,
			template,
			...params,
			format: 'webp',
			scale: '0.5',
			quality: '80'
		});
	});

	function getTransform() {
		// Calculate tilt from center (middle card is 0, sides tilt opposite directions)
		const tiltFromCenter = (index - 1) * 5; // Middle card (index 1) = 0deg, others tilt ±5deg
		// Apply vertical offset to center the whole stack
		const yPos = index * -80 + verticalOffset;
		const middleIndex = (totalCards - 1) / 2;
		const isMiddle = index === middleIndex;

		if (isHovered) {
			return `translate(-50%, -50%) translateX(${index * 80}px) translateY(${yPos}px) translateZ(50px) rotateY(0deg) rotateX(0deg) rotateZ(0deg) scale(1.1)`;
		} else if (isDimmed || (!isHovered && !isMiddle)) {
			// Dimmed cards or non-middle cards when nothing is hovered
			return `translate(-50%, -50%) translateX(${index * 80}px) translateY(${yPos}px) translateZ(${index * -40}px) rotateY(${index * -2}deg) rotateX(${index * 2}deg) rotateZ(${tiltFromCenter}deg)`;
		} else if (isMiddle) {
			// Middle card gets 50% of hover effect by default
			return `translate(-50%, -50%) translateX(${index * 80}px) translateY(${yPos}px) translateZ(25px) rotateY(0deg) rotateX(0deg) rotateZ(0deg) scale(1.05)`;
		} else {
			return `translate(-50%, -50%) translateX(${index * 80}px) translateY(${yPos}px) translateZ(${index * -20}px) rotateY(${index * -2}deg) rotateX(${index * 2}deg) rotateZ(${tiltFromCenter}deg)`;
		}
	}

	function getZIndex() {
		if (isHovered) return 100;

		// Middle card should be on top
		const middleIndex = (totalCards - 1) / 2;
		const distanceFromMiddle = Math.abs(index - middleIndex);

		// Base z-index on distance from middle (closer = higher)
		// If equidistant, left side gets priority
		const baseZ = 10 - distanceFromMiddle * 2;
		const penalty = index > middleIndex ? 1 : 0;

		return baseZ - penalty;
	}

	function shouldDim() {
		const middleIndex = (totalCards - 1) / 2;
		const isMiddle = index === middleIndex;
		// Dim if explicitly dimmed OR if not middle card and nothing is hovered
		return isDimmed || (!isMiddle && !isHovered);
	}
</script>

<div
	class="absolute top-1/2 left-1/2 aspect-[1.91] w-full cursor-pointer overflow-hidden rounded-xl transition-all ease-out"
	class:duration-200={isHovered || isDimmed}
	class:duration-700={!isHovered && !isDimmed}
	class:shadow-lg={!isHovered}
	class:shadow-2xl={isHovered}
	class:opacity-40={shouldDim()}
	style="transform-origin: center center; transform: {getTransform()}; z-index: {getZIndex()};"
	onmouseenter={onHover}
	onmouseleave={onLeave}
	role="button"
	tabindex={index}
>
	<img src={imageUrl} alt={title} class="block h-full w-full object-cover" />
</div>
