<script lang="ts">
	import { Copy, Check } from '@lucide/svelte';
	import { typewriter } from '$lib/transitions';
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
				params[key] = decodeURIComponent(value);
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
	<div class="mt-16">
		{#if showLabel}
			<div class="relative z-10 -mb-5">
				<img src={Simple} alt="it's this simple" width="200" />
			</div>
		{/if}
		<div class="relative w-full rounded-2xl bg-muted/50 backdrop-blur-md border border-border px-6 shadow-lg overflow-hidden">
			<!-- Simple Text Above -->

			<div class="flex items-center justify-between gap-4">
				<!-- URL Display - Single Line -->
				<div class="flex-1 min-w-0 font-mono text-sm py-4 flex items-center gap-1 overflow-x-auto scrollbar-thin scrollbar-track-transparent scrollbar-thumb-muted-foreground/20">
					<!-- Base URL -->
					<span class="text-muted-foreground select-text">{parsedUrl.base}</span>
					{#if Object.keys(parsedUrl.params).length > 0}
						<span class="text-muted-foreground">?</span>
					{/if}

					<!-- Parameters Inline -->
					{#each Object.entries(parsedUrl.params) as [key, value], i (key)}
						<div class="flex items-center gap-1">
							<!-- Parameter Key -->
							<span class="text-blue-600 dark:text-blue-400 select-text">{key}</span>
							<span class="text-muted-foreground">=</span>

							<!-- Editable Parameter Value with Typewriter -->
							{#key value}
								<span class="relative inline-block">
									<span
										in:typewriter={{ speed: 3 }}
										class="bg-emerald-500/10 text-emerald-700 dark:text-emerald-300 inline-block"
									>
										{value}
									</span>
									<input
										type="text"
										value={editableParams[key] || value}
										size={Math.max(10, (editableParams[key] || value).length || 10)}
										oninput={(e) => handleInput(key, e)}
										class="absolute inset-0 bg-transparent text-emerald-700 dark:text-emerald-300 focus:outline-none focus:bg-emerald-500/20 transition-colors p-0 m-0 border-0 opacity-0 focus:opacity-100"
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
						class="p-2 rounded-lg hover:bg-accent transition-all duration-200"
						aria-label="Copy URL"
					>
						{#if copied}
							<Check class="w-5 h-5 text-green-600 dark:text-green-400" />
						{:else}
							<Copy class="w-5 h-5 text-muted-foreground" />
						{/if}
					</button>
				</div>
			</div>
		</div>
	</div>
{/if}
