<script lang="ts">
	import { playground } from '$lib/stores/playground.svelte';
	import { Input } from '$lib/components/ui/input';
	import { Textarea } from '$lib/components/ui/textarea';
	import { Label } from '$lib/components/ui/label';

	const MAX_LENGTH = 1000;

	function handleTitleInput(e: Event) {
		const target = e.target as HTMLInputElement;
		playground.updateContent({ title: target.value });
	}

	function handleSubtitleInput(e: Event) {
		const target = e.target as HTMLInputElement;
		playground.updateContent({ subtitle: target.value });
	}

	function handleDescriptionInput(e: Event) {
		const target = e.target as HTMLTextAreaElement;
		playground.updateContent({ description: target.value });
	}
</script>

<div class="space-y-4">
	<!-- Title -->
	<div class="space-y-2">
		<div class="flex items-center justify-between">
			<Label for="title">Title</Label>
			<span class="text-xs text-muted-foreground">
				{playground.content.title.length}/{MAX_LENGTH}
			</span>
		</div>
		<Input
			id="title"
			type="text"
			placeholder="Enter your title"
			value={playground.content.title}
			oninput={handleTitleInput}
			maxlength={MAX_LENGTH}
		/>
	</div>

	<!-- Subtitle -->
	<div class="space-y-2">
		<div class="flex items-center justify-between">
			<Label for="subtitle">Subtitle</Label>
			<span class="text-xs text-muted-foreground">
				{playground.content.subtitle.length}/{MAX_LENGTH}
			</span>
		</div>
		<Input
			id="subtitle"
			type="text"
			placeholder="Enter a subtitle (appears above title)"
			value={playground.content.subtitle}
			oninput={handleSubtitleInput}
			maxlength={MAX_LENGTH}
		/>
	</div>

	<!-- Description -->
	<div class="space-y-2">
		<div class="flex items-center justify-between">
			<Label for="description">Description</Label>
			<span class="text-xs text-muted-foreground">
				{playground.content.description.length}/{MAX_LENGTH}
			</span>
		</div>
		<Textarea
			id="description"
			placeholder="Enter your description"
			value={playground.content.description}
			oninput={handleDescriptionInput}
			maxlength={MAX_LENGTH}
			rows={3}
		/>
	</div>
</div>
