import http from 'k6/http';
import { check, sleep } from 'k6';
import { config } from './config.js';

/**
 * Default checks for OG image endpoints
 */
export const defaultChecks = {
  'status is 200': (r) => r.status === 200,
  'content-type is png': (r) => r.headers['Content-Type']?.includes('image/png'),
};

/**
 * Create a test runner function with customizable URL generation and response handling.
 *
 * The returned function takes the standard k6 `data` argument (populated from
 * `setup()`) and forwards it to `getUrl` so dynamic content like the live
 * template list is reachable.
 *
 * @param {Object} options
 * @param {Function} options.getUrl - `(data) => string` returning the URL to request
 * @param {Function} [options.onResponse] - Optional callback to handle response
 * @param {Object} [options.checks] - Optional custom checks (defaults to defaultChecks)
 */
export function createTestRunner({ getUrl, onResponse, checks }) {
  return function (data) {
    const url = getUrl(data);
    const res = http.get(url);

    if (onResponse) {
      onResponse(res);
    }

    check(res, checks || defaultChecks);

    if (config.mode === 'concurrent' || config.mode === 'cache_pressure') {
      sleep(0.1);
    }
  };
}
