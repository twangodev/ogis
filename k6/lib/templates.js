import YAML from 'https://cdnjs.cloudflare.com/ajax/libs/js-yaml/4.1.0/js-yaml.min.js';

/**
 * Load template names from templates.yaml
 */
export function loadTemplates(path = '../templates.yaml') {
  const yaml = YAML.load(open(path));
  return yaml.templates.map((t) => t.name);
}

/**
 * Sanitize template name for k6 metric names (no hyphens allowed)
 */
export function sanitize(name) {
  return name.replace(/-/g, '_');
}