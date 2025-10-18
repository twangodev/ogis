<script lang="ts">
	import '../app.css';
	import Navigation from '$lib/components/layout/Navigation.svelte';
	import Footer from '$lib/components/layout/Footer.svelte';
	import { ModeWatcher } from 'mode-watcher';

	let { children } = $props();
</script>

<svelte:head>
	<link rel="icon" href="/favicon.ico" />
    <script
        src="https://rybbit.twango.dev/api/script.js"
        data-site-id="6"
        data-track-errors="true"
        defer
    ></script>
</svelte:head>

<ModeWatcher defaultMode="dark" />

<Navigation />

<main class="min-h-screen">
	{@render children?.()}
</main>

<Footer />
