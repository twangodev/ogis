import { Trend } from 'k6/metrics';
import { sanitize } from './templates.js';

/**
 * Create per-template Trend metrics for the listed names.
 *
 * Used by ogis.js for the static templates (whose names we know at init time)
 * and the chosen BASELINE_TEMPLATE if that one happens to be a gradient. Other
 * gradient templates are reported via the aggregate trend below.
 */
export function createTemplateMetrics(templates) {
  const metrics = {};
  for (const t of templates) {
    metrics[t] = new Trend(`template_${sanitize(t)}`, true);
  }
  return metrics;
}

/**
 * Aggregate trend covering every gradient-* response that doesn't have a
 * dedicated per-template metric. Lets us report the gradient family's overall
 * latency without having to declare 864 separate metrics at init.
 */
export function createGradientAggregate() {
  return new Trend('template_gradient_all', true);
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
