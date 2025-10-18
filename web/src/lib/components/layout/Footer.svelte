<script lang="ts">
	import Logo from './Logo.svelte';
	import { Link } from '$lib/components/ui/link';
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
		description?: string;
		linkColumns?: LinkColumn[];
		socialLinks?: SocialLink[];
	};

	let {
		description = site.description,
		linkColumns = [
			{
				title: 'Product',
				links: [
					{ name: 'Playground', href: '/playground' },
					{ name: 'Templates', href: '/templates' },
					{ name: 'Showcase', href: '/showcase' },
					{ name: 'API Reference', href: 'https://img.ogis.dev/docs' }
				]
			},
			{
				title: 'Resources',
				links: [
					{ name: 'Documentation', href: '/docs' },
					{ name: 'Self-Hosting Guide', href: '/docs/' },
					{ name: 'Status Page', href: 'https://kener.twango.dev/?group=ogis' },
					{ name: 'Changelog', href: `${site.github}/releases` }
				]
			},
			{
				title: 'Community',
				links: [
					{ name: 'GitHub', href: site.github },
					{ name: 'Report an Issue', href: `${site.github}/issues` },
					{ name: 'Contribute', href: `${site.github}/blob/main/CONTRIBUTING.md` },
					{ name: 'Discussions', href: `${site.github}/discussions` }
				]
			}
		],
		socialLinks = []
	}: Props = $props();

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
				Made by <Link href="https://twango.dev" external>James Ding</Link>. Licensed under{' '}
				<Link href="https://www.gnu.org/licenses/agpl-3.0.en.html" external variant="muted"
					>AGPL-v3</Link
				>.
			</p>

			<div class="flex items-center gap-4">
				{#each socialLinks as social (social.name)}
					<a
						href={social.href}
						aria-label={social.name}
						class="text-muted-foreground duration-150 hover:text-accent-foreground"
					>
						<span class="text-sm">{social.name}</span>
					</a>
				{/each}
				<a
					href="https://kener.twango.dev/?group=ogis"
					target="_blank"
					rel="noopener noreferrer"
					class="flex items-center gap-2 text-sm text-muted-foreground duration-150 hover:text-accent-foreground"
				>
					<img
						src="https://kener.twango.dev/badge/ogis/dot?animate=ping"
						alt="Status Badge"
						class="h-4 w-4"
					/>
					<span>View service status</span>
				</a>
			</div>
		</div>
	</div>
</footer>
