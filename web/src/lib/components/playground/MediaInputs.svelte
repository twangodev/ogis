<script lang="ts">
	import { playground } from '$lib/stores/playground.svelte';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { ImageIcon, Building2Icon } from '@lucide/svelte';

	function handleLogoInput(e: Event) {
		const target = e.target as HTMLInputElement;
		playground.updateMedia({ logo: target.value });
	}

	function handleImageInput(e: Event) {
		const target = e.target as HTMLInputElement;
		playground.updateMedia({ image: target.value });
	}

	// Simple URL validation
	function isValidUrl(url: string): boolean {
		if (!url) return true;
		try {
			new URL(url);
			return true;
		} catch {
			return false;
		}
	}

	const logoValid = $derived(isValidUrl(playground.media.logo));
	const imageValid = $derived(isValidUrl(playground.media.image));
</script>

<div class="space-y-4">
	<!-- Logo URL -->
	<div class="space-y-2">
		<Label for="logo" class="flex items-center gap-2">
			<Building2Icon class="size-4" />
			Logo URL
		</Label>
		<Input
			id="logo"
			type="url"
			placeholder="https://example.com/logo.png"
			value={playground.media.logo}
			oninput={handleLogoInput}
			class={!logoValid ? 'border-destructive' : ''}
		/>
		{#if !logoValid}
			<p class="text-xs text-destructive">Please enter a valid URL</p>
		{/if}
		<p class="text-xs text-muted-foreground">Custom logo image (HTTPS only, max 5MB)</p>
	</div>

	<!-- Image URL -->
	<div class="space-y-2">
		<Label for="image" class="flex items-center gap-2">
			<ImageIcon class="size-4" />
			Image URL
		</Label>
		<Input
			id="image"
			type="url"
			placeholder="https://example.com/photo.jpg"
			value={playground.media.image}
			oninput={handleImageInput}
			class={!imageValid ? 'border-destructive' : ''}
		/>
		{#if !imageValid}
			<p class="text-xs text-destructive">Please enter a valid URL</p>
		{/if}
		<p class="text-xs text-muted-foreground">Custom image or photo (HTTPS only, max 5MB)</p>
	</div>
</div>
