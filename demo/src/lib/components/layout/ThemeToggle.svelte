<script lang="ts">
	import { mode, toggleMode } from 'mode-watcher';
	import { Sun, Moon } from '@lucide/svelte';
</script>

<button
	onclick={toggleMode}
	aria-label="Toggle theme"
	class="relative -m-2.5 rounded-md p-2.5 text-muted-foreground transition-colors hover:text-accent-foreground"
>
	<Sun class={['size-5 duration-200', mode.current === 'dark' && 'scale-0 rotate-90 opacity-0']} />
	<Moon
		class={[
			'absolute inset-0 m-auto size-5 scale-0 -rotate-90 opacity-0 duration-200',
			mode.current === 'dark' && 'scale-100 rotate-0 opacity-100'
		]}
	/>
</button>
