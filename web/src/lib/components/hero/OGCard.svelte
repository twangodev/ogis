<script lang="ts">
	type Props = {
		title: string;
		description: string;
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
		index,
		totalCards,
		isHovered,
		isDimmed,
		verticalOffset,
		onHover,
		onLeave
	}: Props = $props();

	// Build imageUrl from title and description
	const imageUrl = `https://img.ogis.dev?title=${encodeURIComponent(title)}&description=${encodeURIComponent(description)}`;

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
	class="absolute w-full aspect-[1.91] rounded-xl overflow-hidden cursor-pointer transition-all duration-300 ease-out top-1/2 left-1/2"
	class:shadow-lg={!isHovered}
	class:shadow-2xl={isHovered}
	class:opacity-40={shouldDim()}
	style="transform-origin: center center; transform: {getTransform()}; z-index: {getZIndex()};"
	onmouseenter={onHover}
	onmouseleave={onLeave}
	role="button"
	tabindex={index}
>
	<img src={imageUrl} alt={title} class="w-full h-full object-cover block" />
</div>
