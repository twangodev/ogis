<script lang="ts">
	import { flip } from 'svelte/animate';
	import { cubicInOut } from 'svelte/easing';
	import { playground } from '$lib/stores/playground.svelte';
	import TemplateCard from './TemplateCard.svelte';
	import * as Dialog from '$lib/components/ui/dialog';
	import { Button } from '$lib/components/ui/button';
	import ChevronRightIcon from '@lucide/svelte/icons/chevron-right';
	import ShuffleIcon from '@lucide/svelte/icons/shuffle';

	let dialogOpen = $state(false);
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
		<Dialog.Content class="sm:max-w-7xl max-h-[85vh] overflow-y-auto">
			<Dialog.Header>
				<div class="flex items-center justify-between pr-8">
					<div>
						<Dialog.Title>All Templates</Dialog.Title>
						<Dialog.Description>Choose from {playground.templates.length} available templates</Dialog.Description>
					</div>
					<Button variant="ghost" size="icon" onclick={() => playground.shuffleTemplates()}>
						<ShuffleIcon class="size-4" />
					</Button>
				</div>
			</Dialog.Header>
			<div class="grid grid-cols-5 gap-4 mt-4">
				{#each playground.templates as template (template.name)}
					<div animate:flip={{ duration: 500, easing: cubicInOut }} class="min-w-0">
						<TemplateCard
							name={template.name}
							label={template.label}
							selected={playground.template === template.name}
							onclick={() => {
								playground.template = template.name;
								dialogOpen = false;
							}}
						/>
					</div>
				{/each}
			</div>
		</Dialog.Content>
	</Dialog.Root>
{/if}
