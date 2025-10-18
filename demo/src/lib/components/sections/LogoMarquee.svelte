<script lang="ts">
	import Marquee from '$lib/components/magic/Marquee.svelte';
	import ProgressiveBlur from '$lib/components/magic/ProgressiveBlur.svelte';
	import { Link } from '$lib/components/ui/link';
	import DEFAULT_LOGOS from '$lib/config/logos.json';

	type Logo = {
		src: string;
		alt: string;
		height: number;
		href: string;
	};

	type Props = {
		logos?: Logo[];
		label?: string;
	};

	let {
		logos = DEFAULT_LOGOS,
		label = 'Powering open source, enterprise, and more.',
	}: Props = $props();
</script>

<section class="bg-background pb-16 md:pb-32">
	<div class="group relative m-auto max-w-6xl px-6">
		<div class="flex flex-col items-center md:flex-row">
			<div class="md:max-w-44 md:border-r md:pr-6">
				<p class="text-end text-sm">{label}</p>
			</div>
			<div class="relative py-6 md:w-[calc(100%-11rem)]">
				<Marquee repeat={8}>
					{#each logos as logo (logo.src)}
						<Link class="flex" href={logo.href} external>
							<img
								class="mx-auto w-fit dark:invert"
								style="height: {logo.height}px"
								src={logo.src}
								alt={logo.alt}
								height={logo.height}
								width="auto"
							/>
						</Link>
					{/each}
				</Marquee>

				<div class="absolute inset-y-0 left-0 w-20 bg-linear-to-r from-background"></div>
				<div class="absolute inset-y-0 right-0 w-20 bg-linear-to-l from-background"></div>

				<ProgressiveBlur
					class="pointer-events-none absolute left-0 top-0 h-full w-20"
					direction="left"
					blurIntensity={1}
				/>
				<ProgressiveBlur
					class="pointer-events-none absolute right-0 top-0 h-full w-20"
					direction="right"
					blurIntensity={1}
				/>
			</div>
		</div>
	</div>
</section>
