import { readFileSync } from 'fs';
import { parse } from 'yaml';
import { resolve } from 'path';

interface TemplateConfig {
	name: string;
	file: string;
	colors: Record<string, string>;
}

interface TemplatesYaml {
	default: string;
	templates: TemplateConfig[];
}

export function load() {
	// Read templates.yaml from repo root (one level up from web/)
	const yamlPath = resolve(process.cwd(), '..', 'templates.yaml');
	const yamlContent = readFileSync(yamlPath, 'utf-8');
	const data = parse(yamlContent) as TemplatesYaml;

	// Split into base and gradient templates
	const base = data.templates.filter((t) => !t.name.startsWith('gradient-'));
	const gradients = data.templates.filter((t) => t.name.startsWith('gradient-'));

	// Create display-friendly format
	const formatTemplate = (t: TemplateConfig) => ({
		name: t.name,
		label: t.name
			.replace('gradient-', '')
			.replace(/-/g, ' ')
			.replace(/\b\w/g, (c) => c.toUpperCase()),
		colors: Object.keys(t.colors)
	});

	return {
		templates: {
			all: data.templates.map(formatTemplate),
			base: base.map(formatTemplate),
			gradients: gradients.map(formatTemplate),
			default: data.default
		}
	};
}