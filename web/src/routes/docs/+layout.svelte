<script lang="ts">
	import { page } from '$app/stores';
	import * as ScrollArea from '$lib/components/ui/scroll-area';
	import { Separator } from '$lib/components/ui/separator';
	import { Home, Rocket, Code, Settings, Lock } from '@lucide/svelte';
	import navigationData from './navigation.json';

	let { children } = $props();

	const iconMap: Record<string, any> = {
		home: Home,
		rocket: Rocket,
		code: Code,
		settings: Settings,
		lock: Lock
	};

	const navigation = navigationData.map((item) => ({
		...item,
		icon: item.icon ? iconMap[item.icon] : undefined,
		items: item.items?.map((subItem: any) => ({
			...subItem,
			icon: subItem.icon ? iconMap[subItem.icon] : undefined
		}))
	}));

	function isActive(href: string) {
		return $page.url.pathname === href;
	}
</script>

<div class="container mx-auto">
	<div class="flex gap-10 py-10">
		<aside class="hidden md:block w-64 shrink-0">
			<div class="sticky top-20">
				<ScrollArea.Root class="h-[calc(100vh-8rem)] pr-6">
					<div class="space-y-6">
						<div>
							<h4 class="mb-2 text-sm font-semibold">Documentation</h4>
							<Separator class="mb-4" />
							<nav class="space-y-1">
								{#each navigation as item}
									{#if item.items && item.items.length > 0}
										<div class="space-y-1">
											<a
												href={item.href}
												class="flex items-center gap-2 rounded-md px-3 py-2 text-sm transition-colors {isActive(
													item.href
												)
													? 'bg-secondary text-secondary-foreground font-medium'
													: 'text-muted-foreground hover:text-foreground hover:bg-secondary/50'}"
											>
												{#if item.icon}
													<item.icon size={16} />
												{/if}
												{item.title}
											</a>
											<div class="ml-4 space-y-1 border-l pl-3">
												{#each item.items as subItem}
													<a
														href={subItem.href}
														class="flex items-center gap-2 rounded-md px-3 py-2 text-sm transition-colors {isActive(
															subItem.href
														)
															? 'bg-secondary text-secondary-foreground font-medium'
															: 'text-muted-foreground hover:text-foreground hover:bg-secondary/50'}"
													>
														{#if subItem.icon}
															<subItem.icon size={16} />
														{/if}
														{subItem.title}
													</a>
												{/each}
											</div>
										</div>
									{:else}
										<a
											href={item.href}
											class="flex items-center gap-2 rounded-md px-3 py-2 text-sm transition-colors {isActive(
												item.href
											)
												? 'bg-secondary text-secondary-foreground font-medium'
												: 'text-muted-foreground hover:text-foreground hover:bg-secondary/50'}"
										>
											{#if item.icon}
												<item.icon size={16} />
											{/if}
											{item.title}
										</a>
									{/if}
								{/each}
							</nav>
						</div>
					</div>
				</ScrollArea.Root>
			</div>
		</aside>

		<main class="flex-1 min-w-0">
			{@render children?.()}
		</main>
	</div>
</div>