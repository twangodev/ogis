<script lang="ts" module>
	import type { WithElementRef } from 'bits-ui';
	import type { HTMLAnchorAttributes } from 'svelte/elements';
	import { type VariantProps, tv } from 'tailwind-variants';

	export const linkVariants = tv({
		base: 'transition-colors inline-flex items-center gap-1',
		variants: {
			variant: {
				default: 'text-foreground hover:text-accent-foreground',
				muted: 'text-muted-foreground hover:text-foreground',
				accent: 'text-accent-foreground hover:text-primary',
				primary: 'text-primary hover:text-primary/80',
				subtle: 'text-foreground hover:text-accent-foreground'
			}
		},
		defaultVariants: {
			variant: 'default'
		}
	});

	export type LinkVariant = VariantProps<typeof linkVariants>['variant'];

	export type LinkProps = WithElementRef<HTMLAnchorAttributes> & {
		variant?: LinkVariant;
		external?: boolean;
	};
</script>

<script lang="ts">
	import { cn } from '$lib/utils';

	let {
		class: className,
		variant = 'default',
		external = false,
		href,
		target = external ? '_blank' : undefined,
		rel = undefined,
		ref = $bindable(null),
		children,
		...restProps
	}: LinkProps = $props();
</script>

<a
	bind:this={ref}
	class={cn(linkVariants({ variant }), className)}
	{href}
	{target}
	{rel}
	{...restProps}
>
	{@render children?.()}
</a>