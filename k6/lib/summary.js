import { htmlReport } from 'https://raw.githubusercontent.com/benc-uk/k6-reporter/main/dist/bundle.js';

/**
 * Format milliseconds value for display
 */
export function formatMs(v) {
  return v !== undefined ? `${v.toFixed(0)}ms` : 'N/A';
}

/**
 * Extract common metrics from k6 data
 */
export function extractMetrics(data) {
  const m = data.metrics;
  return {
    reqDuration: m.http_req_duration?.values || {},
    httpReqs: m.http_reqs?.values || {},
    httpFailed: m.http_req_failed?.values || {},
  };
}

/**
 * Generate the overall metrics markdown table
 */
export function generateOverallTable(metrics) {
  const { reqDuration, httpReqs, httpFailed } = metrics;

  return `| Metric | Value |
|--------|-------|
| Total Requests | ${httpReqs.count?.toFixed(0) || 'N/A'} |
| Requests/sec | ${httpReqs.rate?.toFixed(2) || 'N/A'} |
| Median Latency | ${formatMs(reqDuration.med)} |
| P95 Latency | ${formatMs(reqDuration['p(95)'])} |
| P99 Latency | ${formatMs(reqDuration['p(99)'])} |
| Failed | ${((httpFailed.rate || 0) * 100).toFixed(2)}% |`;
}

/**
 * Generate the default template comparison table
 */
export function generateDefaultTable(data) {
  const defaultMetric = data.metrics.template_default?.values;
  if (!defaultMetric) return '';

  return `### Default Template (for comparison)

| Metric | Value |
|--------|-------|
| Min | ${formatMs(defaultMetric.min)} |
| Avg | ${formatMs(defaultMetric.avg)} |
| Median | ${formatMs(defaultMetric.med)} |
| P90 | ${formatMs(defaultMetric['p(90)'])} |
| P95 | ${formatMs(defaultMetric['p(95)'])} |
| Max | ${formatMs(defaultMetric.max)} |`;
}

/**
 * Generate standard output files for k6 results
 */
export function generateOutputFiles(data, markdown) {
  return {
    stdout: JSON.stringify(data, null, 2),
    'k6/results/summary.md': markdown,
    'k6/results/summary.html': htmlReport(data),
    'k6/results/results.json': JSON.stringify(data, null, 2),
  };
}

/**
 * Create a simple summary handler without per-template metrics
 *
 * @param {string} title - Title for the benchmark results
 */
export function createSummaryHandler(title) {
  return function handleSummary(data) {
    const metrics = extractMetrics(data);
    const defaultTable = generateDefaultTable(data);

    const markdown = `## ${title}

### Overall

${generateOverallTable(metrics)}

${defaultTable}
`;

    return generateOutputFiles(data, markdown);
  };
}
