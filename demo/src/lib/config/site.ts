export const site = {
	name: 'ogis',
	description: 'A fast, free, and beautiful platform for open graph image generation.',
	url: 'https://ogis.dev',
	github: 'https://github.com/twangodev/ogis'
} as const;

export type SiteConfig = typeof site;