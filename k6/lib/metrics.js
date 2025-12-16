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

/**
 * Create default template metrics (for head-to-head comparison)
 */
export function createDefaultMetrics() {
  const trend = new Trend('template_default', true);
  return {
    add(duration) {
      trend.add(duration);
    },
  };
}