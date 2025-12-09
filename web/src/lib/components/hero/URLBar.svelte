<script lang="ts">
	import { Copy, Check } from '@lucide/svelte';
	import { fly } from 'svelte/transition';
	import Simple from '$lib/assets/simple.png';

	type Props = {
		url: string;
		showLabel?: boolean;
	};

	let { url, showLabel = false }: Props = $props();

	let copied = $state(false);
	let editableParams = $state<Record<string, string>>({});

	// Parse URL into base and parameters
	const parsedUrl = $derived.by(() => {
		if (!url) return { base: '', params: {} };

		try {
			const urlObj = new URL(url);
			const params: Record<string, string> = {};

			urlObj.searchParams.forEach((value, key) => {
				params[key] = value;
			});

			return {
				base: `${urlObj.origin}${urlObj.pathname}`,
				params
			};
		} catch {
			return { base: url, params: {} };
		}
	});

	// Initialize editable params when URL changes
	$effect(() => {
		editableParams = { ...parsedUrl.params };
	});

	// Build full URL from editable params
	const fullUrl = $derived.by(() => {
		if (!parsedUrl.base) return '';

		const params = new URLSearchParams();
		Object.entries(editableParams).forEach(([key, value]) => {
			if (value) params.set(key, value);
		});

		const queryString = params.toString();
		return queryString ? `${parsedUrl.base}?${queryString}` : parsedUrl.base;
	});

	async function copyUrl() {
		if (!fullUrl) return;
		try {
			await navigator.clipboard.writeText(fullUrl);
			copied = true;
			setTimeout(() => {
				copied = false;
			}, 2000);
		} catch (err) {
			console.error('Failed to copy:', err);
		}
	}

	function handleInput(key: string, event: Event) {
		const target = event.target as HTMLInputElement;
		editableParams[key] = target.value;
	}
</script>

{#if url}
	<div>
		{#if showLabel}
			<div class="relative z-10 -mb-5">
				<img src={Simple} alt="it's this simple" width="200" />
			</div>
		{/if}
		<div
			class="relative w-full overflow-hidden rounded-2xl border border-border bg-muted/50 px-6 shadow-lg backdrop-blur-md"
		>
			<!-- Simple Text Above -->

			<div class="flex items-center justify-between gap-4">
				<!-- URL Display - Single Line -->
				<div
					class="scrollbar-thin scrollbar-track-transparent scrollbar-thumb-muted-foreground/20 flex min-w-0 flex-1 items-center gap-1 overflow-x-auto py-4 font-mono text-sm"
				>
					<!-- Base URL -->
					<span class="text-muted-foreground select-text">{parsedUrl.base}</span>
					{#if Object.keys(parsedUrl.params).length > 0}
						<span class="text-muted-foreground">?</span>
					{/if}

					<!-- Parameters Inline -->
					{#each Object.entries(parsedUrl.params) as [key, value], i (key)}
						<div class="flex items-center gap-1">
							<!-- Parameter Key -->
							<span class="text-blue-600 select-text dark:text-blue-400">{key}</span>
							<span class="text-muted-foreground">=</span>

							<!-- Editable Parameter Value with Fly -->
							{#key value}
								<span
									class="relative inline-block"
									style="min-width: {(editableParams[key] || value).length}ch;"
								>
									<span
										in:fly={{ x: -10, duration: 300 }}
										class="inline-block bg-emerald-500/10 whitespace-nowrap text-emerald-700 dark:text-emerald-300"
									>
										{value}
									</span>
									<input
										type="text"
										value={editableParams[key] || value}
										size={(editableParams[key] || value).length}
										oninput={(e) => handleInput(key, e)}
										class="absolute inset-0 m-0 border-0 bg-transparent p-0 whitespace-nowrap text-emerald-700 opacity-0 transition-colors focus:bg-emerald-500/20 focus:opacity-100 focus:outline-none dark:text-emerald-300"
										placeholder="Enter value..."
									/>
								</span>
							{/key}

							{#if i < Object.entries(parsedUrl.params).length - 1}
								<span class="text-muted-foreground">&</span>
							{/if}
						</div>
					{/each}
				</div>

				<!-- Copy Button -->
				<div class="flex-shrink-0">
					<button
						onclick={copyUrl}
						class="rounded-lg p-2 transition-all duration-200 hover:bg-accent"
						aria-label="Copy URL"
					>
						{#if copied}
							<Check class="h-5 w-5 text-green-600 dark:text-green-400" />
						{:else}
							<Copy class="h-5 w-5 text-muted-foreground" />
						{/if}
					</button>
				</div>
			</div>
		</div>
	</div>
{/if}
