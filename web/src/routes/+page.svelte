<script lang="ts">
	import Hero from '$lib/components/hero/Hero.svelte';

	let { data } = $props();
</script>

<Hero templates={data.templates} stats={data.stats} />
