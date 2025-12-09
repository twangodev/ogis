<script lang="ts">
	import autoAnimate from '@formkit/auto-animate';
	import fuzzysort from 'fuzzysort';
	import { playground } from '$lib/stores/playground.svelte';
	import TemplateCard from './TemplateCard.svelte';
	import * as Dialog from '$lib/components/ui/dialog';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import ChevronRightIcon from '@lucide/svelte/icons/chevron-right';
	import ShuffleIcon from '@lucide/svelte/icons/shuffle';
	import SearchIcon from '@lucide/svelte/icons/search';
	import FrownIcon from '@lucide/svelte/icons/frown';

	let dialogOpen = $state(false);
	let search = $state('');

	const filteredTemplates = $derived(
		search.trim()
			? fuzzysort.go(search, playground.templates, { keys: ['label', 'name'] }).map((r) => r.obj)
			: playground.templates
	);

	$effect(() => {
		if (!dialogOpen) search = '';
	});
</script>

<!-- First 5 rows (15 templates) always visible -->
<div class="grid grid-cols-3 gap-2">
	{#each playground.templates.slice(0, 15) as template (template.name)}
		<TemplateCard
			name={template.name}
			label={template.label}
			selected={playground.template === template.name}
			onclick={() => (playground.template = template.name)}
		/>
	{/each}
</div>

<!-- Show more button opens dialog -->
{#if playground.templates.length > 15}
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
						<Dialog.Description
							>Choose from {playground.templates.length} available templates</Dialog.Description
						>
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
