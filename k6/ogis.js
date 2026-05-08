import { loadTemplates } from './lib/templates.js';
import { createTemplateMetrics, createDefaultMetrics } from './lib/metrics.js';
import { config, createOptions } from './lib/config.js';
import { randomTitle, randomDescription, buildHandleSummary } from './lib/utils.js';
import { createTestRunner } from './lib/runner.js';

const templates = loadTemplates();
const templateMetrics = createTemplateMetrics(templates);
const defaultMetrics = createDefaultMetrics();

const DEFAULT_TEMPLATE = __ENV.BASELINE_TEMPLATE || 'minimal';

export const options = createOptions(templates);

let currentTemplate;

function selectTemplate() {
  return config.mode === 'sequential'
    ? templates[Math.floor(__ITER / config.requestsPerTemplate) % templates.length]
    : templates[Math.floor(Math.random() * templates.length)];
}

function buildUrl(template) {
  return `${config.baseUrl}/?template=${template}&title=${encodeURIComponent(randomTitle())}&description=${encodeURIComponent(randomDescription())}`;
}

// Baseline scenario: only test default template (for fair comparison with vercel-og)
export const baseline = createTestRunner({
  getUrl: () => buildUrl(DEFAULT_TEMPLATE),
  onResponse: (res) => {
    templateMetrics[DEFAULT_TEMPLATE]?.add(res.timings.duration);
    defaultMetrics.add(res.timings.duration);
  },
});

// Default/load scenario: test all templates
export default createTestRunner({
  getUrl: () => {
    currentTemplate = selectTemplate();
    return buildUrl(currentTemplate);
  },
  onResponse: (res) => {
    templateMetrics[currentTemplate].add(res.timings.duration);
    if (currentTemplate === DEFAULT_TEMPLATE) {
      defaultMetrics.add(res.timings.duration);
    }
  },
});

export const handleSummary = buildHandleSummary(templates);
