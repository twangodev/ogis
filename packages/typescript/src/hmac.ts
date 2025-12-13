import { createHmac } from 'node:crypto';

/**
 * Generate HMAC-SHA256 signature for ogis request
 *
 * Algorithm (matches Rust implementation):
 * 1. Sort parameters alphabetically by key
 * 2. URL-encode all parameter keys and values
 * 3. Build canonical query string (e.g., "a=1&b=2&c=3")
 * 4. Compute HMAC-SHA256(secret, canonical_query_string)
 * 5. Return hex-encoded signature
 *
 * @param secret - HMAC secret key
 * @param params - Query parameters (signature param will be excluded)
 * @returns Hex-encoded HMAC signature
 *
 * @example
 * ```ts
 * const signature = generateHmacSignature('my-secret', {
 *   title: 'Hello',
 *   description: 'World'
 * });
 * // Returns: "a1b2c3d4..." (hex-encoded HMAC)
 * ```
 */
export function generateHmacSignature(
  secret: string,
  params: Record<string, string | undefined>
): string {
  const canonicalQuery = buildCanonicalQuery(params);
  const hmac = createHmac('sha256', secret);
  hmac.update(canonicalQuery);
  return hmac.digest('hex');
}

/**
 * Build canonical query string from parameters
 *
 * - Parameters are sorted alphabetically by key
 * - 'signature' parameter is excluded
 * - All keys and values are URL-encoded
 * - Format: "key1=value1&key2=value2"
 *
 * @param params - Query parameters
 * @returns Canonical query string
 *
 * @internal
 */
export function buildCanonicalQuery(
  params: Record<string, string | undefined>
): string {
  // Sort parameters alphabetically and filter out undefined values and 'signature'
  const sortedKeys = Object.keys(params)
    .filter((key) => key !== 'signature' && params[key] !== undefined)
    .sort();

  // Build canonical query string with URL encoding
  const parts = sortedKeys.map((key) => {
    const value = params[key];
    // Encode both key and value to prevent injection attacks
    return `${encodeURIComponent(key)}=${encodeURIComponent(value!)}`;
  });

  return parts.join('&');
}
