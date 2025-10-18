<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import ThemeToggle from './ThemeToggle.svelte';
	import { Menu, X, CircleSlash2 } from '@lucide/svelte';

	type MenuItem = {
		name: string;
		href: string;
	};

	type Props = {
		menuItems?: MenuItem[];
		logoHref?: string;
	};

	let {
		menuItems = [
			{ name: 'Features', href: '#features' },
			{ name: 'Solution', href: '#solution' },
			{ name: 'Pricing', href: '#pricing' },
			{ name: 'About', href: '#about' }
		],
		logoHref = '/'
	}: Props = $props();

	let menuState = $state(false);
</script>

<header>
	<nav class="fixed z-20 w-full border-b bg-background/50 backdrop-blur-3xl">
		<div class="mx-auto max-w-5xl px-6 transition-all duration-300">
			<div
				class="flex flex-wrap items-center justify-between gap-6 py-3 lg:flex-nowrap lg:gap-6 lg:py-4"
			>
				<div class="flex w-full items-center justify-between lg:w-auto">
					<a href={logoHref} aria-label="home" class="flex items-center space-x-2">
						<CircleSlash2 />
						<span>ogis</span>
					</a>

					<button
						onclick={() => (menuState = !menuState)}
						aria-label={menuState == true ? 'Close Menu' : 'Open Menu'}
						class="relative z-20 -m-2.5 -mr-4 block cursor-pointer p-2.5 lg:hidden"
					>
						<Menu
							class={['m-auto size-6 duration-200', menuState && 'scale-0 rotate-180 opacity-0']}
						/>
						<X
							class={[
								'absolute inset-0 m-auto size-6 scale-0 -rotate-180 opacity-0 duration-200',
								menuState && 'scale-100 rotate-0 opacity-100'
							]}
						/>
					</button>
				</div>

				<div
					class={[
						'mb-6 w-full flex-wrap items-center justify-end space-y-8 rounded-3xl border bg-background p-6 shadow-2xl shadow-zinc-300/20 sm:justify-between md:flex-nowrap lg:m-0 lg:flex  lg:gap-6 lg:space-y-0 lg:border-transparent lg:bg-transparent lg:p-0 lg:shadow-none dark:shadow-none dark:lg:bg-transparent',
						menuState ? 'block lg:flex' : 'hidden'
					]}
				>
					<div class="lg:pr-4">
						<ul class="space-y-6 text-base lg:flex lg:gap-8 lg:space-y-0 lg:text-sm">
							{#each menuItems as item (item.name)}
								<li>
									<a
										href={item.href}
										class="block text-muted-foreground duration-150 hover:text-accent-foreground"
									>
										<span>{item.name}</span>
									</a>
								</li>
							{/each}
						</ul>
					</div>

					<div class="flex w-full flex-col space-y-3 sm:flex-row sm:items-center sm:gap-3 sm:space-y-0 md:w-fit">
						<ThemeToggle />
						<Button variant="outline" size="sm">Login</Button>
						<Button size="sm">Sign Up</Button>
					</div>
				</div>
			</div>
		</div>
	</nav>
</header>
