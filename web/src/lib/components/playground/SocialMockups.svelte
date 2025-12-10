<script lang="ts">
	import { playground } from '$lib/stores/playground.svelte';
	import ImagePreview from './ImagePreview.svelte';

	// Truncate text for social previews
	function truncate(text: string, maxLength: number): string {
		if (text.length <= maxLength) return text;
		return text.slice(0, maxLength - 3) + '...';
	}

	const title = $derived(playground.content.title || 'Untitled');
	const description = $derived(playground.content.description || 'No description');
</script>

<div class="grid grid-cols-1 gap-4 md:grid-cols-2">
	<!-- Row 1: Twitter & Facebook (image on top) -->
	<div class="flex flex-col gap-2">
		<span class="text-xs font-medium text-muted-foreground">Twitter / X</span>
		<div class="flex-1 overflow-hidden rounded-xl border border-border bg-card">
			<ImagePreview src={playground.previewUrl} alt="Twitter card preview" />
			<div class="border-t border-border p-3">
				<p class="text-xs text-muted-foreground">ogis.dev</p>
				<p class="truncate text-sm font-medium">{truncate(title, 70)}</p>
				<p class="truncate text-sm text-muted-foreground">{truncate(description, 100)}</p>
			</div>
		</div>
	</div>

	<div class="flex flex-col gap-2">
		<span class="text-xs font-medium text-muted-foreground">Facebook</span>
		<div class="flex-1 overflow-hidden rounded-lg border border-border bg-card">
			<ImagePreview src={playground.previewUrl} alt="Facebook card preview" />
			<div class="bg-muted/50 p-3">
				<p class="text-xs tracking-wide text-muted-foreground uppercase">ogis.dev</p>
				<p class="mt-1 line-clamp-2 text-sm font-semibold">{truncate(title, 100)}</p>
				<p class="mt-1 line-clamp-1 text-sm text-muted-foreground">{truncate(description, 80)}</p>
			</div>
		</div>
	</div>

	<!-- Row 2: LinkedIn & Discord -->
	<div class="flex flex-col gap-2">
		<span class="text-xs font-medium text-muted-foreground">LinkedIn</span>
		<div class="flex-1 overflow-hidden rounded-lg border border-border bg-card">
			<ImagePreview src={playground.previewUrl} alt="LinkedIn card preview" />
			<div class="p-3">
				<p class="line-clamp-2 text-sm font-semibold">{truncate(title, 150)}</p>
				<p class="mt-1 text-xs text-muted-foreground">ogis.dev</p>
			</div>
		</div>
	</div>

	<div class="flex flex-col gap-2">
		<span class="text-xs font-medium text-muted-foreground">Discord</span>
		<div
			class="flex flex-1 flex-col overflow-hidden rounded-lg border-l-4 border-l-[#5865F2] bg-[#313338]"
		>
			<div class="flex flex-1 flex-col p-3">
				<p class="text-xs font-medium text-[#00AFF4]">ogis.dev</p>
				<p
					class="mt-1 line-clamp-1 cursor-pointer text-sm font-semibold text-[#00AFF4] hover:underline"
				>
					{truncate(title, 100)}
				</p>
				<p class="mt-1 line-clamp-2 text-xs text-[#B5BAC1]">
					{truncate(description, 150)}
				</p>
				<div class="mt-auto overflow-hidden rounded pt-2">
					<ImagePreview src={playground.previewUrl} alt="Discord embed preview" />
				</div>
			</div>
		</div>
	</div>

	<!-- Row 3: Slack & WhatsApp -->
	<div class="flex flex-col gap-2">
		<span class="text-xs font-medium text-muted-foreground">Slack</span>
		<div
			class="flex flex-1 flex-col overflow-hidden rounded-lg border border-l-4 border-border border-l-[#36C5F0] bg-card"
		>
			<div class="flex flex-1 flex-col p-3">
				<div class="flex items-center gap-2">
					<div class="flex size-4 items-center justify-center rounded bg-muted">
						<span class="text-[10px] font-bold">O</span>
					</div>
					<span class="text-sm font-bold">ogis.dev</span>
				</div>
				<p
					class="mt-2 line-clamp-1 cursor-pointer text-sm font-bold text-[#1264A3] hover:underline dark:text-[#1D9BD1]"
				>
					{truncate(title, 100)}
				</p>
				<p class="mt-1 line-clamp-2 text-xs text-muted-foreground">
					{truncate(description, 150)}
				</p>
				<div class="mt-auto overflow-hidden rounded pt-2">
					<ImagePreview src={playground.previewUrl} alt="Slack preview" />
				</div>
			</div>
		</div>
	</div>

	<div class="flex flex-col gap-2">
		<span class="text-xs font-medium text-muted-foreground">WhatsApp</span>
		<div class="flex-1 overflow-hidden rounded-lg bg-[#202C33]">
			<div class="h-full p-2">
				<div class="flex h-full flex-col overflow-hidden rounded-lg bg-[#025144]">
					<ImagePreview src={playground.previewUrl} alt="WhatsApp preview" />
					<div class="flex-1 bg-[#025144] p-2">
						<p class="text-xs text-[#8696A0] uppercase">ogis.dev</p>
						<p class="mt-0.5 line-clamp-2 text-sm font-medium text-white">
							{truncate(title, 80)}
						</p>
						<p class="mt-0.5 line-clamp-2 text-xs text-[#8696A0]">{truncate(description, 100)}</p>
					</div>
				</div>
			</div>
		</div>
	</div>
</div>
