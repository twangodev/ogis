<script lang="ts">
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';

	interface Props {
		label: string;
		value: string;
		onchange: (value: string) => void;
	}

	let { label, value, onchange }: Props = $props();

	// Validate hex color (6 characters, no #)
	function isValidHex(hex: string): boolean {
		if (!hex) return true;
		return /^[0-9a-fA-F]{6}$/.test(hex);
	}

	const isValid = $derived(isValidHex(value));

	function handleInput(e: Event) {
		const target = e.target as HTMLInputElement;
		// Remove # if present and limit to 6 chars
		let hex = target.value.replace(/^#/, '').slice(0, 6);
		onchange(hex);
	}

	function handleColorPickerChange(e: Event) {
		const target = e.target as HTMLInputElement;
		// Remove # from the color picker value
		onchange(target.value.slice(1));
	}

	// Convert hex to CSS color (with #)
	const cssColor = $derived(value ? `#${value}` : '#808080');
</script>

<div class="space-y-1">
	<Label class="text-xs">{label}</Label>
	<div class="flex items-center gap-2">
		<!-- Color preview/picker -->
		<label class="relative">
			<div
				class="size-8 cursor-pointer overflow-hidden rounded-md border border-border"
				style="background-color: {cssColor}"
			>
				<input
					type="color"
					value={cssColor}
					oninput={handleColorPickerChange}
					class="absolute inset-0 cursor-pointer opacity-0"
				/>
			</div>
		</label>

		<!-- Hex input -->
		<div class="relative flex-1">
			<span class="absolute top-1/2 left-2 -translate-y-1/2 text-sm text-muted-foreground">#</span>
			<Input
				type="text"
				placeholder="FF0000"
				{value}
				oninput={handleInput}
				maxlength={6}
				class="pl-5 font-mono text-xs uppercase {!isValid ? 'border-destructive' : ''}"
			/>
		</div>
	</div>
</div>
