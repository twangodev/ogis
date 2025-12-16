import http from 'k6/http';
import { check, sleep } from 'k6';

import { loadTemplates } from './lib/templates.js';
import { createTemplateMetrics } from './lib/metrics.js';
import { config, buildScenarios } from './lib/config.js';
import { randomTitle, randomDescription, buildHandleSummary } from './lib/utils.js';

// Setup
const templates = loadTemplates();
const templateMetrics = createTemplateMetrics(templates);

export const options = {
  scenarios: buildScenarios(templates),
  thresholds: {
    http_req_duration: ['p(95)<3000'],
    http_req_failed: ['rate<0.01'],
  },
};

export default function () {
  const template =
    config.mode === 'sequential'
      ? templates[Math.floor(__ITER / config.requestsPerTemplate) % templates.length]
      : templates[Math.floor(Math.random() * templates.length)];

  const url = `${config.baseUrl}/?template=${template}&title=${encodeURIComponent(randomTitle())}&description=${encodeURIComponent(randomDescription())}`;

  const res = http.get(url, { tags: { template } });
  templateMetrics[template].add(res.timings.duration);

  check(res, {
    'status is 200': (r) => r.status === 200,
    'content-type is png': (r) => r.headers['Content-Type']?.includes('image/png'),
  });

  if (config.mode === 'concurrent') {
    sleep(0.1);
  }
}

export const handleSummary = buildHandleSummary(templates);
