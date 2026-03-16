import { goto } from '$app/navigation';
import { shuffle } from '$lib/utils/shuffle';
import { apiConfig } from '$lib/config/api.svelte';

export interface TemplateDefinition {
	name: string;
	label: string;
	colors: string[];
}

export interface PlaygroundContent {
	title: string;
	subtitle: string;
	description: string;
}

export interface PlaygroundMedia {
	logo: string;
	image: string;
}

export type OutputFormat = 'png' | 'jpeg' | 'webp';

export interface PlaygroundRenderOptions {
	format: OutputFormat;
	scale: number; // 0.1 - 1.0
	quality: number; // 1 - 100
}

// Create the playground state as an object (so it can be exported)
function createPlaygroundState() {
	let templates = $state<TemplateDefinition[]>([]);
	let template = $state<string>('twilight');
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
	let renderOptions = $state<PlaygroundRenderOptions>({
		format: 'png',
		scale: 1.0,
		quality: 90
	});
	let urlSyncEnabled = $state(false);

	// Debounce timer for URL updates
	let urlUpdateTimeout: ReturnType<typeof setTimeout> | null = null;

	// Debounced API URL for preview components
	let debouncedApiUrl = $state('');
	let apiUrlDebounceTimer: ReturnType<typeof setTimeout> | null = null;

	// Set templates from layout data
	function setTemplates(newTemplates: TemplateDefinition[]) {
		templates = newTemplates;
	}

	// Initialize state from URL params
	function initFromUrl(searchParams: URLSearchParams) {
		const urlTemplate = searchParams.get('template');
		if (urlTemplate && templates.some((t) => t.name === urlTemplate)) {
			template = urlTemplate;
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
		const templateColors = templates.find((t) => t.name === template)?.colors ?? [];
		for (const colorKey of templateColors) {
			const colorValue = searchParams.get(colorKey);
			if (colorValue && /^[0-9a-fA-F]{6}$/.test(colorValue)) {
				colors[colorKey] = colorValue;
			}
		}

		// Parse render options
		const urlFormat = searchParams.get('format');
		if (urlFormat && ['png', 'jpeg', 'webp'].includes(urlFormat)) {
			renderOptions.format = urlFormat as OutputFormat;
		}

		const urlScale = searchParams.get('scale');
		if (urlScale) {
			const scale = parseFloat(urlScale);
			if (!isNaN(scale) && scale >= 0.1 && scale <= 1.0) {
				renderOptions.scale = scale;
			}
		}

		const urlQuality = searchParams.get('quality');
		if (urlQuality) {
			const quality = parseInt(urlQuality, 10);
			if (!isNaN(quality) && quality >= 1 && quality <= 100) {
				renderOptions.quality = quality;
			}
		}

		urlSyncEnabled = true;

		// Initialize debounced URL immediately
		debouncedApiUrl = buildApiUrl();
	}

	// Build the current API URL (not debounced)
	function buildApiUrl(): string {
		const params: Record<string, string> = { template };

		if (content.title) params.title = content.title;
		if (content.subtitle) params.subtitle = content.subtitle;
		if (content.description) params.description = content.description;
		if (media.logo) params.logo = media.logo;
		if (media.image) params.image = media.image;

		// Add color overrides
		for (const [key, value] of Object.entries(colors)) {
			if (value) params[key] = value;
		}

		// Add render options (only non-default values)
		if (renderOptions.format !== 'png') params.format = renderOptions.format;
		if (renderOptions.scale !== 1.0) params.scale = renderOptions.scale.toString();
		if (renderOptions.quality !== 90) params.quality = renderOptions.quality.toString();

		return apiConfig.generateUrl(params);
	}

	// Update debounced API URL
	function updateDebouncedApiUrl() {
		if (apiUrlDebounceTimer) {
			clearTimeout(apiUrlDebounceTimer);
		}

		apiUrlDebounceTimer = setTimeout(() => {
			debouncedApiUrl = buildApiUrl();
		}, 500);
	}

	// Update URL with current state (debounced)
	function syncToUrl() {
		if (!urlSyncEnabled) return;

		// Also update debounced API URL for previews
		updateDebouncedApiUrl();

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
			if (
				content.description &&
				content.description !== 'Generate beautiful OG images in real-time'
			) {
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

			// Add render options (only non-default values)
			if (renderOptions.format !== 'png') {
				params.set('format', renderOptions.format);
			}
			if (renderOptions.scale !== 1.0) {
				params.set('scale', renderOptions.scale.toString());
			}
			if (renderOptions.quality !== 90) {
				params.set('quality', renderOptions.quality.toString());
			}

			const queryString = params.toString();
			const newUrl = queryString ? `/playground?${queryString}` : '/playground';

			goto(newUrl, { replaceState: true, noScroll: true, keepFocus: true });
		}, 500);
	}

	return {
		get templates() {
			return templates;
		},
		setTemplates,
		shuffleTemplates() {
			templates = shuffle(templates);
		},

		get template() {
			return template;
		},
		set template(value: string) {
			template = value;
			// Reset colors when template changes
			colors = {};
			syncToUrl();
		},

		get content() {
			return content;
		},
		set content(value: PlaygroundContent) {
			content = value;
			syncToUrl();
		},
		updateContent(updates: Partial<PlaygroundContent>) {
			content = { ...content, ...updates };
			syncToUrl();
		},

		get media() {
			return media;
		},
		set media(value: PlaygroundMedia) {
			media = value;
			syncToUrl();
		},
		updateMedia(updates: Partial<PlaygroundMedia>) {
			media = { ...media, ...updates };
			syncToUrl();
		},

		get colors() {
			return colors;
		},
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

		get renderOptions() {
			return renderOptions;
		},
		set renderOptions(value: PlaygroundRenderOptions) {
			renderOptions = value;
			syncToUrl();
		},
		updateRenderOptions(updates: Partial<PlaygroundRenderOptions>) {
			renderOptions = { ...renderOptions, ...updates };
			syncToUrl();
		},

		initFromUrl,

		// Generate the API URL for the current state (not debounced - updates immediately)
		get apiUrl() {
			return buildApiUrl();
		},

		// Debounced API URL for preview components (waits 500ms after last change)
		get previewUrl() {
			return debouncedApiUrl;
		},

		// Get current template info
		get currentTemplate() {
			return templates.find((t) => t.name === template);
		}
	};
}

export const playground = createPlaygroundState();
