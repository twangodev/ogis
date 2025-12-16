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
 * Create a test runner function with customizable URL generation and response handling
 *
 * @param {Object} options
 * @param {Function} options.getUrl - Function that returns the URL to request
 * @param {Function} [options.onResponse] - Optional callback to handle response (e.g., record metrics)
 * @param {Object} [options.checks] - Optional custom checks (defaults to defaultChecks)
 */
export function createTestRunner({ getUrl, onResponse, checks }) {
  return function () {
    const url = getUrl();
    const res = http.get(url);

    if (onResponse) {
      onResponse(res);
    }

    check(res, checks || defaultChecks);

    if (config.mode === 'concurrent') {
      sleep(0.1);
    }
  };
}
