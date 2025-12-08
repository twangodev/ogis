import { goto } from '$app/navigation';
import { page } from '$app/state';

// Template definitions with their customizable colors
export const TEMPLATES = [
	{ name: 'twilight', label: 'Twilight', colors: ['background', 'primary', 'secondary', 'title_text', 'subtitle_text', 'description_text'] },
	{ name: 'daybreak', label: 'Daybreak', colors: ['background', 'gradient_accent', 'title_text', 'subtitle_text', 'description_text', 'logo_accent', 'image_accent'] },
	{ name: 'minimal', label: 'Minimal', colors: ['background', 'border', 'title_text', 'subtitle_text', 'description_text'] },
	{ name: 'stripe', label: 'Stripe', colors: ['background', 'accent_primary', 'accent_secondary', 'border', 'box_background', 'title_text', 'subtitle_text', 'description_text'] },
	{ name: 'hero', label: 'Hero', colors: ['background', 'placeholder_text', 'title_text', 'subtitle_text', 'description_text'] },
	{ name: 'modern', label: 'Modern', colors: ['gradient_start', 'gradient_end', 'accent_blue', 'accent_pink', 'box_background', 'border', 'title_text', 'subtitle_text', 'description_text'] },
	{ name: 'fish', label: 'Fish', colors: ['background', 'background_end', 'blob_purple', 'blob_purple_mid', 'blob_purple_dark', 'waveform_dark', 'waveform_mid', 'waveform_light', 'text'] },
	{ name: 'gradient-arctic', label: 'Arctic', colors: ['background', 'blob_cyan', 'blob_light_cyan', 'blob_sky', 'blob_ice', 'blob_teal', 'text'] },
	{ name: 'gradient-aurora', label: 'Aurora', colors: ['background', 'blob_violet', 'blob_pink', 'blob_orange', 'blob_yellow', 'blob_blue', 'text'] },
	{ name: 'gradient-berry', label: 'Berry', colors: ['background', 'blob_rose', 'blob_crimson', 'blob_red', 'blob_wine', 'blob_pink', 'text'] },
	{ name: 'gradient-candy', label: 'Candy', colors: ['background', 'blob_pink', 'blob_fuchsia', 'blob_purple', 'blob_rose', 'blob_light_pink', 'text'] },
	{ name: 'gradient-copper', label: 'Copper', colors: ['background', 'blob_amber', 'blob_orange', 'blob_yellow', 'blob_gold', 'blob_brown', 'text'] },
	{ name: 'gradient-ember', label: 'Ember', colors: ['background', 'blob_red', 'blob_orange', 'blob_flame', 'blob_yellow', 'blob_dark_red', 'text'] },
	{ name: 'gradient-forest', label: 'Forest', colors: ['background', 'blob_green', 'blob_emerald', 'blob_teal', 'blob_lime', 'blob_light_lime', 'text'] },
	{ name: 'gradient-galaxy', label: 'Galaxy', colors: ['background', 'blob_indigo', 'blob_violet', 'blob_purple', 'blob_blue', 'blob_pink', 'blob_light_pink', 'text'] },
	{ name: 'gradient-lavender', label: 'Lavender', colors: ['background', 'blob_violet', 'blob_light_violet', 'blob_pale_violet', 'blob_pink', 'blob_lavender', 'text'] },
	{ name: 'gradient-midnight', label: 'Midnight', colors: ['background', 'blob_navy', 'blob_indigo', 'blob_violet', 'blob_light_indigo', 'blob_periwinkle', 'text'] },
	{ name: 'gradient-mint', label: 'Mint', colors: ['background', 'blob_aqua', 'blob_teal', 'blob_light_teal', 'blob_green', 'blob_emerald', 'text'] },
	{ name: 'gradient-moss', label: 'Moss', colors: ['background', 'blob_olive', 'blob_lime', 'blob_stone', 'blob_green', 'blob_gray', 'text'] },
	{ name: 'gradient-neon', label: 'Neon', colors: ['background', 'blob_cyan', 'blob_fuchsia', 'blob_light_cyan', 'blob_purple', 'blob_pink', 'text'] },
	{ name: 'gradient-ocean', label: 'Ocean', colors: ['background', 'blob_sky', 'blob_cyan', 'blob_teal', 'blob_light_cyan', 'blob_aqua', 'text'] },
	{ name: 'gradient-peach', label: 'Peach', colors: ['background', 'blob_peach', 'blob_light_peach', 'blob_yellow', 'blob_orange', 'blob_cream', 'text'] },
	{ name: 'gradient-storm', label: 'Storm', colors: ['background', 'blob_slate', 'blob_gray', 'blob_dark_slate', 'blob_light_gray', 'blob_silver', 'text'] },
	{ name: 'gradient-sunset', label: 'Sunset', colors: ['background', 'blob_yellow', 'blob_orange', 'blob_red', 'blob_pink', 'blob_rose', 'text'] }
] as const;

export type TemplateName = typeof TEMPLATES[number]['name'];

export interface PlaygroundContent {
	title: string;
	subtitle: string;
	description: string;
}

export interface PlaygroundMedia {
	logo: string;
	image: string;
}

// Create the playground state as an object (so it can be exported)
function createPlaygroundState() {
	let template = $state<TemplateName>('twilight');
	let content = $state<PlaygroundContent>({
		title: 'Your Title Here',
		subtitle: 'Open Graph Images',
		description: 'Generate beautiful OG images in real-time'
	});
	let media = $state<PlaygroundMedia>({
		logo: '',
		image: ''
	});
	let colors = $state<Record<string, string>>({});
	let urlSyncEnabled = $state(false);

	// Debounce timer for URL updates
	let urlUpdateTimeout: ReturnType<typeof setTimeout> | null = null;

	// Initialize state from URL params
	function initFromUrl(searchParams: URLSearchParams) {
		const urlTemplate = searchParams.get('template');
		if (urlTemplate && TEMPLATES.some(t => t.name === urlTemplate)) {
			template = urlTemplate as TemplateName;
		}

		const urlTitle = searchParams.get('title');
		if (urlTitle) content.title = urlTitle;

		const urlSubtitle = searchParams.get('subtitle');
		if (urlSubtitle) content.subtitle = urlSubtitle;

		const urlDescription = searchParams.get('description');
		if (urlDescription) content.description = urlDescription;

		const urlLogo = searchParams.get('logo');
		if (urlLogo) media.logo = urlLogo;

		const urlImage = searchParams.get('image');
		if (urlImage) media.image = urlImage;

		// Parse color overrides
		const templateColors = TEMPLATES.find(t => t.name === template)?.colors ?? [];
		for (const colorKey of templateColors) {
			const colorValue = searchParams.get(colorKey);
			if (colorValue && /^[0-9a-fA-F]{6}$/.test(colorValue)) {
				colors[colorKey] = colorValue;
			}
		}

		urlSyncEnabled = true;
	}

	// Update URL with current state (debounced)
	function syncToUrl() {
		if (!urlSyncEnabled) return;

		if (urlUpdateTimeout) {
			clearTimeout(urlUpdateTimeout);
		}

		urlUpdateTimeout = setTimeout(() => {
			const params = new URLSearchParams();

			if (template !== 'twilight') {
				params.set('template', template);
			}
			if (content.title && content.title !== 'Your Title Here') {
				params.set('title', content.title);
			}
			if (content.subtitle && content.subtitle !== 'Open Graph Images') {
				params.set('subtitle', content.subtitle);
			}
			if (content.description && content.description !== 'Generate beautiful OG images in real-time') {
				params.set('description', content.description);
			}
			if (media.logo) {
				params.set('logo', media.logo);
			}
			if (media.image) {
				params.set('image', media.image);
			}

			// Add color overrides
			for (const [key, value] of Object.entries(colors)) {
				if (value) {
					params.set(key, value);
				}
			}

			const queryString = params.toString();
			const newUrl = queryString ? `/playground?${queryString}` : '/playground';

			goto(newUrl, { replaceState: true, noScroll: true, keepFocus: true });
		}, 500);
	}

	return {
		get template() { return template; },
		set template(value: TemplateName) {
			template = value;
			// Reset colors when template changes
			colors = {};
			syncToUrl();
		},

		get content() { return content; },
		set content(value: PlaygroundContent) {
			content = value;
			syncToUrl();
		},
		updateContent(updates: Partial<PlaygroundContent>) {
			content = { ...content, ...updates };
			syncToUrl();
		},

		get media() { return media; },
		set media(value: PlaygroundMedia) {
			media = value;
			syncToUrl();
		},
		updateMedia(updates: Partial<PlaygroundMedia>) {
			media = { ...media, ...updates };
			syncToUrl();
		},

		get colors() { return colors; },
		set colors(value: Record<string, string>) {
			colors = value;
			syncToUrl();
		},
		updateColor(key: string, value: string) {
			colors = { ...colors, [key]: value };
			syncToUrl();
		},
		resetColors() {
			colors = {};
			syncToUrl();
		},

		initFromUrl,

		// Generate the API URL for the current state
		get apiUrl() {
			const params = new URLSearchParams();
			params.set('template', template);

			if (content.title) params.set('title', content.title);
			if (content.subtitle) params.set('subtitle', content.subtitle);
			if (content.description) params.set('description', content.description);
			if (media.logo) params.set('logo', media.logo);
			if (media.image) params.set('image', media.image);

			// Add color overrides
			for (const [key, value] of Object.entries(colors)) {
				if (value) {
					params.set(key, value);
				}
			}

			return `https://img.ogis.dev/?${params.toString()}`;
		},

		// Get current template info
		get currentTemplate() {
			return TEMPLATES.find(t => t.name === template);
		}
	};
}

export const playground = createPlaygroundState();
