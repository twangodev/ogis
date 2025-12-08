<script lang="ts">
	import { playground } from '$lib/stores/playground.svelte';
	import LivePreview from './LivePreview.svelte';
	import SocialMockups from './SocialMockups.svelte';
	import { Button } from '$lib/components/ui/button';
	import { CopyIcon, CheckIcon } from '@lucide/svelte';

	let copied = $state(false);

	async function copyUrl() {
		try {
			await navigator.clipboard.writeText(playground.apiUrl);
			copied = true;
			setTimeout(() => (copied = false), 2000);
		} catch {
			// Fallback for older browsers
			const input = document.createElement('input');
			input.value = playground.apiUrl;
			document.body.appendChild(input);
			input.select();
			document.execCommand('copy');
			document.body.removeChild(input);
			copied = true;
			setTimeout(() => (copied = false), 2000);
		}
	}
</script>

<div class="space-y-6">
	<!-- Live Preview -->
	<div class="space-y-3">
		<h2 class="text-sm font-medium">Preview</h2>
		<LivePreview />
	</div>

	<!-- URL Bar -->
	<div class="space-y-2">
		<h3 class="text-sm font-medium">Generated URL</h3>
		<div
			class="flex items-center gap-2 p-3 bg-muted rounded-lg border border-border overflow-hidden"
		>
			<code class="flex-1 text-xs text-muted-foreground truncate font-mono">
				{playground.apiUrl}
			</code>
			<Button variant="ghost" size="sm" onclick={copyUrl} class="shrink-0">
				{#if copied}
					<CheckIcon class="size-4 text-green-500" />
				{:else}
					<CopyIcon class="size-4" />
				{/if}
			</Button>
		</div>
	</div>

	<!-- Social Mockups -->
	<div class="space-y-3">
		<h3 class="text-sm font-medium">Social Preview</h3>
		<SocialMockups />
	</div>
</div>
