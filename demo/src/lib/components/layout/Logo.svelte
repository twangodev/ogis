<script lang="ts">
	import { CircleSlash2 } from '@lucide/svelte';
	import { site } from '$lib/config/site';

	type Props = {
		href?: string;
		showText?: boolean;
		size?: 'sm' | 'md' | 'lg';
	};

	let { href = '/', showText = true, size = 'md' }: Props = $props();

	const iconSizes = {
		sm: 'size-4',
		md: 'size-5',
		lg: 'size-6'
	};

	const textSizes = {
		sm: 'text-base',
		md: 'text-lg',
		lg: 'text-xl'
	};
</script>

<a
	{href}
	aria-label="home"
	class="flex items-center gap-2 text-foreground transition-opacity hover:opacity-80"
>
	<CircleSlash2 class={iconSizes[size]} />
	{#if showText}
		<span class="{textSizes[size]} font-semibold -translate-y-px">{site.name}</span>
	{/if}
</a>
