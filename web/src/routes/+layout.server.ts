import { readFileSync } from 'fs';
import { parse } from 'yaml';
import { resolve } from 'path';
import { fetchCloudflareStats } from '$lib/cloudflare';

interface GradientBlob {
	name: string;
}

interface GradientDef {
	blobs: GradientBlob[];
}

interface TemplateEntry {
	name: string;
	// File-based templates
	file?: string;
	colors?: Record<string, string>;
	// Composed templates
	layout?: string;
	gradient?: string;
}

interface TemplatesYaml {
	default: string;
	gradients?: Record<string, GradientDef>;
	templates: TemplateEntry[];
}

export async function load() {
	// Read templates.yaml from repo root (one level up from web/)
	const yamlPath = resolve(process.cwd(), '..', 'templates.yaml');
	const yamlContent = readFileSync(yamlPath, 'utf-8');
	const data = parse(yamlContent) as TemplatesYaml;

	// Split into base and gradient templates
	const base = data.templates.filter((t) => !t.name.startsWith('gradient-'));
	const gradients = data.templates.filter((t) => t.name.startsWith('gradient-'));

	// Get color keys for a template entry
	function getColorKeys(t: TemplateEntry): string[] {
		// File-based templates have explicit colors
		if (t.colors) return Object.keys(t.colors);
		// Composed templates derive colors from the gradient definition
		if (t.gradient && data.gradients?.[t.gradient]) {
			const grad = data.gradients[t.gradient];
			return ['background', ...grad.blobs.map((b) => `blob_${b.name}`), 'text'];
		}
		return [];
	}

	// Create display-friendly format
	const formatTemplate = (t: TemplateEntry) => ({
		name: t.name,
		label: t.name
			.replace('gradient-', '')
			.replace(/-/g, ' ')
			.replace(/\b\w/g, (c) => c.toUpperCase()),
		colors: getColorKeys(t)
	});

	// Fetch Cloudflare stats at build time
	const stats = await fetchCloudflareStats();

	return {
		templates: {
			all: data.templates.map(formatTemplate),
			base: base.map(formatTemplate),
			gradients: gradients.map(formatTemplate),
			default: data.default
		},
		stats
	};
}
