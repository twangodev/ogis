import { OgisClient, type OgisParams } from 'ogis';

const DEFAULT_API_URL = 'https://img.ogis.dev';
const STORAGE_KEY = 'ogis-api-url';

function createApiConfig() {
	let baseUrl = $state(DEFAULT_API_URL);
	let client = $state(new OgisClient({ baseUrl }));

	// Load from localStorage on init (browser only)
	if (typeof window !== 'undefined') {
		const stored = localStorage.getItem(STORAGE_KEY);
		if (stored) {
			baseUrl = stored;
			client = new OgisClient({ baseUrl: stored });
		}
	}

	return {
		get baseUrl() {
			return baseUrl;
		},
		set baseUrl(value: string) {
			const normalized = value.replace(/\/+$/, '') || DEFAULT_API_URL;
			baseUrl = normalized;
			client = new OgisClient({ baseUrl: normalized });
			if (typeof window !== 'undefined') {
				if (normalized === DEFAULT_API_URL) {
					localStorage.removeItem(STORAGE_KEY);
				} else {
					localStorage.setItem(STORAGE_KEY, normalized);
				}
			}
		},
		get isCustom() {
			return baseUrl !== DEFAULT_API_URL;
		},
		get defaultUrl() {
			return DEFAULT_API_URL;
		},
		reset() {
			baseUrl = DEFAULT_API_URL;
			client = new OgisClient({ baseUrl: DEFAULT_API_URL });
			if (typeof window !== 'undefined') {
				localStorage.removeItem(STORAGE_KEY);
			}
		},
		/** Generate an image URL using the ogis client */
		generateUrl(params: OgisParams): string {
			return client.generateUrl(params);
		}
	};
}

export const apiConfig = createApiConfig();
