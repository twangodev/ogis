<script lang="ts">
	import { playground } from '$lib/stores/playground.svelte';
	import TemplateCard from './TemplateCard.svelte';
</script>

<div class="grid grid-cols-3 gap-2">
	{#each playground.templates as template (template.name)}
		<TemplateCard
			name={template.name}
			label={template.label}
			selected={playground.template === template.name}
			onclick={() => playground.template = template.name}
		/>
	{/each}
</div>
