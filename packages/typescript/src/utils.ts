import type { OgisParams } from './types.js';

/**
 * Merge default parameters with request-specific parameters
 *
 * Request parameters override defaults.
 * Undefined values are filtered out.
 *
 * @param defaults - Default parameters
 * @param params - Request-specific parameters
 * @returns Merged parameters
 *
 * @internal
 */
export function mergeParams(
  defaults: Partial<OgisParams> | undefined,
  params: OgisParams,
): Record<string, string> {
  const merged: Record<string, string> = {};

  // Apply defaults first
  if (defaults) {
    for (const [key, value] of Object.entries(defaults)) {
      if (value !== undefined) {
        merged[key] = value;
      }
    }
  }

  // Override with request params
  for (const [key, value] of Object.entries(params)) {
    if (value !== undefined) {
      merged[key] = value;
    }
  }

  return merged;
}

/**
 * Build query string from parameters
 *
 * @param params - Query parameters
 * @returns URL-encoded query string (e.g., "?key1=value1&key2=value2")
 *
 * @internal
 */
export function buildQueryString(params: Record<string, string>): string {
  const searchParams = new URLSearchParams();

  for (const [key, value] of Object.entries(params)) {
    searchParams.append(key, value);
  }

  const queryString = searchParams.toString();
  return queryString ? `?${queryString}` : '';
}

/**
 * Validate base URL format
 *
 * @param baseUrl - Base URL to validate
 * @throws {Error} If URL is invalid
 *
 * @internal
 */
export function validateBaseUrl(baseUrl: string): void {
  if (!baseUrl) {
    throw new Error('baseUrl is required');
  }

  try {
    const url = new URL(baseUrl);
    if (!url.protocol.startsWith('http')) {
      throw new Error('baseUrl must use http or https protocol');
    }
  } catch (error) {
    if (error instanceof TypeError) {
      throw new Error(`Invalid baseUrl format: ${baseUrl}`);
    }
    throw error;
  }
}

/**
 * Normalize base URL (remove trailing slash)
 *
 * @param baseUrl - Base URL to normalize
 * @returns Normalized URL
 *
 * @internal
 */
export function normalizeBaseUrl(baseUrl: string): string {
  return baseUrl.replace(/\/$/, '');
}
