import { htmlReport } from 'https://raw.githubusercontent.com/benc-uk/k6-reporter/main/dist/bundle.js';
import { sanitize } from './templates.js';

const WORDS = [
  'cloud', 'stream', 'pixel', 'graph', 'node', 'data', 'sync', 'mesh',
  'flux', 'core', 'edge', 'pulse', 'grid', 'wave', 'link', 'flow',
  'spark', 'beam', 'shift', 'scale', 'swift', 'prime', 'apex', 'nexus',
];

const ADJECTIVES = [
  'fast', 'smart', 'bold', 'fresh', 'sleek', 'agile', 'rapid', 'smooth',
  'sharp', 'clear', 'bright', 'dynamic', 'modern', 'elegant', 'powerful',
];

function randomFrom(arr) {
  return arr[Math.floor(Math.random() * arr.length)];
}

/**
 * Generate a random title (2-5 words, capitalized)
 */
export function randomTitle() {
  const count = 2 + Math.floor(Math.random() * 4);
  const parts = [];

  if (Math.random() > 0.5) {
    parts.push(randomFrom(ADJECTIVES));
  }

  while (parts.length < count) {
    parts.push(randomFrom(WORDS));
  }

  return parts.map((w) => w.charAt(0).toUpperCase() + w.slice(1)).join(' ');
}

/**
 * Generate a random description (10-20 words)
 */
export function randomDescription() {
  const count = 10 + Math.floor(Math.random() * 11);
  const parts = [];

  for (let i = 0; i < count; i++) {
    parts.push(i % 3 === 0 && Math.random() > 0.6 ? randomFrom(ADJECTIVES) : randomFrom(WORDS));
  }

  const sentence = parts.join(' ');
  return sentence.charAt(0).toUpperCase() + sentence.slice(1) + '.';
}

/**
 * Build a handleSummary function that includes per-template metrics
 */
export function buildHandleSummary(templates) {
  return function handleSummary(data) {
    const m = data.metrics;
    const reqDuration = m.http_req_duration?.values || {};
    const httpReqs = m.http_reqs?.values || {};
    const httpFailed = m.http_req_failed?.values || {};

    const templateRows = templates
      .map((t) => {
        const metric = m[`template_${sanitize(t)}`]?.values;
        if (!metric || metric.med === undefined) return null;
        return { name: t, med: metric.med, p95: metric['p(95)'] };
      })
      .filter(Boolean)
      .sort((a, b) => a.med - b.med)
      .map((t) => `| ${t.name} | ${t.med?.toFixed(0) || '-'} | ${t.p95?.toFixed(0) || '-'} |`)
      .join('\n');

    const fmtMs = (v) => (v !== undefined ? `${v.toFixed(0)}ms` : 'N/A');

    const markdown = `## OGIS Benchmark Results

### Overall

| Metric | Value |
|--------|-------|
| Total Requests | ${httpReqs.count?.toFixed(0) || 'N/A'} |
| Requests/sec | ${httpReqs.rate?.toFixed(2) || 'N/A'} |
| Median Latency | ${fmtMs(reqDuration.med)} |
| P95 Latency | ${fmtMs(reqDuration['p(95)'])} |
| P99 Latency | ${fmtMs(reqDuration['p(99)'])} |
| Failed | ${((httpFailed.rate || 0) * 100).toFixed(2)}% |

### Per Template

| Template | Median (ms) | P95 (ms) |
|----------|-------------|----------|
${templateRows}
`;

    return {
      stdout: JSON.stringify(data, null, 2),
      'k6/results/summary.md': markdown,
      'k6/results/summary.html': htmlReport(data),
      'k6/results/results.json': JSON.stringify(data, null, 2),
    };
  };
}
