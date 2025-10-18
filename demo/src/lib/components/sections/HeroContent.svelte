<script lang="ts">
	import { Button } from '$lib/components/ui/button';

	type CTA = {
		text: string;
		href: string;
	};

	type Props = {
		title?: string;
		description?: string;
		primaryCta?: CTA;
		secondaryCta?: CTA;
	};

	let {
		title = 'Ship 10x Faster with NS',
		description = 'Highly customizable components for building modern websites and applications that look and feel the way you mean it.',
		primaryCta = { text: 'Start Building', href: '#' },
		secondaryCta = { text: 'Request a demo', href: '#' }
	}: Props = $props();
</script>

<section>
	<div class="pt-12 pb-24 md:pb-32 lg:pt-44 lg:pb-56">
		<div class="relative mx-auto flex max-w-6xl flex-col px-6 lg:block">
			<div class="mx-auto max-w-lg text-center lg:ml-0 lg:w-1/2 lg:text-left">
				<h1
					class="mt-8 max-w-2xl text-5xl font-medium text-balance md:text-6xl lg:mt-16 xl:text-7xl"
				>
					{title}
				</h1>
				<p class="mt-8 max-w-2xl text-lg text-pretty">
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
			</div>
		</div>
	</div>
</section>
