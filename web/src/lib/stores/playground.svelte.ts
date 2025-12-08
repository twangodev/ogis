import { goto } from '$app/navigation';

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
	let urlSyncEnabled = $state(false);

	// Debounce timer for URL updates
	let urlUpdateTimeout: ReturnType<typeof setTimeout> | null = null;

	// Set templates from layout data
	function setTemplates(newTemplates: TemplateDefinition[]) {
		templates = newTemplates;
	}

	// Initialize state from URL params
	function initFromUrl(searchParams: URLSearchParams) {
		const urlTemplate = searchParams.get('template');
		if (urlTemplate && templates.some(t => t.name === urlTemplate)) {
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
		const templateColors = templates.find(t => t.name === template)?.colors ?? [];
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
		get templates() { return templates; },
		setTemplates,

		get template() { return template; },
		set template(value: string) {
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
			return templates.find(t => t.name === template);
		}
	};
}

export const playground = createPlaygroundState();