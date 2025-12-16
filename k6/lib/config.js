/**
 * Configuration from environment variables
 *
 * REQUESTS_PER_TEMPLATE: 50 default for statistically significant P95 estimates
 * (need 30+ samples minimum for reliable percentile calculations)
 */
export const config = {
  mode: __ENV.MODE || 'sequential', // 'sequential' or 'concurrent'
  requestsPerTemplate: parseInt(__ENV.REQUESTS_PER_TEMPLATE || '50'),
  vus: parseInt(__ENV.VUS || '100'),
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

  return {
    concurrent: {
      executor: 'constant-vus',
      vus: config.vus,
      duration: config.duration,
    },
  };
}