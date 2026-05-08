/**
 * Configuration from environment variables
 *
 * Modes:
 * - sequential: one VU rotates through templates with REQUESTS_PER_TEMPLATE
 *   hits each, capped by SEQUENTIAL_MAX_ITERATIONS / sequentialMaxDuration.
 * - baseline: spam only default template (for head-to-head comparison)
 * - concurrent: full send all templates randomly
 * - cache_pressure: random across templates with a unique color override per
 *   iteration so every request is a cold gradient-cache miss (worst case).
 *
 * Gradient template controls:
 * - INCLUDE_STATIC (default true) — include file-based templates.
 * - INCLUDE_GRADIENTS (default true) — append gradient-* templates.
 * - GRADIENT_LAYOUTS (default 'centered') — comma-separated layout names. Use
 *   'all' to fan out across every layout (864 templates). The full set easily
 *   exceeds the gradient cache budget, which is the point of cache_pressure.
 * - GRADIENT_SAMPLE (default unset) — cap the gradient list to this many
 *   entries (after layout filtering, in deterministic order).
 */

function parseLayouts(value) {
  if (!value || value === 'all') return null; // null means "every layout"
  return value
    .split(',')
    .map((s) => s.trim())
    .filter(Boolean);
}

export const config = {
  mode: __ENV.MODE || 'sequential',
  requestsPerTemplate: parseInt(__ENV.REQUESTS_PER_TEMPLATE || '50'),
  vus: parseInt(__ENV.VUS || '10'),
  duration: __ENV.DURATION || '60s',
  baseUrl: __ENV.BASE_URL || 'http://localhost:3000',
  includeStatic: (__ENV.INCLUDE_STATIC || 'true').toLowerCase() !== 'false',
  includeGradients: (__ENV.INCLUDE_GRADIENTS || 'true').toLowerCase() !== 'false',
  gradientLayouts: parseLayouts(__ENV.GRADIENT_LAYOUTS || 'centered'),
  gradientSample: __ENV.GRADIENT_SAMPLE ? parseInt(__ENV.GRADIENT_SAMPLE) : null,
  sequentialMaxIterations: parseInt(__ENV.SEQUENTIAL_MAX_ITERATIONS || '20000'),
  sequentialMaxDuration: __ENV.SEQUENTIAL_MAX_DURATION || '30m',
};

/**
 * Build k6 scenarios based on mode. Length-independent — the actual template
 * list is discovered from setup() so options can be exported at init time.
 */
export function buildScenarios() {
  if (config.mode === 'sequential') {
    return {
      sequential: {
        executor: 'shared-iterations',
        vus: 1,
        iterations: config.sequentialMaxIterations,
        maxDuration: config.sequentialMaxDuration,
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

  if (config.mode === 'cache_pressure') {
    return {
      cache_pressure: {
        executor: 'constant-vus',
        vus: config.vus,
        duration: config.duration,
        exec: 'cachePressure',
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
  http_req_duration: ['p(95)<5000'],
  http_req_failed: ['rate<0.05'],
};

/**
 * Create k6 options with scenarios and thresholds.
 *
 * @param {Object} [customThresholds] - Optional custom thresholds to merge
 */
export function createOptions(customThresholds = {}) {
  return {
    scenarios: buildScenarios(),
    thresholds: {
      ...defaultThresholds,
      ...customThresholds,
    },
    summaryTrendStats: ['avg', 'min', 'med', 'max', 'p(90)', 'p(95)', 'p(99)'],
  };
}
