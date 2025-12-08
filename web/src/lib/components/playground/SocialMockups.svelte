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

<div class="grid grid-cols-1 md:grid-cols-2 gap-4">
	<!-- Row 1: Twitter & Facebook (image on top) -->
	<div class="flex flex-col gap-2">
		<span class="text-xs font-medium text-muted-foreground">Twitter / X</span>
		<div class="border border-border rounded-xl overflow-hidden bg-card flex-1">
			<ImagePreview src={playground.apiUrl} alt="Twitter card preview" />
			<div class="p-3 border-t border-border">
				<p class="text-xs text-muted-foreground">ogis.dev</p>
				<p class="text-sm font-medium truncate">{truncate(title, 70)}</p>
				<p class="text-sm text-muted-foreground truncate">{truncate(description, 100)}</p>
			</div>
		</div>
	</div>

	<div class="flex flex-col gap-2">
		<span class="text-xs font-medium text-muted-foreground">Facebook</span>
		<div class="border border-border rounded-lg overflow-hidden bg-card flex-1">
			<ImagePreview src={playground.apiUrl} alt="Facebook card preview" />
			<div class="p-3 bg-muted/50">
				<p class="text-xs text-muted-foreground uppercase tracking-wide">ogis.dev</p>
				<p class="text-sm font-semibold mt-1 line-clamp-2">{truncate(title, 100)}</p>
				<p class="text-sm text-muted-foreground mt-1 line-clamp-1">{truncate(description, 80)}</p>
			</div>
		</div>
	</div>

	<!-- Row 2: LinkedIn & Discord -->
	<div class="flex flex-col gap-2">
		<span class="text-xs font-medium text-muted-foreground">LinkedIn</span>
		<div class="border border-border rounded-lg overflow-hidden bg-card flex-1">
			<ImagePreview src={playground.apiUrl} alt="LinkedIn card preview" />
			<div class="p-3">
				<p class="text-sm font-semibold line-clamp-2">{truncate(title, 150)}</p>
				<p class="text-xs text-muted-foreground mt-1">ogis.dev</p>
			</div>
		</div>
	</div>

	<div class="flex flex-col gap-2">
		<span class="text-xs font-medium text-muted-foreground">Discord</span>
		<div
			class="bg-[#313338] rounded-lg overflow-hidden border-l-4 border-l-[#5865F2] flex-1 flex flex-col"
		>
			<div class="p-3 flex-1 flex flex-col">
				<p class="text-xs text-[#00AFF4] font-medium">ogis.dev</p>
				<p
					class="text-[#00AFF4] font-semibold text-sm mt-1 hover:underline cursor-pointer line-clamp-1"
				>
					{truncate(title, 100)}
				</p>
				<p class="text-xs text-[#B5BAC1] mt-1 line-clamp-2">
					{truncate(description, 150)}
				</p>
				<div class="mt-auto pt-2 rounded overflow-hidden">
					<ImagePreview src={playground.apiUrl} alt="Discord embed preview" />
				</div>
			</div>
		</div>
	</div>

	<!-- Row 3: Slack & WhatsApp -->
	<div class="flex flex-col gap-2">
		<span class="text-xs font-medium text-muted-foreground">Slack</span>
		<div
			class="bg-card rounded-lg overflow-hidden border border-border border-l-4 border-l-[#36C5F0] flex-1 flex flex-col"
		>
			<div class="p-3 flex-1 flex flex-col">
				<div class="flex items-center gap-2">
					<div class="size-4 rounded bg-muted flex items-center justify-center">
						<span class="text-[10px] font-bold">O</span>
					</div>
					<span class="text-sm font-bold">ogis.dev</span>
				</div>
				<p
					class="text-[#1264A3] dark:text-[#1D9BD1] font-bold text-sm mt-2 hover:underline cursor-pointer line-clamp-1"
				>
					{truncate(title, 100)}
				</p>
				<p class="text-xs text-muted-foreground mt-1 line-clamp-2">
					{truncate(description, 150)}
				</p>
				<div class="mt-auto pt-2 rounded overflow-hidden">
					<ImagePreview src={playground.apiUrl} alt="Slack preview" />
				</div>
			</div>
		</div>
	</div>

	<div class="flex flex-col gap-2">
		<span class="text-xs font-medium text-muted-foreground">WhatsApp</span>
		<div class="bg-[#202C33] rounded-lg overflow-hidden flex-1">
			<div class="p-2 h-full">
				<div class="bg-[#025144] rounded-lg overflow-hidden h-full flex flex-col">
					<ImagePreview src={playground.apiUrl} alt="WhatsApp preview" />
					<div class="p-2 bg-[#025144] flex-1">
						<p class="text-xs text-[#8696A0] uppercase">ogis.dev</p>
						<p class="text-sm text-white font-medium mt-0.5 line-clamp-2">
							{truncate(title, 80)}
						</p>
						<p class="text-xs text-[#8696A0] mt-0.5 line-clamp-2">{truncate(description, 100)}</p>
					</div>
				</div>
			</div>
		</div>
	</div>
</div>