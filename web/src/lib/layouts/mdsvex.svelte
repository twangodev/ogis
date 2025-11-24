<script lang="ts">
	let { children } = $props();
</script>

<article class="prose prose-slate dark:prose-invert max-w-none">
	{@render children?.()}
</article>
