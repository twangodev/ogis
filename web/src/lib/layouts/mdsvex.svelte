<script lang="ts">
	let { children } = $props();
</script>

<article class="prose max-w-none prose-slate dark:prose-invert">
	{@render children?.()}
</article>
