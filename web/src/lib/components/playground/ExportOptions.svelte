<script lang="ts">
	import { playground } from '$lib/stores/playground.svelte';
	import { Button } from '$lib/components/ui/button';
	import { DownloadIcon, CopyIcon, CodeIcon, CheckIcon } from '@lucide/svelte';

	let copiedUrl = $state(false);
	let copiedMeta = $state(false);
	let downloading = $state(false);

	// Generate HTML meta tags
	const metaTags = $derived(`<meta property="og:image" content="${playground.apiUrl}" />
<meta property="og:title" content="${playground.content.title}" />
<meta property="og:description" content="${playground.content.description}" />
<meta name="twitter:card" content="summary_large_image" />
<meta name="twitter:image" content="${playground.apiUrl}" />
<meta name="twitter:title" content="${playground.content.title}" />
<meta name="twitter:description" content="${playground.content.description}" />`);

	async function copyUrl() {
		try {
			await navigator.clipboard.writeText(playground.apiUrl);
			copiedUrl = true;
			setTimeout(() => (copiedUrl = false), 2000);
		} catch {
			fallbackCopy(playground.apiUrl);
			copiedUrl = true;
			setTimeout(() => (copiedUrl = false), 2000);
		}
	}

	async function copyMetaTags() {
		try {
			await navigator.clipboard.writeText(metaTags);
			copiedMeta = true;
			setTimeout(() => (copiedMeta = false), 2000);
		} catch {
			fallbackCopy(metaTags);
			copiedMeta = true;
			setTimeout(() => (copiedMeta = false), 2000);
		}
	}

	function fallbackCopy(text: string) {
		const input = document.createElement('textarea');
		input.value = text;
		document.body.appendChild(input);
		input.select();
		document.execCommand('copy');
		document.body.removeChild(input);
	}

	async function downloadImage() {
		downloading = true;
		try {
			const response = await fetch(playground.apiUrl);
			const blob = await response.blob();
			const url = URL.createObjectURL(blob);

			const a = document.createElement('a');
			a.href = url;
			a.download = `og-image-${playground.template}.png`;
			document.body.appendChild(a);
			a.click();
			document.body.removeChild(a);

			URL.revokeObjectURL(url);
		} catch (error) {
			console.error('Failed to download image:', error);
		} finally {
			downloading = false;
		}
	}
</script>

<div class="flex flex-wrap items-center gap-3 p-4 bg-muted/50 rounded-lg border border-border">
	<Button onclick={downloadImage} disabled={downloading}>
		<DownloadIcon class="size-4 mr-2" />
		{downloading ? 'Downloading...' : 'Download PNG'}
	</Button>

	<Button variant="outline" onclick={copyUrl}>
		{#if copiedUrl}
			<CheckIcon class="size-4 mr-2 text-green-500" />
			Copied!
		{:else}
			<CopyIcon class="size-4 mr-2" />
			Copy URL
		{/if}
	</Button>

	<Button variant="outline" onclick={copyMetaTags}>
		{#if copiedMeta}
			<CheckIcon class="size-4 mr-2 text-green-500" />
			Copied!
		{:else}
			<CodeIcon class="size-4 mr-2" />
			Copy Meta Tags
		{/if}
	</Button>
</div>
