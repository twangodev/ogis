<script lang="ts">
	import '../app.css';
	import Navigation from '$lib/components/layout/Navigation.svelte';
	import Footer from '$lib/components/layout/Footer.svelte';
	import { ModeWatcher } from 'mode-watcher';

	let { children } = $props();
</script>

<svelte:head>
	<link rel="icon" href="/favicon.ico" />
</svelte:head>

<ModeWatcher defaultMode="dark" />

<Navigation />

<main class="min-h-screen">
	{@render children?.()}
</main>

<Footer />
