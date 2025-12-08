<script lang="ts">
	import { playground } from '$lib/stores/playground.svelte';
	import LivePreview from './LivePreview.svelte';
	import SocialMockups from './SocialMockups.svelte';
	import URLBar from '$lib/components/hero/URLBar.svelte';
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
		<URLBar url={playground.apiUrl} />
	</div>

	<!-- Social Mockups -->
	<div class="space-y-3">
		<h3 class="text-sm font-medium">Social Preview</h3>
		<SocialMockups />
	</div>
</div>
