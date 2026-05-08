import type { OgisParams } from 'ogis';

const DEFAULT_API_URL = 'https://img.ogis.dev';
const STORAGE_KEY = 'ogis-api-url';

function buildUrl(base: string, params: OgisParams): string {
	const qs = new URLSearchParams();
	for (const [key, value] of Object.entries(params)) {
		if (value !== undefined) qs.append(key, String(value));
	}
	const query = qs.toString();
	return query ? `${base}/?${query}` : `${base}/`;
}

function normalize(value: string): string {
	return value.trim().replace(/\/+$/, '') || DEFAULT_API_URL;
}

function createApiConfig() {
	let baseUrl = $state(DEFAULT_API_URL);

	if (typeof window !== 'undefined') {
		const stored = localStorage.getItem(STORAGE_KEY);
		if (stored) {
			const normalized = normalize(stored);
			baseUrl = normalized;
			if (normalized !== stored) {
				if (normalized === DEFAULT_API_URL) {
					localStorage.removeItem(STORAGE_KEY);
				} else {
					localStorage.setItem(STORAGE_KEY, normalized);
				}
			}
		}
	}

	return {
		get baseUrl() {
			return baseUrl;
		},
		set baseUrl(value: string) {
			const normalized = normalize(value);
			baseUrl = normalized;
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
			if (typeof window !== 'undefined') {
				localStorage.removeItem(STORAGE_KEY);
			}
		},
		generateUrl(params: OgisParams): string {
			return buildUrl(baseUrl, params);
		}
	};
}

export const apiConfig = createApiConfig();
