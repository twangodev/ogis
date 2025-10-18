<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import ThemeToggle from './ThemeToggle.svelte';
	import Logo from './Logo.svelte';
	import { Menu, X, Github } from '@lucide/svelte';
	import { site } from '$lib/config/site';

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
			// { name: 'Features', href: '#features' },
			// { name: 'Solution', href: '#solution' },
			// { name: 'Pricing', href: '#pricing' },
			// { name: 'About', href: '#about' }
		],
		logoHref = '/'
	}: Props = $props();

	let menuState = $state(false);
</script>

<header>
	<nav class="fixed z-20 w-full border-b bg-background/50 backdrop-blur-3xl">
		<div class="mx-auto max-w-5xl px-6 transition-all duration-300">
			<div class="flex items-center justify-between py-3 lg:grid lg:grid-cols-3 lg:py-4">
				<!-- Logo - Left -->
				<div class="flex items-center">
					<Logo href={logoHref} />
				</div>

				<!-- Menu Items - Center (desktop only) -->
				<nav class={['hidden lg:flex lg:justify-center']}>
					<ul class="flex gap-8 text-sm">
						{#each menuItems as item (item.name)}
							<li>
								<a
									href={item.href}
									class="text-muted-foreground duration-150 hover:text-accent-foreground"
								>
									{item.name}
								</a>
							</li>
						{/each}
					</ul>
				</nav>

				<!-- Actions - Right (desktop) / Mobile menu button -->
				<div class="flex items-center justify-end gap-3">
					<div class="hidden items-center gap-3 lg:flex">
						<Button href={site.github} variant="ghost" size="icon" aria-label="GitHub">
							<Github class="size-5" />
						</Button>
						<ThemeToggle />
					</div>

					<!-- Mobile menu toggle -->
					<button
						onclick={() => (menuState = !menuState)}
						aria-label={menuState == true ? 'Close Menu' : 'Open Menu'}
						class="relative -m-2.5 p-2.5 lg:hidden"
					>
						<Menu
							class={['m-auto size-6 duration-200', menuState && 'scale-0 rotate-180 opacity-0']}
						/>
						<X
							class={[
								'absolute inset-0 m-auto size-6 scale-0 -rotate-90 opacity-0 duration-200',
								menuState && 'scale-100 rotate-0 opacity-100'
							]}
						/>
					</button>
				</div>
			</div>

			<!-- Mobile Menu -->
			<div
				class={[
					'mb-6 space-y-8 rounded-3xl border bg-background p-6 shadow-2xl shadow-zinc-300/20 lg:hidden dark:shadow-none',
					menuState ? 'block' : 'hidden'
				]}
			>
				<ul class="space-y-6 text-base">
					{#each menuItems as item (item.name)}
						<li>
							<a
								href={item.href}
								class="block text-muted-foreground duration-150 hover:text-accent-foreground"
							>
								{item.name}
							</a>
						</li>
					{/each}
				</ul>

				<div class="flex flex-col gap-3">
					<Button href={site.github} variant="outline" class="justify-start">
						<Github class="mr-2 size-5" />
						GitHub
					</Button>
					<ThemeToggle />
				</div>
			</div>
		</div>
	</nav>
</header>
