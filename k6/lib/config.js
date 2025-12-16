/**
 * Configuration from environment variables
 *
 * Modes:
 * - sequential: one-by-one testing of all templates (for per-template stats)
 * - baseline: spam only default template (for head-to-head comparison)
 * - concurrent: full send all templates randomly
 */
export const config = {
  mode: __ENV.MODE || 'sequential',
  requestsPerTemplate: parseInt(__ENV.REQUESTS_PER_TEMPLATE || '50'),
  vus: parseInt(__ENV.VUS || '10'),
  duration: __ENV.DURATION || '60s',
  baseUrl: __ENV.BASE_URL || 'http://localhost:3000',
};

/**
 * Build k6 scenarios based on mode
 */
export function buildScenarios(templates) {
  if (config.mode === 'sequential') {
    return {
      sequential: {
        executor: 'per-vu-iterations',
        vus: 1,
        iterations: templates.length * config.requestsPerTemplate,
        maxDuration: '30m',
      },
    };
  }

  if (config.mode === 'baseline') {
    return {
      baseline: {
        executor: 'constant-vus',
        vus: config.vus,
        duration: config.duration,
        exec: 'baseline',
      },
    };
  }

  // concurrent: full send all templates
  return {
    concurrent: {
      executor: 'constant-vus',
      vus: config.vus,
      duration: config.duration,
    },
  };
}

/**
 * Default thresholds for OG image generation benchmarks
 */
export const defaultThresholds = {
  http_req_duration: ['p(95)<3000'],
  http_req_failed: ['rate<0.01'],
};

/**
 * Create k6 options with scenarios and thresholds
 *
 * @param {string[]} templates - List of template names
 * @param {Object} [customThresholds] - Optional custom thresholds to merge
 */
export function createOptions(templates, customThresholds = {}) {
  return {
    scenarios: buildScenarios(templates),
    thresholds: {
      ...defaultThresholds,
      ...customThresholds,
    },
    summaryTrendStats: ['avg', 'min', 'med', 'max', 'p(90)', 'p(95)', 'p(99)'],
  };
}