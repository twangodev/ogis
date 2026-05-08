import { readFileSync, readdirSync } from 'fs';
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
	layouts?: Record<string, string>;
	templates: TemplateEntry[];
}

interface ParsedTemplates {
	allTemplates: TemplateEntry[];
	gradientDefs: Record<string, GradientDef>;
	defaultTemplate: string;
}

let parsedCache: ParsedTemplates | null = null;

function loadParsedTemplates(): ParsedTemplates {
	if (parsedCache) return parsedCache;

	const repoRoot = resolve(process.cwd(), '..');

	// Read templates.yaml from repo root
	let data: TemplatesYaml = { default: '', templates: [] };
	try {
		const yamlPath = resolve(repoRoot, 'templates.yaml');
		const yamlContent = readFileSync(yamlPath, 'utf-8');
		data = parse(yamlContent) as TemplatesYaml;
	} catch (err) {
		console.error('Failed to load templates.yaml', err);
	}

	// Read gradient definitions from gradients/ directory
	const gradientsDir = resolve(repoRoot, 'gradients');
	const gradientDefs: Record<string, GradientDef> = {};
	try {
		for (const file of readdirSync(gradientsDir)) {
			if (file.endsWith('.yaml') || file.endsWith('.yml')) {
				const name = file.replace(/\.ya?ml$/, '');
				try {
					const content = readFileSync(resolve(gradientsDir, file), 'utf-8');
					gradientDefs[name] = parse(content) as GradientDef;
				} catch (err) {
					console.error(`Failed to parse gradient '${name}'`, err);
				}
			}
		}
	} catch (err) {
		console.error('Failed to read gradients directory', err);
	}

	// Auto-generate all layout × gradient combinations
	const layoutNames = Object.keys(data.layouts ?? {}).sort();
	const gradientNames = Object.keys(gradientDefs).sort();

	const composedTemplates: TemplateEntry[] = gradientNames.flatMap((gradient) =>
		layoutNames.map((layout) => ({
			name: `gradient-${gradient}-${layout}`,
			layout,
			gradient
		}))
	);

	// Combine: composed templates + file-based templates from YAML
	const allTemplates = [...composedTemplates, ...(data.templates ?? [])];

	parsedCache = {
		allTemplates,
		gradientDefs,
		defaultTemplate: data.default
	};
	return parsedCache;
}

export async function load() {
	const { allTemplates, gradientDefs, defaultTemplate } = loadParsedTemplates();

	// Get color keys for a template entry
	function getColorKeys(t: TemplateEntry): string[] {
		if (t.colors) return Object.keys(t.colors);
		if (t.gradient && gradientDefs[t.gradient]) {
			const grad = gradientDefs[t.gradient];
			return ['background', ...(grad.blobs ?? []).map((b) => `blob_${b.name}`), 'text'];
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
		colors: getColorKeys(t),
		layout: t.layout
	});

	const base = allTemplates.filter((t) => !t.name.startsWith('gradient-'));
	const gradients = allTemplates.filter((t) => t.name.startsWith('gradient-'));

	// Fetch Cloudflare stats at build time
	const stats = await fetchCloudflareStats();

	return {
		templates: {
			all: allTemplates.map(formatTemplate),
			base: base.map(formatTemplate),
			gradients: gradients.map(formatTemplate),
			default: defaultTemplate
		},
		stats
	};
}
