import { Trend } from 'k6/metrics';
import { sanitize } from './templates.js';

/**
 * Create per-template Trend metrics
 */
export function createTemplateMetrics(templates) {
  const metrics = {};
  for (const t of templates) {
    metrics[t] = new Trend(`template_${sanitize(t)}`, true);
  }
  return metrics;
}