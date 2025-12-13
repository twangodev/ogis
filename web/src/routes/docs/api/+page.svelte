<script lang="ts">
	import { onMount } from 'svelte';

	let swaggerLoaded = $state(false);
	let swaggerError = $state<string | null>(null);

	onMount(() => {
		// Load Swagger UI CSS
		const link = document.createElement('link');
		link.rel = 'stylesheet';
		link.href = 'https://unpkg.com/swagger-ui-dist@5/swagger-ui.css';
		document.head.appendChild(link);

		// Load Swagger UI JS
		const script = document.createElement('script');
		script.src = 'https://unpkg.com/swagger-ui-dist@5/swagger-ui-bundle.js';
		script.onload = () => {
			try {
				// Initialize Swagger UI
				(window as any).SwaggerUIBundle({
					url: 'https://img.ogis.dev/docs/openapi.json',
					dom_id: '#swagger-ui',
					deepLinking: true,
					presets: [
						(window as any).SwaggerUIBundle.presets.apis,
						(window as any).SwaggerUIBundle.SwaggerUIStandalonePreset
					],
					layout: 'BaseLayout'
				});
				swaggerLoaded = true;
			} catch (err) {
				swaggerError = err instanceof Error ? err.message : 'Failed to load Swagger UI';
				console.error('Swagger UI initialization error:', err);
			}
		};
		script.onerror = () => {
			swaggerError = 'Failed to load Swagger UI script';
		};
		document.head.appendChild(script);

		// Cleanup
		return () => {
			link.remove();
			script.remove();
		};
	});
</script>

<svelte:head>
	<title>API Reference - ogis</title>
	<meta name="description" content="Complete API documentation for ogis OpenGraph Image Service" />
</svelte:head>

<div class="space-y-6">
	<div>
		<h1 class="text-4xl font-bold tracking-tight">API Reference</h1>
		<p class="mt-2 text-muted-foreground">Complete OpenAPI specification for the ogis service</p>
	</div>

	{#if swaggerError}
		<div class="rounded-lg border border-destructive bg-destructive/10 p-4">
			<p class="text-sm text-destructive">Error loading API documentation: {swaggerError}</p>
		</div>
	{/if}

	<div id="swagger-ui" class="swagger-ui-container"></div>
</div>

<style>
	:global(.swagger-ui-container) {
		/* Override some Swagger UI styles to match the site theme */
		background: transparent;
	}

	:global(.swagger-ui .topbar) {
		display: none;
	}

	:global(.swagger-ui .info) {
		margin: 0;
	}
</style>
