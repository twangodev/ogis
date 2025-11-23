<script lang="ts">
	let { children } = $props();
</script>

<div class="container mx-auto max-w-4xl px-6 py-24">
	<article class="prose prose-slate dark:prose-invert max-w-none">
		{@render children?.()}
	</article>
</div>
