<script lang="ts">
	import autoAnimate from '@formkit/auto-animate';
	import fuzzysort from 'fuzzysort';
	import { playground } from '$lib/stores/playground.svelte';
	import TemplateCard from './TemplateCard.svelte';
	import * as Dialog from '$lib/components/ui/dialog';
	import * as Select from '$lib/components/ui/select';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import ChevronRightIcon from '@lucide/svelte/icons/chevron-right';
	import ShuffleIcon from '@lucide/svelte/icons/shuffle';
	import SearchIcon from '@lucide/svelte/icons/search';
	import FrownIcon from '@lucide/svelte/icons/frown';

	let dialogOpen = $state(false);
	let search = $state('');
	let selectedLayout = $state<string>('all');

	function formatLayoutName(layout: string): string {
		return layout.replace(/-/g, ' ').replace(/\b\w/g, (c) => c.toUpperCase());
	}

	const layoutOptions = $derived.by(() => {
		const counts = new Map<string, number>();
		for (const t of playground.templates) {
			if (t.layout) {
				counts.set(t.layout, (counts.get(t.layout) ?? 0) + 1);
			}
		}
		return Array.from(counts.entries())
			.sort((a, b) => b[1] - a[1])
			.map(([layout, count]) => ({
				value: layout,
				label: formatLayoutName(layout),
				count
			}));
	});

	const layoutFilteredTemplates = $derived(
		selectedLayout === 'all'
			? playground.templates
			: playground.templates.filter((t) => t.layout === selectedLayout)
	);

	const filteredTemplates = $derived(
		search.trim()
			? fuzzysort.go(search, layoutFilteredTemplates, { keys: ['label', 'name'] }).map((r) => r.obj)
			: layoutFilteredTemplates
	);

	const selectedLayoutLabel = $derived(
		selectedLayout === 'all'
			? `All Layouts (${playground.templates.length})`
			: `${formatLayoutName(selectedLayout)} (${layoutFilteredTemplates.length})`
	);

	$effect(() => {
		if (!dialogOpen) search = '';
	});
</script>

<!-- Layout filter -->
<div class="mb-2">
	<Select.Root type="single" bind:value={selectedLayout}>
		<Select.Trigger class="w-full">
			{selectedLayoutLabel}
		</Select.Trigger>
		<Select.Content>
			<Select.Item value="all" label="All Layouts ({playground.templates.length})">
				All Layouts ({playground.templates.length})
			</Select.Item>
			{#each layoutOptions as opt (opt.value)}
				<Select.Item value={opt.value} label="{opt.label} ({opt.count})">
					{opt.label} ({opt.count})
				</Select.Item>
			{/each}
		</Select.Content>
	</Select.Root>
</div>

<!-- First 5 rows (15 templates) always visible -->
<div class="grid grid-cols-3 gap-2">
	{#each layoutFilteredTemplates.slice(0, 15) as template (template.name)}
		<TemplateCard
			name={template.name}
			label={template.label}
			selected={playground.template === template.name}
			onclick={() => (playground.template = template.name)}
		/>
	{/each}
</div>

<!-- Show more button opens dialog -->
{#if layoutFilteredTemplates.length > 15}
	<Dialog.Root bind:open={dialogOpen}>
		<Dialog.Trigger
			class="flex w-full items-center justify-center gap-1 py-2 text-sm text-muted-foreground transition-colors hover:text-foreground"
		>
			<span>Show more</span>
			<ChevronRightIcon class="size-4" />
		</Dialog.Trigger>
		<Dialog.Content class="max-h-[85vh] overflow-y-auto sm:max-w-7xl">
			<Dialog.Header>
				<div class="flex items-center justify-between pr-8">
					<div>
						<Dialog.Title>All Templates</Dialog.Title>
						<Dialog.Description>
							{selectedLayout === 'all'
								? `Choose from ${playground.templates.length} available templates`
								: `${layoutFilteredTemplates.length} templates with ${formatLayoutName(selectedLayout)} layout`}
						</Dialog.Description>
					</div>
					<Button variant="ghost" size="icon" onclick={() => playground.shuffleTemplates()}>
						<ShuffleIcon class="size-4" />
					</Button>
				</div>
			</Dialog.Header>
			<div class="relative mt-4">
				<SearchIcon class="absolute top-1/2 left-3 size-4 -translate-y-1/2 text-muted-foreground" />
				<Input type="text" placeholder="Search templates..." class="pl-9" bind:value={search} />
			</div>
			{#if filteredTemplates.length > 0}
				<div
					use:autoAnimate={{ duration: 300, easing: 'ease-in-out' }}
					class="mt-4 grid grid-cols-5 gap-4 overflow-hidden"
				>
					{#each filteredTemplates as template (template.name)}
						<TemplateCard
							name={template.name}
							label={template.label}
							selected={playground.template === template.name}
							onclick={() => {
								playground.template = template.name;
								dialogOpen = false;
							}}
						/>
					{/each}
				</div>
			{:else}
				<div class="flex flex-col items-center justify-center py-12 text-muted-foreground">
					<FrownIcon class="mb-2 size-8 opacity-50" />
					<p>No templates found</p>
				</div>
			{/if}
		</Dialog.Content>
	</Dialog.Root>
{/if}
