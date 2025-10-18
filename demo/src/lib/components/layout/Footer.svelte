<script lang="ts">
	import Logo from './Logo.svelte';
	import { site } from '$lib/config/site';

	type LinkColumn = {
		title: string;
		links: Array<{ name: string; href: string }>;
	};

	type SocialLink = {
		name: string;
		href: string;
		icon: string;
	};

	type Props = {
		companyName?: string;
		description?: string;
		linkColumns?: LinkColumn[];
		socialLinks?: SocialLink[];
	};

	let {
		companyName = site.name,
		description = site.description,
		linkColumns = [
			{
				title: 'Product',
				links: [
					{ name: 'Features', href: '#features' },
					{ name: 'Pricing', href: '#pricing' },
					{ name: 'Updates', href: '#updates' }
				]
			},
			{
				title: 'Company',
				links: [
					{ name: 'About', href: '#about' },
					{ name: 'Blog', href: '#blog' },
					{ name: 'Careers', href: '#careers' }
				]
			},
			{
				title: 'Resources',
				links: [
					{ name: 'Documentation', href: '#docs' },
					{ name: 'Support', href: '#support' },
					{ name: 'Contact', href: '#contact' }
				]
			}
		],
		socialLinks = [
			{ name: 'Twitter', href: '#', icon: 'twitter' },
			{ name: 'GitHub', href: '#', icon: 'github' },
			{ name: 'LinkedIn', href: '#', icon: 'linkedin' }
		]
	}: Props = $props();

	const currentYear = new Date().getFullYear();
</script>

<footer class="border-t bg-background">
	<div class="mx-auto max-w-6xl px-6 py-12 md:py-16">
		<div class="grid gap-8 md:grid-cols-2 lg:grid-cols-12">
			<!-- Company Info -->
			<div class="lg:col-span-4">
				<Logo />
				<p class="mt-4 text-sm text-muted-foreground">
					{description}
				</p>
			</div>

			<!-- Link Columns -->
			{#each linkColumns as column (column.title)}
				<div class="lg:col-span-2">
					<h3 class="text-sm font-semibold">{column.title}</h3>
					<ul class="mt-4 space-y-3">
						{#each column.links as link (link.name)}
							<li>
								<a
									href={link.href}
									class="text-sm text-muted-foreground duration-150 hover:text-accent-foreground"
								>
									{link.name}
								</a>
							</li>
						{/each}
					</ul>
				</div>
			{/each}
		</div>

		<!-- Bottom Bar -->
		<div
			class="mt-12 flex flex-col items-center justify-between gap-4 border-t pt-8 sm:flex-row"
		>
			<p class="text-sm text-muted-foreground">
				&copy; {currentYear} {companyName}. All rights reserved.
			</p>

			<div class="flex gap-4">
				{#each socialLinks as social (social.name)}
					<a
						href={social.href}
						aria-label={social.name}
						class="text-muted-foreground duration-150 hover:text-accent-foreground"
					>
						<span class="text-sm">{social.name}</span>
					</a>
				{/each}
			</div>
		</div>
	</div>
</footer>
