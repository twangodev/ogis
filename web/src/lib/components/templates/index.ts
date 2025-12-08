export { default as TemplateCategory } from './TemplateCategory.svelte';
export { default as TemplateCard } from './TemplateCard.svelte';
export { default as TemplateMarquee } from './TemplateMarquee.svelte';

// Template categories for display
export const TEMPLATE_CATEGORIES = {
	base: [
		{ name: 'twilight', label: 'Twilight', description: 'Deep purple gradients with a cosmic feel' },
		{ name: 'daybreak', label: 'Daybreak', description: 'Warm sunrise colors for a fresh start' },
		{ name: 'minimal', label: 'Minimal', description: 'Clean and simple, focus on content' },
		{ name: 'stripe', label: 'Stripe', description: 'Inspired by modern SaaS design' },
		{ name: 'hero', label: 'Hero', description: 'Bold imagery with your photo front and center' },
		{ name: 'modern', label: 'Modern', description: 'Geometric patterns with vibrant accents' },
		{ name: 'fish', label: 'Fish', description: 'Organic waveforms with fluid motion' }
	],
	gradients: [
		{ name: 'gradient-arctic', label: 'Arctic', description: 'Cool cyan and ice blue tones' },
		{ name: 'gradient-aurora', label: 'Aurora', description: 'Northern lights in motion' },
		{ name: 'gradient-berry', label: 'Berry', description: 'Rich reds and deep wines' },
		{ name: 'gradient-candy', label: 'Candy', description: 'Sweet pinks and playful purples' },
		{ name: 'gradient-copper', label: 'Copper', description: 'Warm metallics and amber glow' },
		{ name: 'gradient-ember', label: 'Ember', description: 'Fiery oranges and burning reds' },
		{ name: 'gradient-forest', label: 'Forest', description: 'Deep greens and natural tones' },
		{ name: 'gradient-galaxy', label: 'Galaxy', description: 'Deep space purples and cosmic blues' },
		{ name: 'gradient-lavender', label: 'Lavender', description: 'Soft violets and gentle purples' },
		{ name: 'gradient-midnight', label: 'Midnight', description: 'Dark navy with subtle indigo' },
		{ name: 'gradient-mint', label: 'Mint', description: 'Fresh teals and cool aquas' },
		{ name: 'gradient-moss', label: 'Moss', description: 'Earthy olives and stone grays' },
		{ name: 'gradient-neon', label: 'Neon', description: 'Electric cyans and hot magentas' },
		{ name: 'gradient-ocean', label: 'Ocean', description: 'Deep sea blues and aquatic teals' },
		{ name: 'gradient-peach', label: 'Peach', description: 'Soft oranges and creamy yellows' },
		{ name: 'gradient-storm', label: 'Storm', description: 'Moody grays and silver streaks' },
		{ name: 'gradient-sunset', label: 'Sunset', description: 'Golden hour oranges and pinks' }
	]
} as const;

export type TemplateInfo = {
	name: string;
	label: string;
	description: string;
};
