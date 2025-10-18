<script lang="ts">
	import Marquee from '$lib/components/magic/Marquee.svelte';
	import ProgressiveBlur from '$lib/components/magic/ProgressiveBlur.svelte';

	type Logo = {
		src: string;
		alt: string;
		height: number;
	};

	type Props = {
		logos?: Logo[];
		label?: string;
	};

	let {
		logos = [
			{
				src: 'https://html.tailus.io/blocks/customers/nvidia.svg',
				alt: 'Nvidia Logo',
				height: 20
			},
			{
				src: 'https://html.tailus.io/blocks/customers/column.svg',
				alt: 'Column Logo',
				height: 16
			},
			{
				src: 'https://html.tailus.io/blocks/customers/github.svg',
				alt: 'GitHub Logo',
				height: 16
			},
			{ src: 'https://html.tailus.io/blocks/customers/nike.svg', alt: 'Nike Logo', height: 20 },
			{
				src: 'https://html.tailus.io/blocks/customers/lemonsqueezy.svg',
				alt: 'Lemon Squeezy Logo',
				height: 20
			},
			{
				src: 'https://html.tailus.io/blocks/customers/laravel.svg',
				alt: 'Laravel Logo',
				height: 16
			},
			{ src: 'https://html.tailus.io/blocks/customers/lilly.svg', alt: 'Lilly Logo', height: 28 },
			{
				src: 'https://html.tailus.io/blocks/customers/openai.svg',
				alt: 'OpenAI Logo',
				height: 24
			}
		],
		label = 'Powering the best teams'
	}: Props = $props();
</script>

<section class="bg-background pb-16 md:pb-32">
	<div class="group relative m-auto max-w-6xl px-6">
		<div class="flex flex-col items-center md:flex-row">
			<div class="md:max-w-44 md:border-r md:pr-6">
				<p class="text-end text-sm">{label}</p>
			</div>
			<div class="relative py-6 md:w-[calc(100%-11rem)]">
				<Marquee>
					{#each logos as logo (logo.src)}
						<div class="flex">
							<img
								class="mx-auto w-fit dark:invert"
								style="height: {logo.height}px"
								src={logo.src}
								alt={logo.alt}
								height={logo.height}
								width="auto"
							/>
						</div>
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
