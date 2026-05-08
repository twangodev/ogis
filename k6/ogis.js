import {
  loadStaticTemplates,
  fetchAllTemplates,
  filterTemplates,
} from './lib/templates.js';
import {
  createTemplateMetrics,
  createDefaultMetrics,
  createGradientAggregate,
} from './lib/metrics.js';
import { config, createOptions } from './lib/config.js';
import { randomTitle, randomDescription, buildHandleSummary } from './lib/utils.js';
import { createTestRunner } from './lib/runner.js';

// Init context — we know static templates from templates.yaml. Per-template
// Trend metrics need to be declared here. Gradient templates are auto-composed
// (864 of them) and discovered from the running server in setup(); they share
// a single aggregate trend so we don't blow up the metric registry.
const staticTemplates = loadStaticTemplates();

const DEFAULT_TEMPLATE = __ENV.BASELINE_TEMPLATE || 'minimal';
const knownTemplateNames = Array.from(new Set([...staticTemplates, DEFAULT_TEMPLATE]));

const templateMetrics = createTemplateMetrics(knownTemplateNames);
const gradientAggregate = createGradientAggregate();
const defaultMetrics = createDefaultMetrics();

// Scenarios are length-independent (constant-vus + duration / shared-iterations
// with a fixed cap), so options can be defined without knowing the gradient
// list yet.
export const options = createOptions();

export function setup() {
  const all = fetchAllTemplates(config.baseUrl);
  const templates = filterTemplates(all, {
    includeStatic: config.includeStatic,
    includeGradients: config.includeGradients,
    gradientLayouts: config.gradientLayouts,
    gradientSample: config.gradientSample,
  });
  if (templates.length === 0) {
    throw new Error(
      `[setup] filtered template list is empty — adjust INCLUDE_STATIC / INCLUDE_GRADIENTS / GRADIENT_LAYOUTS / GRADIENT_SAMPLE so at least one template is selected (server returned ${all.length} total)`
    );
  }
  console.log(
    `[setup] using ${templates.length} template(s): ${
      templates.filter((t) => !t.startsWith('gradient-')).length
    } static + ${templates.filter((t) => t.startsWith('gradient-')).length} gradient`
  );
  return { templates };
}

let currentTemplate;

function selectTemplate(templates) {
  return config.mode === 'sequential'
    ? templates[Math.floor(__ITER / config.requestsPerTemplate) % templates.length]
    : templates[Math.floor(Math.random() * templates.length)];
}

function buildUrl(template, extraParams = '') {
  const base = `${config.baseUrl}/?template=${encodeURIComponent(template)}&title=${encodeURIComponent(randomTitle())}&description=${encodeURIComponent(randomDescription())}`;
  return extraParams ? `${base}${extraParams}` : base;
}

function uniqueColor() {
  // VU + iter combine into a six-char hex value that doesn't repeat within a
  // single benchmark run.
  const seed = (__VU * 1_000_003 + __ITER) % 0xffffff;
  return seed.toString(16).padStart(6, '0');
}

function recordResponse(template, durationMs) {
  const metric = templateMetrics[template];
  if (metric) {
    metric.add(durationMs);
  } else if (template.startsWith('gradient-')) {
    gradientAggregate.add(durationMs);
  } else {
    // Static template that wasn't in templates.yaml at init time (server has
    // drifted ahead of the file) — log once instead of silently bucketing into
    // the gradient aggregate, which would mask the drift.
    console.warn(`[record] unknown non-gradient template '${template}' — not tallied; regen templates.yaml`);
  }
  if (template === DEFAULT_TEMPLATE) {
    defaultMetrics.add(durationMs);
  }
}

// Baseline scenario: only test default template (for fair comparison with vercel-og)
export const baseline = createTestRunner({
  getUrl: () => buildUrl(DEFAULT_TEMPLATE),
  onResponse: (res) => recordResponse(DEFAULT_TEMPLATE, res.timings.duration),
});

// Cache-pressure scenario: each iteration picks a random template AND attaches
// a unique color override so every request derives a fresh gradient cache key.
export const cachePressure = createTestRunner({
  getUrl: (data) => {
    currentTemplate = selectTemplate(data.templates);
    return buildUrl(currentTemplate, `&background=${uniqueColor()}`);
  },
  onResponse: (res) => recordResponse(currentTemplate, res.timings.duration),
});

// Default/load scenario: random across the filtered template list
export default createTestRunner({
  getUrl: (data) => {
    currentTemplate = selectTemplate(data.templates);
    return buildUrl(currentTemplate);
  },
  onResponse: (res) => recordResponse(currentTemplate, res.timings.duration),
});

export const handleSummary = buildHandleSummary(knownTemplateNames);
