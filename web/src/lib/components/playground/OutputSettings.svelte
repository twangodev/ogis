<script lang="ts">
	import { playground, type OutputFormat } from '$lib/stores/playground.svelte';
	import { Label } from '$lib/components/ui/label';
	import { Tabs, TabsContent, TabsList, TabsTrigger } from '$lib/components/ui/tabs';
	import { Slider } from '$lib/components/ui/slider';

	// Derived state from store
	const format = $derived(playground.renderOptions.format);
	const scale = $derived(playground.renderOptions.scale);
	const quality = $derived(playground.renderOptions.quality);

	function handleFormatChange(value: string) {
		playground.updateRenderOptions({ format: value as OutputFormat });
	}

	function handleScaleChange(values: number[]) {
		playground.updateRenderOptions({ scale: values[0] });
	}

	function handleQualityChange(values: number[]) {
		playground.updateRenderOptions({ quality: values[0] });
	}
</script>

<div class="space-y-5">
	<!-- Format Tabs -->
	<Tabs value={format} onValueChange={handleFormatChange}>
		<TabsList class="w-full">
			<TabsTrigger value="png" class="flex-1">PNG</TabsTrigger>
			<TabsTrigger value="jpeg" class="flex-1">JPEG</TabsTrigger>
			<TabsTrigger value="webp" class="flex-1">WebP</TabsTrigger>
		</TabsList>

		<TabsContent value="png" class="mt-4 space-y-1">
			<p class="text-sm text-muted-foreground">Lossless compression, best quality.</p>
			<p class="text-xs text-muted-foreground">Supports transparency. Larger file size.</p>
		</TabsContent>

		<TabsContent value="jpeg" class="mt-4 space-y-4">
			<div class="space-y-1">
				<p class="text-sm text-muted-foreground">Lossy compression, smaller files.</p>
				<p class="text-xs text-muted-foreground">No transparency. Best for photos.</p>
			</div>
			<div class="space-y-3">
				<div class="flex items-center justify-between">
					<Label class="text-sm font-medium">Quality</Label>
					<span class="text-sm tabular-nums text-muted-foreground">{quality}</span>
				</div>
				<Slider
					type="multiple"
					value={[quality]}
					onValueChange={handleQualityChange}
					min={1}
					max={100}
					step={1}
					class="w-full"
				/>
			</div>
		</TabsContent>

		<TabsContent value="webp" class="mt-4 space-y-4">
			<div class="space-y-1">
				<p class="text-sm text-muted-foreground">Modern format, best compression.</p>
				<p class="text-xs text-muted-foreground">Supports transparency. Smaller than PNG/JPEG.</p>
			</div>
			<div class="space-y-3">
				<div class="flex items-center justify-between">
					<Label class="text-sm font-medium">Quality</Label>
					<span class="text-sm tabular-nums text-muted-foreground">
						{quality}{quality >= 100 ? ' (lossless)' : ''}
					</span>
				</div>
				<Slider
					type="multiple"
					value={[quality]}
					onValueChange={handleQualityChange}
					min={1}
					max={100}
					step={1}
					class="w-full"
				/>
				<p class="text-xs text-muted-foreground">Set to 100 for lossless WebP.</p>
			</div>
		</TabsContent>
	</Tabs>

	<!-- Scale Slider (applies to all formats) -->
	<div class="space-y-3">
		<div class="flex items-center justify-between">
			<Label class="text-sm font-medium">Scale</Label>
			<span class="text-sm tabular-nums text-muted-foreground">
				{Math.round(scale * 100)}%
			</span>
		</div>
		<Slider
			type="multiple"
			value={[scale]}
			onValueChange={handleScaleChange}
			min={0.1}
			max={1}
			step={0.1}
			class="w-full"
		/>
		<p class="text-xs text-muted-foreground">
			Output: {Math.round(1200 * scale)} x {Math.round(630 * scale)} pixels
		</p>
	</div>
</div>
