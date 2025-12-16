import { createDefaultMetrics } from './lib/metrics.js';
import { createOptions } from './lib/config.js';
import { randomTitle, randomDescription } from './lib/utils.js';
import { createTestRunner } from './lib/runner.js';
import { createSummaryHandler } from './lib/summary.js';

const baseUrl = __ENV.BASE_URL || 'http://localhost:3001';
const defaultMetrics = createDefaultMetrics();

export const options = createOptions(['default']);

function buildUrl() {
  return `${baseUrl}/?title=${encodeURIComponent(randomTitle())}&description=${encodeURIComponent(randomDescription())}`;
}

const runner = createTestRunner({
  getUrl: buildUrl,
  onResponse: (res) => defaultMetrics.add(res.timings.duration),
});

// Both baseline and default use the same runner (single template)
export const baseline = runner;
export default runner;

export const handleSummary = createSummaryHandler('Vercel OG Benchmark Results');
