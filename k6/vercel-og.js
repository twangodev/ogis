import { createDefaultMetrics } from './lib/metrics.js';
import { createOptions } from './lib/config.js';
import { randomTitle, randomDescription } from './lib/utils.js';
import { createTestRunner } from './lib/runner.js';
import { createSummaryHandler } from './lib/summary.js';

const baseUrl = __ENV.BASE_URL || 'http://localhost:3001';
const defaultMetrics = createDefaultMetrics();

// createOptions() takes an optional thresholds-override map (no longer takes a
// templates list — the gradient-cache-aware refactor moved that into setup()).
export const options = createOptions();

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
