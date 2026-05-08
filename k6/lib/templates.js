import http from 'k6/http';
import YAML from 'https://cdnjs.cloudflare.com/ajax/libs/js-yaml/4.1.0/js-yaml.min.js';

/**
 * Init-context helper. k6's init phase can read files but cannot make HTTP
 * requests, so we use the file-based template manifest to get the static
 * template names. These are what we declare per-template Trend metrics for —
 * the gradient family is reported in aggregate (see ogis.js).
 *
 * @param {string} [path='../templates.yaml']
 */
export function loadStaticTemplates(path = '../templates.yaml') {
  const yaml = YAML.load(open(path));
  return yaml.templates.map((t) => t.name);
}

/**
 * Setup-context helper (must be called from k6's `setup()`, where HTTP is
 * allowed). Fetches the running server's `/templates` list — the source of
 * truth for everything ogis can render, including the 864 auto-composed
 * gradient templates.
 *
 * @param {string} baseUrl
 */
export function fetchAllTemplates(baseUrl) {
  const res = http.get(`${baseUrl}/templates`);
  if (res.status !== 200) {
    throw new Error(
      `GET ${baseUrl}/templates returned ${res.status}; ensure ogis is running before starting k6`
    );
  }
  return res.json().templates || [];
}

/**
 * Filter a template list to a desired subset for the benchmark.
 *
 * @param {string[]} all - Full template list (typically from fetchAllTemplates).
 * @param {Object} [opts]
 * @param {boolean} [opts.includeStatic=true]
 * @param {boolean} [opts.includeGradients=true]
 * @param {string|string[]|null} [opts.gradientLayouts=null]
 *   Restrict gradient templates to these layout name(s). Pass null/undefined
 *   to include every layout (864 total).
 * @param {number|null} [opts.gradientSample=null]
 *   Cap the gradient list (after layout filtering, sorted order).
 */
export function filterTemplates(all, opts = {}) {
  const {
    includeStatic = true,
    includeGradients = true,
    gradientLayouts = null,
    gradientSample = null,
  } = opts;

  const isGradient = (name) => name.startsWith('gradient-');
  const layoutOf = (name) => {
    // gradient-{gradient}-{layout} where layout itself may contain hyphens
    // (e.g. left-heavy). The gradient name has no hyphens, so the layout is
    // everything after the second hyphen.
    const idx = name.indexOf('-', 'gradient-'.length);
    return idx >= 0 ? name.slice(idx + 1) : '';
  };
  const wantedLayouts = (() => {
    if (gradientLayouts == null) return null;
    const arr = Array.isArray(gradientLayouts) ? gradientLayouts : [gradientLayouts];
    return new Set(arr);
  })();

  let staticNames = [];
  let gradients = [];
  for (const name of all) {
    if (isGradient(name)) {
      if (!includeGradients) continue;
      if (wantedLayouts && !wantedLayouts.has(layoutOf(name))) continue;
      gradients.push(name);
    } else {
      if (!includeStatic) continue;
      staticNames.push(name);
    }
  }
  staticNames.sort();
  gradients.sort();

  if (gradientSample && gradientSample > 0 && gradientSample < gradients.length) {
    gradients = gradients.slice(0, gradientSample);
  }

  return [...staticNames, ...gradients];
}

/**
 * Sanitize template name for k6 metric names (no hyphens allowed)
 */
export function sanitize(name) {
  return name.replace(/-/g, '_');
}
