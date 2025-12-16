import { sanitize } from './templates.js';
import { extractMetrics, formatMs, generateOverallTable, generateDefaultTable, generateOutputFiles } from './summary.js';

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
    const metrics = extractMetrics(data);

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

    const defaultTable = generateDefaultTable(data);

    const markdown = `## OGIS Benchmark Results

### Overall

${generateOverallTable(metrics)}

${defaultTable}

### Per Template

| Template | Median (ms) | P95 (ms) |
|----------|-------------|----------|
${templateRows}
`;

    return generateOutputFiles(data, markdown);
  };
}
